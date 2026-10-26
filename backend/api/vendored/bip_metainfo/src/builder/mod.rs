use std::iter::ExactSizeIterator;

use bip_bencode::{BencodeMut, BMutAccess, BRefAccess};
use bip_util::sha::{self, ShaHash};

use accessor::{Accessor, IntoAccessor};
use error::ParseResult;
use parse;

mod buffer;
mod queue;
mod worker;

// Piece length is inversly related to the file size.
// Transfer reliability is inversly related to the piece length.
// Transfer reliability is directly related to the file size.

// These statements hold even today, although the piece lengths that were historically
// recommended may be out of date as we get faster and more reliable network speeds.

// So for balanced, file size, and transfer piece length optimizations, calculate the
// minimum piece length we can do to reach the designated pieces size. Then, if that
// piece length is less than the minimum piece length for that optimization, set it equal
// to the minimum. Setting it equal to the minimum (in that case) will increase the piece
// size which will shrink the pieces size which ensures we do not go outside of our max size.
// This ensure we can generate good piece lengths for both large and small files.

// Maximum Piece Length Across The Board, Takes Priority Over Max Pieces Sizes
// (Not Applied To Custom Lengths)
const ALL_OPT_MAX_PIECE_LENGTH: usize = 16 * 1024 * 1024;

const BALANCED_MAX_PIECES_SIZE: usize = 40000;
const BALANCED_MIN_PIECE_LENGTH: usize = 512 * 1024;

const FILE_SIZE_MAX_PIECES_SIZE: usize = 20000;
const FILE_SIZE_MIN_PIECE_LENGTH: usize = 1 * 1024 * 1024;

const TRANSFER_MAX_PIECES_SIZE: usize = 60000;
const TRANSFER_MIN_PIECE_LENGTH: usize = 1 * 1024;

/// Enumerates settings for piece length for generating a torrent file.
pub enum PieceLength {
    /// Optimize piece length for torrent file size and file transfer.
    OptBalanced,
    /// Optimize piece length for torrent file size.
    OptFileSize,
    /// Optimize piece length for torrent file transfer.
    OptTransfer,
    /// Custom piece length.
    Custom(usize),
}

/// Builder for generating a torrent file from some accessor.
pub struct MetainfoBuilder<'a> {
    root: BencodeMut<'a>,
    info: InfoBuilder<'a>
}

