use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use chrono::naive::NaiveDateTime;
use exif::{Exif, Reader as ExifReader};
use globwalk::GlobWalkerBuilder;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use path_slash::PathExt;
use rayon::prelude::*;
use rusqlite::Connection as DbConnection;
use sha1::{Digest, Sha1};

use crate::config::{self, Config};
use crate::db::{self, SyncedDb};
use crate::imaging::*;
use crate::interlude::*;
use crate::model;

pub fn scan(db: SyncedDb, config: Config) -> Result<()> {
    let date_paths = config.date_path;
    for err in config
        .markers
        .disk
        .into_par_iter()
        .enumerate()
        .filter_map(|(i, marker)| process_tree(i, marker, date_paths.clone(), db.clone()).err())
        .collect::<Vec<_>>()
    {
        ieprintln!("Error: " err);
    }

    // FIXME: Stage 2: check if all files from DB are present on disk, delete entries for any missing

    // FIXME: Stage 3: scan all files once more and refresh them in DB

    Ok(())
}

pub fn process_tree(
    i: usize,
    marker_path: impl AsRef<Path>,
    mut date_paths_per_marker: config::DatePathsPerMarker,
    db: Arc<Mutex<DbConnection>>,
) -> Result<()> {
    let marker_path = marker_path.as_ref();
    let m = marker_read(marker_path);
    if check_io_error(&m) == Some(io::ErrorKind::NotFound) {
        iprintln!("\nSkipping tree at '" marker_path.display() "': " error_chain(&m.unwrap_err()));
        return Ok(());
    }
    let (root, marker) = m?;
    iprintln!("marker " &marker " at: " root.display());

    // Match any date-path config to marker.
    let date_paths = date_paths_per_marker.remove(&marker);
    iprintln!("\nDate-paths at " marker;? ": " date_paths;?);

    // Stage 1: add not-yet-known files into DB
    // TODO[LATER]: in parallel thread, count all matching files, then when done start showing progress bar/percentage
    let images = GlobWalkerBuilder::new(&root, "*.{jpg,jpeg}")
        .case_insensitive(true)
        .file_type(globwalk::FileType::FILE)
        .build();
    for entry in images? {
        // Extract path.
        let path = match entry {
            Ok(entry) => entry.path().to_owned(),
            Err(err) => {
                ieprintln!("\nFailed to access file, skipping: " err);
                continue;
            }
        };
        // Read file contents to memory.
        let buf = fs::read(&path)?;

        // Split-out relative path from root.
        let os_relative = path.strip_prefix(&root)?;
        let relative = os_relative
            .to_slash()
            .with_context(|| format!("Failed to convert path {:?} to slash-based", os_relative))?;
        // If file already exists in DB, skip it.
        let db_readable = db.lock().unwrap();
        if db::exists(&db_readable, &marker, &relative)? {
            print!(".");
            io::stdout().flush()?;
            continue;
        }
        drop(db_readable);

        // Calculate sha1 hash of the file contents.
        // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
        let hash = format!("{:x}", Sha1::digest(&buf));

        // FIXME: if image is very small, it's probably a thumbnail already and we don't want to archive it

        // Does the JPEG have Exif block? We assume it'd be the most reliable source of metadata.
        let exif = ExifReader::new()
            .read_from_container(&mut io::Cursor::new(&buf))
            .ok();
        let date = try_deduce_date(exif.as_ref(), &relative);
        // // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
        // let orientation = exif.as_ref().and_then(|v| v.orientation()).unwrap_or(1);

        // Parse the file as an image and create thumbnail, or skip with warning if impossible.
        let img = match ImageReader::new(io::Cursor::new(&buf))
            .with_guessed_format()?
            .decode()
        {
            Ok(img) => img,
            Err(err) => {
                // TODO[LATER]: use termcolor crate to print errors in red
                // FIXME[LATER]: resolve JPEG decoding error: "spectral selection is not allowed in non-progressive scan"
                ieprintln!("\nFailed to decode JPEG " &path;? ", skipping: " err);
                continue;
            }
        };
        // let thumb = img.resize(200, 200, FilterType::Lanczos3);
        let thumb = img.resize(200, 200, FilterType::CatmullRom);
        // FIXME[LATER]: fix the thumbnail's orientation
        let mut thumb_jpeg = Vec::<u8>::new();
        thumb.write_to(&mut thumb_jpeg, image::ImageOutputFormat::Jpeg(90))?;

        // Add image entry to DB.
        let info = model::FileInfo {
            hash: hash.clone(),
            date,
            thumb: thumb_jpeg,
        };
        let db_writable = db.lock().unwrap();
        db::upsert(&db_writable, &marker, &relative, &info)?;
        drop(db_writable);

        // Print some debugging info, showing which marker is still being processed.
        iprint!(i);
        io::stdout().flush()?;
        // println!("{} {} {:?} {:?}", &hash, path.display(), date.map(|d| d.to_string()), orientation);
    }

    Ok(())
}

// TODO[LATER]: accept Path and return Result<(Path,...)> with proper lifetime
fn marker_read(file_path: &Path) -> Result<(PathBuf, String)> {
    let parent = file_path.parent().ok_or_else(|| {
        anyhow!(
            "Could not split parent directory of '{}'",
            file_path.display()
        )
    })?;

    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Marker {
        id: String,
    }
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open '{}'", file_path.display()))?;
    let m: Marker = serde_json::from_reader(io::BufReader::new(file))?;

    Ok((parent.to_owned(), m.id))
}

fn check_io_error<T>(result: &Result<T>) -> Option<io::ErrorKind> {
    result
        .as_ref()
        .err()
        .and_then(|err| err.downcast_ref::<io::Error>())
        .map(|cause| cause.kind())
}

/// Try hard to find out some datetime info from either `exif` data, or `relative_path` of the file.
fn try_deduce_date(exif: Option<&Exif>, relative_path: &str) -> Option<NaiveDateTime> {
    if let Some(exif) = exif {
        use exif::Tag;
        // TODO[LATER]: are ther other fields we could try?
        if let Some(d) = vec![Tag::DateTime, Tag::DateTimeOriginal]
            .into_iter()
            .filter_map(|tag| exif.datetime(tag))
            .filter_map(|dt| dt.to_naive_opt())
            .next()
        {
            return Some(d);
        }
    }
    // TODO[LATER]: try extracting date from relative_path
    // TODO[LATER]: try extracting date from file's creation and modification date (NOTE: latter can be earlier than former on Windows!)
    None
}
