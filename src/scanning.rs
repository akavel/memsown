use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use exif::{Exif, Reader as ExifReader};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use path_slash::{PathBufExt, PathExt};
use rayon::prelude::*;
use rusqlite::Connection as DbConnection;
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::config::{self, Config, DatePath};
use crate::db::{self, SyncedDb};
use crate::imaging::*;
use crate::interlude::*;
use crate::model;
use crate::pathwalk::{matcher, walker};

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

    Ok(())
}

pub fn process_tree(
    i: usize,
    marker_path: impl AsRef<Path>,
    date_paths_per_marker: config::DatePathsPerMarker,
    db: Arc<Mutex<DbConnection>>,
) -> Result<()> {
    let m = Tree::open(marker_path, &date_paths_per_marker);
    if let Err(TreeError::NotFound { .. }) = &m {
        iprintln!("\nSkipping tree: " error_chain(&m.unwrap_err().into()));
        return Ok(());
    }
    let tree: Tree = m?;
    iprintln!("marker " &tree.marker " at: " tree.root;?);

    // Match any date-path config to marker.
    iprintln!("\nDate-paths at " tree.marker;? ": " tree.date_paths;?);

    // Stage 1: add not-yet-known files into DB
    stage1(i, &tree, &db, OnExisting::Skip)?;

    // Stage 2: check if all files from DB are present on disk, delete entries for any missing
    stage2(&tree, &db)?;

    // Stage 3: scan all files once more and refresh them in DB
    stage1(i, &tree, &db, OnExisting::Refresh)?;

    Ok(())
}

#[derive(PartialEq)]
enum OnExisting {
    Skip,
    Refresh,
}

fn stage1(
    i: usize,
    tree: &Tree,
    db: &Arc<Mutex<DbConnection>>,
    on_existing: OnExisting,
) -> Result<()> {
    // TODO[LATER]: in parallel thread, count all matching files, then when done start showing progress bar/percentage
    for entry in tree.iter() {
        let entry = match entry {
            // TODO[LATER]: use `let else` once stable
            Ok(entry) => entry,
            Err(err) => {
                ieprintln!("\nFailed to access file, skipping: " err);
                continue;
            }
        };
        let os_relative = entry.relative_path();
        let path = tree.root.join(os_relative);
        let relative = os_relative
            .to_slash()
            .with_context(|| ifmt!("Failed to convert path " os_relative;? " to slash-based"))?;

        // Read file contents to memory.
        let buf = fs::read(&path)?;

        // If file already exists in DB, skip it.
        let db_readable = db.lock().unwrap();
        if db::exists(&db_readable, &tree.marker, &relative)? && on_existing == OnExisting::Skip {
            print!(".");
            io::stdout().flush()?;
            continue;
        }
        drop(db_readable);

        // Calculate sha1 hash of the file contents.
        // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
        let hash = hash(&buf);
        // let hash = format!("{:x}", Sha1::digest(&buf));

        // FIXME: if image is very small, it's probably a thumbnail already and we don't want to archive it

        // Does the JPEG have Exif block? We assume it'd be the most reliable source of metadata.
        let exif = ExifReader::new()
            .read_from_container(&mut io::Cursor::new(&buf))
            .ok();
        let date = try_deduce_date(exif.as_ref(), &relative, tree.date_paths.iter());
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
        db::upsert(&db_writable, &tree.marker, &relative, &info)?;
        drop(db_writable);

        // Print some debugging info, showing which marker is still being processed.
        iprint!(i);
        io::stdout().flush()?;
        // println!("{} {} {:?} {:?}", &hash, path.display(), date.map(|d| d.to_string()), orientation);
    }

    Ok(())
}

