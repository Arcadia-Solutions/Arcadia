use std::sync::Arc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use bip_util::sha::ShaHash;
use builder::queue::Queue;

use accessor::{Accessor, PieceAccess};
use builder::buffer::{PieceBuffers, PieceBuffer};
use error::ParseResult;

/// Messages sent to the master hasher.
pub enum MasterMessage {
    /// Accepts the piece hash with the given piece index.
    AcceptPiece(usize, ShaHash),
    /// One of our workers has finished and shut down.
    WorkerFinished,
}

/// Message sent to a worker hasher.
enum WorkerMessage {
    /// Start processing the given memory region.
    HashPiece(usize, PieceBuffer),
    /// Worker should exit it's thread.
    Finish,
}

/// Starts a number of hasher workers which will generate the hash pieces for the files we send to it.
pub fn start_hasher_workers<A, C>(accessor: A,
                                  piece_length: usize,
                                  num_pieces: u64,
                                  num_workers: usize,
                                  progress: C)
                                  -> ParseResult<Vec<(usize, ShaHash)>>
    where A: Accessor,
          C: FnMut(f64) + Send + 'static
{
    // Create channels to communicate with the master
    let (master_send, master_recv) = mpsc::channel();
    let (prog_send, prog_recv) = mpsc::channel();

    // Create queue to push work to and pull work from
    let work_queue = Arc::new(Queue::new());

    // Create buffer allocator to reuse pre allocated buffers
    let piece_buffers = Arc::new(PieceBuffers::new(piece_length, num_workers));

    // Create n worker threads that pull work from the queue
    for _ in 0..num_workers {
        let share_master_send = master_send.clone();
        let share_work_queue = work_queue.clone();
        let share_piece_buffers = piece_buffers.clone();

        thread::spawn(move || {
            start_hash_worker(share_master_send, share_work_queue, share_piece_buffers);
        });
    }

    // Create a worker thread to execute the user callback for the progress update
    thread::spawn(move || {
        start_progress_updater(prog_recv, num_pieces, progress);
    });

    // Create the master worker to coordinate between the workers
    start_hash_master(accessor,
                      num_workers,
                      master_recv,
                      work_queue,
                      piece_buffers,
                      prog_send)
}

// ----------------------------------------------------------------------------//

/// Start a master hasher which will take care of chunking sequential/overlapping pieces from the data given to it and giving
/// updates to the hasher workers.
fn start_hash_master<A>(accessor: A,
                        num_workers: usize,
                        recv: Receiver<MasterMessage>,
                        work: Arc<Queue<WorkerMessage>>,
                        buffers: Arc<PieceBuffers>,
                        progress_sender: Sender<usize>)
                        -> ParseResult<Vec<(usize, ShaHash)>>
    where A: Accessor
{
    let mut pieces = Vec::new();
    let mut piece_index = 0;

    // Our closure may be called multiple times, save partial pieces buffers between calls
    let mut opt_piece_buffer = None;
    try!(accessor.access_pieces(|piece_access| {
        match piece_access {
            PieceAccess::Compute(piece_region) => {
                let mut curr_piece_buffer = if let Some(piece_buffer) = opt_piece_buffer.take() {
                    piece_buffer
                } else {
                    buffers.checkout()
                };

                let mut end_of_region = false;
                while !end_of_region {
                    end_of_region =
                        try!(curr_piece_buffer.write_bytes(|buffer| piece_region.read(buffer))) == 0;

                    if curr_piece_buffer.is_whole() {
                        work.push(WorkerMessage::HashPiece(piece_index, curr_piece_buffer));

                        piece_index += 1;
                        curr_piece_buffer = buffers.checkout();

                        if progress_sender.send(piece_index).is_err() {
                            // TODO: Add logging here
                        }
                    }
                }

                opt_piece_buffer = Some(curr_piece_buffer);
            },
            PieceAccess::PreComputed(hash) => {
                pieces.push((piece_index, hash));

                piece_index += 1;
            }
        }

        Ok(())
    }));

    // If we still have a partial piece left over, push it to the workers
    if let Some(piece_buffer) = opt_piece_buffer {
        if !piece_buffer.is_empty() {
            work.push(WorkerMessage::HashPiece(piece_index, piece_buffer));

            piece_index += 1;
            if progress_sender.send(piece_index).is_err() {
                // TODO: Add logging here
            }
        }
    }

    // No more entries, tell workers to shut down
    for _ in 0..num_workers {
        work.push(WorkerMessage::Finish);
    }

    // Wait for all of the workers to finish up the last pieces
    let mut workers_finished = 0;
    while workers_finished < num_workers {
        match recv.recv() {
            Ok(MasterMessage::AcceptPiece(index, piece)) => pieces.push((index, piece)),
            Ok(MasterMessage::WorkerFinished) => workers_finished += 1,
            Err(_) => panic!("bip_metainfo: Master failed to verify all workers shutdown..."),
        }
    }

    // Sort our list to make sure the pieces are in order before we send them off
    pieces.sort_by(|one, two| one.0.cmp(&two.0));

    Ok(pieces)
}