impl<'a> MetainfoBuilder<'a> {
    /// Create a new MetainfoBuilder with some default values set.
    pub fn new() -> MetainfoBuilder<'a> {
        MetainfoBuilder {
            root: BencodeMut::new_dict(),
            info: InfoBuilder::new()
        }
    }

    /// Set announce-list content
    pub fn set_trackers(mut self, opt_trackers: Option<&'a Vec<Vec<String>>>) -> MetainfoBuilder<'a> {
        {
            let dict_access = self.root.dict_mut().unwrap();

            if let Some(groups) = opt_trackers {
                let mut list = BencodeMut::new_list();

                {
                    let list_access = list.list_mut().unwrap();

                    for group in groups.iter() {
                        let mut tracker_list = BencodeMut::new_list();

                        {
                            let tracker_list_access = tracker_list.list_mut().unwrap();

                            for tracker_url in group.iter() {
                                tracker_list_access.push(ben_bytes!(&tracker_url[..]));
                            }
                        }

                        list_access.push(tracker_list);
                    }
                }

                dict_access.insert(parse::ANNOUNCE_LIST_KEY.into(), list);
            } else {
                dict_access.remove(parse::ANNOUNCE_LIST_KEY);
            }
        }

        self
    }

    /// Set or unset the main tracker that this torrent file points to.
    pub fn set_main_tracker(mut self, opt_tracker_url: Option<&'a str>) -> MetainfoBuilder<'a> {
        {
            let dict_access = self.root.dict_mut().unwrap();

            if let Some(tracker_url) = opt_tracker_url {
                dict_access.insert(parse::ANNOUNCE_URL_KEY.into(), ben_bytes!(tracker_url));
            } else {
                dict_access.remove(parse::ANNOUNCE_URL_KEY);
            }
        }

        self
    }

    /// Set or unset the creation date for the torrent.
    pub fn set_creation_date(mut self, opt_secs_epoch: Option<i64>) -> MetainfoBuilder<'a> {
        {
            let dict_access = self.root.dict_mut().unwrap();

            if let Some(secs_epoch) = opt_secs_epoch {
                dict_access.insert(parse::CREATION_DATE_KEY.into(), ben_int!(secs_epoch));
            } else {
                dict_access.remove(parse::CREATION_DATE_KEY);
            }
        }

        self
    }

    /// Set or unset a comment for the torrent file.
    pub fn set_comment(mut self, opt_comment: Option<&'a str>) -> MetainfoBuilder<'a> {
        {
            let dict_access = self.root.dict_mut().unwrap();

            if let Some(comment) = opt_comment {
                dict_access.insert(parse::COMMENT_KEY.into(), ben_bytes!(comment));
            } else {
                dict_access.remove(parse::COMMENT_KEY);
            }
        }

        self
    }

    /// Set or unset the created by for the torrent file.
    pub fn set_created_by(mut self, opt_created_by: Option<&'a str>) -> MetainfoBuilder<'a> {
        {
            let dict_access = self.root.dict_mut().unwrap();

            if let Some(created_by) = opt_created_by {
                dict_access.insert(parse::CREATED_BY_KEY.into(), ben_bytes!(created_by));
            } else {
                dict_access.remove(parse::CREATED_BY_KEY);
            }
        }

        self
    }

    /// Set or unset the private flag for the torrent file.
    pub fn set_private_flag(mut self, opt_is_private: Option<bool>) -> MetainfoBuilder<'a> {
        self.info = self.info.set_private_flag(opt_is_private);

        self
    }

    /// Set or unset the source tag for the torrent file (within the info dictionary).
    pub fn set_source(mut self, opt_source: Option<&'a str>) -> MetainfoBuilder<'a> {
        self.info = self.info.set_source(opt_source);

        self
    }

    /// Sets the piece length for the torrent file.
    pub fn set_piece_length(mut self, piece_length: PieceLength) -> MetainfoBuilder<'a> {
        self.info = self.info.set_piece_length(piece_length);

        self
    }

    /// Get decoded value of announce-list key
    pub fn get_trackers(&self) -> Option<Vec<Vec<String>>> {
        let dict_access = self.root.dict().unwrap();

        parse::parse_announce_list(dict_access).map(parse::convert_announce_list)
    }

    /// Get decoded value of announce-url key
    pub fn get_main_tracker(&self) -> Option<String> {
        let dict_access = self.root.dict().unwrap();

        parse::parse_announce_url(dict_access).map(String::from)
    }

    /// Get decoded value of creation-date key
    pub fn get_creation_date(&self) -> Option<i64> {
        let dict_access = self.root.dict().unwrap();

        parse::parse_creation_date(dict_access)
    }

    /// Get decoded value of comment key
    pub fn get_comment(&self) -> Option<String> {
        let dict_access = self.root.dict().unwrap();

        parse::parse_comment(dict_access).map(String::from)
    }

    /// Get decoded value of created-by key
    pub fn get_created_by(&self) -> Option<String> {
        let dict_access = self.root.dict().unwrap();

        parse::parse_created_by(dict_access).map(String::from)
    }

    /// Build the metainfo file from the given accessor and the number of worker threads.
    ///
    /// Panics if threads is equal to zero.
    pub fn build<A, C>(self, threads: usize, accessor: A, progress: C) -> ParseResult<Vec<u8>>
        where A: IntoAccessor,
              C: FnMut(f64) + Send + 'static
    {
        let accessor = try!(accessor.into_accessor());

        build_with_accessor(threads, accessor, progress, Some(self.root), self.info.info, self.info.piece_length)
    }
}

// ----------------------------------------------------------------------------//

/// Builder for generating an info dictionary file from some accessor.
pub struct InfoBuilder<'a> {
    info:         BencodeMut<'a>,
    // Stored outside of root as some of the variants need the total
    // file sizes in order for the final piece length to be calculated.
    piece_length: PieceLength
}