pub fn stage2(tree: &Tree, db: &Arc<Mutex<DbConnection>>) -> Result<()> {
    for item in db::hashes(db.clone(), &tree.marker) {
        let (relative_path, db_hash) = item?;

        let path = tree.root.join(PathBuf::from_slash(&relative_path));

        // Try reading file contents to memory.
        let contents = match fs::read(&path) {
            Ok(data) => Some(data),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(anyhow!(err)),
        };
        if let Some(data) = contents {
            let disk_hash = hash(&data);

            if disk_hash == db_hash {
                print!(",");
                io::stdout().flush()?;
            } else {
                iprintln!("\nBAD HASH: " disk_hash " != " db_hash " @ " path;?);
                // iprintln!("* " db_hash " @ " path;?);
            }
        } else {
            let db = db.lock().unwrap();
            // TODO[LATER]: add error context info
            db::remove(&db, &tree.marker, &relative_path)?;
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Tree {
    pub marker: String,
    pub root: PathBuf,
    pub date_paths: Vec<DatePath>,
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("marker file not found at: '{0}'")]
    NotFound(PathBuf),
    #[error("error reading marker file at {path:?}")]
    Other {
        path: PathBuf,
        source: anyhow::Error,
    },
}

impl Tree {
    pub fn open(
        marker_path: impl AsRef<Path>,
        date_paths_per_marker: &config::DatePathsPerMarker,
    ) -> Result<Tree, TreeError> {
        let (root, marker) = match marker_read(marker_path.as_ref()) {
            Err(err)
                if err.downcast_ref().map(io::Error::kind) == Some(io::ErrorKind::NotFound) =>
            {
                return Err(TreeError::NotFound(marker_path.as_ref().to_owned()))
            }
            Err(err) => {
                return Err(TreeError::Other {
                    path: marker_path.as_ref().to_owned(),
                    source: err,
                })
            }
            Ok(tree) => tree,
        };
        let date_paths = date_paths_per_marker.get(&marker);
        let date_paths = date_paths.map(|v| v.to_owned()).unwrap_or(Vec::new());
        Ok(Tree {
            marker,
            root,
            date_paths,
        })
    }

    pub fn iter(&self) -> walker::FilesIterator {
        let jpeg_matcher = matcher::CaseInsensitiveExtensions::boxed(["jpg", "jpeg"]);
        walker::Files::new(&self.root, [jpeg_matcher]).into_iter()
    }
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

/// Calculate a hash of the buf contents, and return it in a pretty-printed format for storing in
/// the DB.
pub fn hash(buf: &[u8]) -> String {
    // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
    format!("{:x}", Sha1::digest(&buf))
}

/// Try hard to find out some datetime info from either `exif` data, or `relative_path` of the file.
fn try_deduce_date<'a>(
    exif: Option<&Exif>,
    relative_path: &str,
    date_paths: impl Iterator<Item = &'a config::DatePath>,
) -> Option<NaiveDateTime> {
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
    // try extracting date from relative_path
    // TODO: helper binary for checking which paths would decode to what dates
    // TODO[LATER]: add option/button to pre-check date-path patterns on real files tree in GUI
    for date_path in date_paths {
        if let Some(found) = date_path.path.captures(&relative_path) {
            let mut buf = String::new();
            found.expand(&date_path.date, &mut buf);
            const YMD_HMS: &str = "%Y-%m-%d %H:%M:%S";
            const YMD: &str = "%Y-%m-%d";
            let date = NaiveDateTime::parse_from_str(&buf, YMD_HMS)
                .or_else(|_| NaiveDate::parse_from_str(&buf, YMD).map(|d| d.and_hms(0, 0, 0)));
            if let Ok(d) = date {
                return Some(d);
            }
        }
    }
    // TODO[LATER]: try extracting date from file's creation and modification date (NOTE: latter can be earlier than former on Windows!)
    None
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::db;

    use super::*;

    #[test]
    fn stage2_file_not_found() {
        // arrange

        let marker: &str = "foo-marker";
        let relative_path: &str = "foo-dir/foo-file.jpeg";
        let root = tempdir().unwrap();
        let mut marker_file = fs::File::create(root.path().join("marker.json")).unwrap();
        marker_file
            .write_all(r#"{"id": "foo-marker"}"#.as_bytes())
            .unwrap();
        // writeln!(marker_file, r#"{"id": "foo-marker"}"#).unwrap();
        drop(marker_file);
        let tree = Tree::open(
            root.path().join("marker.json"),
            &config::DatePathsPerMarker::new(),
        )
        .unwrap();

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let db = Arc::new(Mutex::new(conn));
        let conn = db.lock().unwrap();
        db::init(&conn).unwrap();
        db::upsert(
            &conn,
            &marker,
            &relative_path,
            &crate::model::FileInfo {
                hash: hash(&Vec::new()),
                date: None,
                thumb: Vec::new(),
            },
        )
        .unwrap();
        assert_eq!(db::exists(&conn, &marker, &relative_path), Ok(true));
        drop(conn);

        // act

        let res = stage2(&tree, &db);

        // assert

        assert!(res.is_ok(), "stage2 == {:?}", &res);

        let conn = db.lock().unwrap();
        assert_eq!(db::exists(&conn, &marker, &relative_path), Ok(false));
        drop(conn);
    }
}