// ----------------------------------------------------------------------------//

fn start_progress_updater<C>(recv: Receiver<usize>, num_pieces: u64, mut progress: C)
    where C: FnMut(f64)
{
    for finished_piece in recv {
        let percent_complete = (finished_piece as f64) / (num_pieces as f64);

        progress(percent_complete);
    }
}

// ----------------------------------------------------------------------------//

/// Starts a hasher worker which will hash all of the buffers it receives.
fn start_hash_worker(send: Sender<MasterMessage>,
                     work: Arc<Queue<WorkerMessage>>,
                     buffers: Arc<PieceBuffers>) {
    let mut work_to_do = true;

    // Loop until we are instructed to stop working
    while work_to_do {
        let work_item = work.pop();

        match work_item {
            WorkerMessage::Finish => {
                work_to_do = false;
            }
            WorkerMessage::HashPiece(index, buffer) => {
                let hash = ShaHash::from_bytes(buffer.as_slice());

                send.send(MasterMessage::AcceptPiece(index, hash)).unwrap();
                buffers.checkin(buffer);
            }
        }
    }

    // Let the master know we have exited our thread
    send.send(MasterMessage::WorkerFinished).unwrap();
}

#[cfg(test)]
mod tests {
    use std::ops::{Range, Index};
    use std::io::{self, Cursor};
    use std::path::Path;
    use std::sync::mpsc;

    use bip_util::sha::ShaHash;
    use rand::{self, RngCore};

    use accessor::{Accessor, PieceAccess};
    use builder::worker;

    // Keep these numbers fairly small to avoid lengthy tests
    const DEFAULT_PIECE_LENGTH: usize = 1024;
    const DEFAULT_NUM_PIECES: usize = 300;

    // Mock object for providing direct access to bytes via DataEntry.
    #[derive(Clone)]
    struct MockAccessor {
        buffer_ranges: Vec<Range<usize>>,
        contiguous_buffer: Vec<u8>,
    }

    impl MockAccessor {
        fn new() -> MockAccessor {
            MockAccessor {
                buffer_ranges: Vec::new(),
                contiguous_buffer: Vec::new(),
            }
        }

        fn create_region(&mut self, num_bytes: usize) {
            let mut buffer = vec![0u8; num_bytes];
            let mut rng = rand::rng();

            rng.fill_bytes(&mut buffer);

            let (begin, end) = (self.contiguous_buffer.len(),
                                self.contiguous_buffer.len() + buffer.len());

            self.contiguous_buffer.extend_from_slice(&buffer);
            self.buffer_ranges.push(begin..end);
        }

        fn as_slice(&self) -> &[u8] {
            &self.contiguous_buffer
        }
    }

    impl Accessor for MockAccessor {
        /// Access the directory that all files should be relative to.
        fn access_directory(&self) -> Option<&Path> {
            panic!("Accessor::access_directory should not be called with MockAccessor...");
        }

        /// Access the metadata for all files including their length and path.
        fn access_metadata<C>(&self, _: C) -> io::Result<()>
            where C: FnMut(u64, &Path)
        {
            panic!("Accessor::access_metadata should not be called with MockAccessor...");
        }

