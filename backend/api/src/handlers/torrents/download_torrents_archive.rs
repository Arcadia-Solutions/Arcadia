use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use actix_web::{
    HttpResponse,
    http::header::{Charset, ContentDisposition, DispositionParam, DispositionType, ExtendedValue},
    web::{Bytes, Data},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;
use async_zip::{Compression, ZipEntryBuilder, base::write::ZipFileWriter};
use chrono::Utc;
use tokio::io::AsyncReadExt;

use crate::Arcadia;

/// Streams a zip archive of the given torrents straight to the client, building each `.torrent`
/// file on the fly and never materializing the whole archive in memory or on disk.
///
/// Each torrent is built with the downloading user's passkey and counts as a grab, exactly like
/// downloading it one by one. The passkey is resolved once and reused across the whole archive. A
/// torrent whose file is invalid is skipped so a single bad torrent does not abort the whole
/// archive; any other failure (a failed transaction, a database outage) aborts the stream instead
/// of silently delivering an incomplete archive.
///
/// The archive always ends with an `export-metadata.txt` entry reporting how many torrents were
/// exported, how many were skipped (with their ids) and when the export ran, so the user can tell
/// from the extracted folder whether anything is missing.
///
/// This is source-agnostic: the caller resolves which torrent ids go in (uploaded, snatched, and
/// later an artist's or a collage's torrents), and this builds and streams the archive.
pub async fn stream_torrents_archive<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user_id: i32,
    torrent_ids: Vec<i32>,
    archive_file_name: String,
) -> Result<HttpResponse> {
    // A bounded in-flight buffer connects the producer task to the response body: the producer
    // blocks once the buffer fills up, so memory stays bounded regardless of the archive size.
    let (duplex_writer, mut duplex_reader) = tokio::io::duplex(64 * 1024);

    // The passkey is identical for every torrent in the archive, so resolve the user once here
    // instead of once per torrent inside the producer loop.
    let passkey = arc.pool.find_user_with_id(user_id).await?.passkey;

    // The producer streams the body after the 200 status and headers are already sent, so a
    // mid-stream failure can no longer change the status code. Instead it flags the failure here
    // and the body stream turns the flag into an error, breaking the transfer so the client sees a
    // failed download rather than silently saving a truncated, invalid zip.
    let producer_failed = Arc::new(AtomicBool::new(false));
    let producer_failed_for_task = producer_failed.clone();

    // Only owned, `Send` values are moved into the producer task (not the `!Send` `Data`), so it
    // can be spawned on the shared runtime and keeps streaming while the client reads the body.
    let pool = arc.pool.clone();
    let tracker_name = arc.tracker.name.clone();
    let frontend_url = arc.api.frontend_url.clone();
    let tracker_url = arc.tracker.url.clone();
    let torrent_source_tag = arc.tracker.torrent_source_tag.clone();

    tokio::spawn(async move {
        let mut zip_writer = ZipFileWriter::with_tokio(duplex_writer);
        let mut exported_torrent_count = 0usize;
        let mut failed_torrent_ids: Vec<i32> = Vec::new();

        for torrent_id in torrent_ids {
            let torrent = match pool
                .get_torrent_with_passkey(
                    user_id,
                    &passkey,
                    torrent_id,
                    &tracker_name,
                    frontend_url.as_ref(),
                    tracker_url.as_ref(),
                    torrent_source_tag.as_deref(),
                )
                .await
            {
                Ok(torrent) => torrent,
                // Only an unusable torrent file is skipped: a single bad torrent must not abort
                // the whole archive.
                Err(error @ Error::TorrentFileInvalid) => {
                    tracing::warn!(
                        torrent_id,
                        %error,
                        "skipping a torrent that could not be built for the archive"
                    );
                    failed_torrent_ids.push(torrent_id);
                    continue;
                }
                // Anything else (a failed transaction, a database outage, ...) would likely affect
                // every remaining torrent too, so abort.
                Err(error) => {
                    tracing::error!(
                        torrent_id,
                        %error,
                        "aborting the torrents archive: failed to build a torrent"
                    );
                    producer_failed_for_task.store(true, Ordering::SeqCst);
                    return;
                }
            };

            let file_name = torrent_file_name(torrent_id, &torrent.title, &tracker_name);
            let entry = ZipEntryBuilder::new(file_name.into(), Compression::Stored);
            if let Err(error) = zip_writer
                .write_entry_whole(entry, &torrent.file_contents)
                .await
            {
                tracing::error!(%error, "failed to write a torrent into the archive");
                producer_failed_for_task.store(true, Ordering::SeqCst);
                return;
            }
            exported_torrent_count += 1;
        }

        let mut export_metadata_lines = vec![
            format!("torrents exported successfully: {exported_torrent_count}"),
            format!(
                "torrents failed to be exported: {}",
                failed_torrent_ids.len()
            ),
        ];
        if !failed_torrent_ids.is_empty() {
            let failed_torrent_ids_list = failed_torrent_ids
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            export_metadata_lines.push(format!("failed torrent ids: {failed_torrent_ids_list}"));
        }
        export_metadata_lines.push(format!(
            "date: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        let export_metadata = export_metadata_lines.join("\n") + "\n";

        let metadata_entry = ZipEntryBuilder::new(
            String::from("export-metadata.txt").into(),
            Compression::Stored,
        );
        if let Err(error) = zip_writer
            .write_entry_whole(metadata_entry, export_metadata.as_bytes())
            .await
        {
            tracing::error!(%error, "failed to write the export metadata into the archive");
            producer_failed_for_task.store(true, Ordering::SeqCst);
            return;
        }

        if let Err(error) = zip_writer.close().await {
            tracing::error!(%error, "failed to finalize the torrents archive");
            producer_failed_for_task.store(true, Ordering::SeqCst);
        }
    });

    let body = async_stream::stream! {
        let mut buffer = vec![0u8; 16 * 1024];
        loop {
            match duplex_reader.read(&mut buffer).await {
                Ok(0) => {
                    // The producer always sets the flag before dropping the writer, so by the time
                    // the reader observes EOF the outcome is known.
                    if producer_failed.load(Ordering::SeqCst) {
                        yield Err(std::io::Error::other(
                            "failed to build the torrents archive",
                        ));
                    }
                    break;
                }
                Ok(bytes_read) => {
                    yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&buffer[..bytes_read]));
                }
                Err(error) => {
                    yield Err(error);
                    break;
                }
            }
        }
    };

    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
            charset: Charset::Ext(String::from("UTF-8")),
            language_tag: None,
            value: archive_file_name.into_bytes(),
        })],
    };

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header(content_disposition)
        .streaming(body))
}

/// Builds a `[<site_name>] <title> (<torrent_id>).torrent` entry name,
/// The torrent id is globally unique, so the name can never collide with
/// another entry. Path separators are stripped so no entry can escape the archive root.
fn torrent_file_name(torrent_id: i32, title: &str, site_name: &str) -> String {
    let sanitized_site_name = site_name.replace(['/', '\\'], "_");
    let sanitized_title = title.replace(['/', '\\'], "_");
    format!("[{sanitized_site_name}] {sanitized_title} ({torrent_id}).torrent")
}