impl<'a> InfoBuilder<'a> {
    pub fn new() -> InfoBuilder<'a> {
        InfoBuilder{ info: BencodeMut::new_dict(), piece_length: PieceLength::OptBalanced }
    }

    /// Set or unset the private flag for the torrent file.
    pub fn set_private_flag(mut self, opt_is_private: Option<bool>) -> InfoBuilder<'a> {
        {
            let dict_access = self.info.dict_mut().unwrap();

            if let Some(private) = opt_is_private {
                dict_access.insert(parse::PRIVATE_KEY.into(), ben_int!(if private { 1 } else { 0 }));
            } else {
                dict_access.remove(parse::PRIVATE_KEY);
            }
        }

        self
    }

    /// Set or unset the source tag for the torrent file.
    pub fn set_source(mut self, opt_source: Option<&'a str>) -> InfoBuilder<'a> {
        {
            let dict_access = self.info.dict_mut().unwrap();

            if let Some(source) = opt_source {
                dict_access.insert(parse::SOURCE_KEY.into(), ben_bytes!(source));
            } else {
                dict_access.remove(parse::SOURCE_KEY);
            }
        }

        self
    }

    /// Sets the piece length for the torrent file.
    pub fn set_piece_length(mut self, piece_length: PieceLength) -> InfoBuilder<'a> {
        self.piece_length = piece_length;

        self
    }

    /// Build the metainfo file from the given accessor and the number of worker threads.
    ///
    /// Panics if threads is equal to zero.
    pub fn build<A, C>(self, threads: usize, accessor: A, progress: C) -> ParseResult<Vec<u8>>
        where A: IntoAccessor,
              C: FnMut(f64) + Send + 'static
    {
        let accessor = try!(accessor.into_accessor());

        build_with_accessor(threads, accessor, progress, None, self.info, self.piece_length)
    }
}

// ----------------------------------------------------------------------------//

fn build_with_accessor<'a, A, C>(threads:       usize,
                                accessor:       A,
                                progress:       C,
                                opt_root:       Option<BencodeMut<'a>>,
                                info:           BencodeMut<'a>,
                                piece_length:   PieceLength) -> ParseResult<Vec<u8>>
    where A: Accessor,
          C: FnMut(f64) + Send + 'static {
        if threads == 0 {
            panic!("bip_metainfo: Cannot Build Metainfo File With threads == 0");
        }

        // Collect all of the file information into a list
        let mut files_info = Vec::new();
        try!(accessor.access_metadata(|len, path| {
            let path_list: Vec<String> = path.iter()
                .map(|os_str| os_str.to_string_lossy().into_owned())
                .collect();

            files_info.push((len, path_list));
        }));

        // Build the pieces for the data our accessor is pointing at
        let total_files_len = files_info.iter().fold(0, |acc, nex| acc + nex.0);
        let piece_length = determine_piece_length(total_files_len, piece_length);
        let total_num_pieces = ((total_files_len as f64) / (piece_length as f64)).ceil() as u64;
        let pieces_list = try!(worker::start_hasher_workers(&accessor,
                                                            piece_length,
                                                            total_num_pieces,
                                                            threads,
                                                            progress));
        let pieces = map_pieces_list(pieces_list.into_iter().map(|(_, piece)| piece));

        let mut single_file_name = String::new();
        let access_directory = accessor.access_directory().map(|path| path.to_string_lossy());

        // Move these below access directory for borrow checker
        let opt_root = opt_root;
        let mut info = info;

        // Update the info bencode with values
        {
            let info_access = info.dict_mut().unwrap();

            info_access.insert(parse::PIECE_LENGTH_KEY.into(), ben_int!(piece_length as i64));
            info_access.insert(parse::PIECES_KEY.into(), ben_bytes!(&pieces[..]));

            // If the accessor specifies a directory OR there are mutliple files, we will build a multi file torrent
            // If the directory is not present but there are multiple files, the direcotry field will be set to empty
            match (&access_directory, files_info.len() > 1) {
                (&Some(ref directory), _) => {
                    let mut bencode_files = BencodeMut::new_list();

                    {
                        let bencode_files_access = bencode_files.list_mut().unwrap();

                        // Multi File
                        for &(len, ref path) in files_info.iter() {
                            let mut bencode_path = BencodeMut::new_list();

                            {
                                let bencode_path_access = bencode_path.list_mut().unwrap();

                                for path_element in path.iter() {
                                    bencode_path_access.push(ben_bytes!(&path_element[..]));
                                }
                            }

                            bencode_files_access.push(ben_map!{
                                parse::LENGTH_KEY => ben_int!(len as i64),
                                parse::PATH_KEY   => bencode_path
                            });
                        }
                    }

                    info_access.insert(parse::NAME_KEY.into(), ben_bytes!(directory.as_ref()));
                    info_access.insert(parse::FILES_KEY.into(), bencode_files);
                }
                (&None, true) => {
                    let mut bencode_files = BencodeMut::new_list();

                    {
                        let bencode_files_access = bencode_files.list_mut().unwrap();

                        // Multi File
                        for &(len, ref path) in files_info.iter() {
                            let mut bencode_path = BencodeMut::new_list();

                            {
                                let bencode_path_access = bencode_path.list_mut().unwrap();

                                for path_element in path.iter() {
                                    bencode_path_access.push(ben_bytes!(&path_element[..]));
                                }
                            }

                            bencode_files_access.push(ben_map!{
                                parse::LENGTH_KEY => ben_int!(len as i64),
                                parse::PATH_KEY   => bencode_path
                            });
                        }
                    }

                    info_access.insert(parse::NAME_KEY.into(), ben_bytes!(""));
                    info_access.insert(parse::FILES_KEY.into(), bencode_files);
                }
                (&None, false) => {
                    // Single File
                    for name_component in files_info[0].1.iter() {
                        single_file_name.push_str(name_component);
                    }

                    info_access.insert(parse::LENGTH_KEY.into(), ben_int!(files_info[0].0 as i64));
                    info_access.insert(parse::NAME_KEY.into(), ben_bytes!(&single_file_name[..]));
                }
            }
        }

        if let Some(mut root) = opt_root {
            root.dict_mut().unwrap().insert(parse::INFO_KEY.into(), info);

            Ok(root.encode())
        } else {
            Ok(info.encode())
        }
}