        /// Access the sequential pieces that make up all of the files.
        fn access_pieces<C>(&self, mut callback: C) -> io::Result<()>
            where C: for<'a> FnMut(PieceAccess<'a>) -> io::Result<()>
        {
            for range in self.buffer_ranges.iter() {
                let mut next_region = Cursor::new(self.contiguous_buffer.index(range.clone()));

                try!(callback(PieceAccess::Compute(&mut next_region)));
            }

            Ok(())
        }
    }

    fn validate_entries_pieces(accessor: MockAccessor, piece_length: usize, num_threads: usize) {
        let (prog_send, prog_recv) = mpsc::channel();

        let total_num_pieces = ((accessor.as_slice().len() as f64) / (piece_length as f64))
            .ceil() as u64;
        let received_pieces = worker::start_hasher_workers(&accessor,
                                                           piece_length,
                                                           total_num_pieces,
                                                           num_threads,
                                                           move |update| {
                                                               prog_send.send(update).unwrap();
                                                           }).unwrap();

        let computed_pieces = accessor.as_slice()
            .chunks(piece_length)
            .enumerate()
            .map(|(index, chunk)| (index, ShaHash::from_bytes(chunk)))
            .collect::<Vec<(usize, ShaHash)>>();

        let updates_received = prog_recv.iter().count() as u64;

        assert_eq!(total_num_pieces, updates_received);
        assert_eq!(received_pieces, computed_pieces);
    }

    #[test]
    fn positive_piece_length_divisible_region_single_thread() {
        let mut accessor = MockAccessor::new();

        let region_length = DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES;
        accessor.create_region(region_length);

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 1);
    }

    #[test]
    fn positive_piece_length_divisible_region_multiple_threads() {
        let mut accessor = MockAccessor::new();

        let region_length = DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES;
        accessor.create_region(region_length);

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 4);
    }

    #[test]
    fn positive_piece_length_undivisible_region_single_thread() {
        let mut accessor = MockAccessor::new();

        let region_length = DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES + 1;
        accessor.create_region(region_length);

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 1);
    }

    #[test]
    fn positive_piece_length_undivisible_region_multiple_threads() {
        let mut accessor = MockAccessor::new();

        let region_length = DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES + 1;
        accessor.create_region(region_length);

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 4);
    }

    #[test]
    fn positive_piece_length_divisible_regions_single_thread() {
        let mut accessor = MockAccessor::new();

        let region_lengths = [DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH * 1,
                              DEFAULT_PIECE_LENGTH * 50];
        for &region_length in region_lengths.into_iter() {
            accessor.create_region(region_length);
        }

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 1);
    }

    #[test]
    fn positive_piece_length_divisible_regions_multiple_threads() {
        let mut accessor = MockAccessor::new();

        let region_lengths = [DEFAULT_PIECE_LENGTH * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH * 1,
                              DEFAULT_PIECE_LENGTH * 50];
        for &region_length in region_lengths.into_iter() {
            accessor.create_region(region_length);
        }

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 4);
    }

    #[test]
    fn positive_piece_length_undivisible_regions_single_thread() {
        let mut accessor = MockAccessor::new();

        let region_lengths = [DEFAULT_PIECE_LENGTH / 2 * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH / 4 * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH * 1,
                              (DEFAULT_PIECE_LENGTH * 2 - 1) * 2];
        for &region_length in region_lengths.into_iter() {
            accessor.create_region(region_length);
        }

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 1);
    }

    #[test]
    fn positive_piece_length_undivisible_regions_multiple_threads() {
        let mut accessor = MockAccessor::new();

        let region_lengths = [DEFAULT_PIECE_LENGTH / 2 * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH / 4 * DEFAULT_NUM_PIECES,
                              DEFAULT_PIECE_LENGTH * 1,
                              (DEFAULT_PIECE_LENGTH * 2 - 1) * 2];
        for &region_length in region_lengths.into_iter() {
            accessor.create_region(region_length);
        }

        validate_entries_pieces(accessor, DEFAULT_PIECE_LENGTH, 4);
    }
}