/// Calculate the final piece length given the total file size and piece length strategy.
///
/// Lower piece length will result in a bigger file but better transfer reliability and vice versa.
fn determine_piece_length(total_file_size: u64, piece_length: PieceLength) -> usize {
    match piece_length {
        PieceLength::Custom(len) => len,
        PieceLength::OptBalanced => {
            calculate_piece_length(total_file_size,
                                   BALANCED_MAX_PIECES_SIZE,
                                   BALANCED_MIN_PIECE_LENGTH)
        }
        PieceLength::OptFileSize => {
            calculate_piece_length(total_file_size,
                                   FILE_SIZE_MAX_PIECES_SIZE,
                                   FILE_SIZE_MIN_PIECE_LENGTH)
        }
        PieceLength::OptTransfer => {
            calculate_piece_length(total_file_size,
                                   TRANSFER_MAX_PIECES_SIZE,
                                   TRANSFER_MIN_PIECE_LENGTH)
        }
    }
}

/// Calculate the minimum power of 2 piece length for the given max pieces size and total file size.
fn calculate_piece_length(total_file_size: u64,
                          max_pieces_size: usize,
                          min_piece_length: usize)
                          -> usize {
    let num_pieces = (max_pieces_size as f64) / (sha::SHA_HASH_LEN as f64);
    let piece_length = ((total_file_size as f64) / num_pieces + 0.5) as usize;

    let pot_piece_length = piece_length.next_power_of_two();

    match (pot_piece_length > min_piece_length, pot_piece_length < ALL_OPT_MAX_PIECE_LENGTH) {
        (true, true) => pot_piece_length,
        (false, _) => min_piece_length,
        (_, false) => ALL_OPT_MAX_PIECE_LENGTH,
    }
}
/// Map the pieces list into a list of bytes (byte string).
fn map_pieces_list<I>(pieces: I) -> Vec<u8>
    where I: Iterator<Item = ShaHash> + ExactSizeIterator
{
    let mut concated_pieces = Vec::with_capacity(pieces.len() * sha::SHA_HASH_LEN);
    for piece in pieces {
        concated_pieces.extend_from_slice(piece.as_ref());
    }

    concated_pieces
}
