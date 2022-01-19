//! Filesystem tree walking & path matching, wrapped in a simplified API with only the
//! functionalities currently seeming to be useful for the project.

pub mod matcher {
    use std::ffi::{OsStr, OsString};

    pub trait DirEntry {
        /// Extension of the file/directory's name, if present.
        fn extension(&self) -> Option<&OsStr>;
    }

    pub trait Matcher {
        fn matches(&self, entry: &dyn DirEntry) -> bool;
    }

    pub struct CaseInsensitiveExtensions(Vec<OsString>);

    impl CaseInsensitiveExtensions {
        pub fn boxed(extensions: impl IntoIterator<Item = &'static str>) -> Box<dyn Matcher> {
            Box::new(Self(Vec::from_iter(
                extensions.into_iter().map(|s| s.into()),
            )))
        }
    }

    impl Matcher for CaseInsensitiveExtensions {
        fn matches(&self, entry: &dyn DirEntry) -> bool {
            let ext = if let Some(ext) = entry.extension() {
                ext // TODO[LATER]: use `let-else` syntax
            } else {
                return false;
            };
            for candidate in &self.0 {
                if ext.eq_ignore_ascii_case(candidate) {
                    return true;
                }
            }
            false
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        struct MockEntry(Option<OsString>);

        impl DirEntry for MockEntry {
            fn extension(&self) -> Option<&OsStr> {
                self.0.as_deref()
            }
        }

        #[test]
        fn case_insensitive_jpeg_extensions() {
            let jpegs = CaseInsensitiveExtensions(vec!["jpg".into(), "jpeg".into()]);

            // Positive
            assert!(jpegs.matches(&MockEntry(Some("jpg".into()))));
            assert!(jpegs.matches(&MockEntry(Some("jpeg".into()))));
            assert!(jpegs.matches(&MockEntry(Some("JPG".into()))));
            assert!(jpegs.matches(&MockEntry(Some("JPEG".into()))));

            // Negative
            assert!(!jpegs.matches(&MockEntry(None)));
            assert!(!jpegs.matches(&MockEntry(Some("png".into()))));
        }
    }
}

pub mod walker {
    use std::path::{Path, PathBuf};

    use anyhow::{anyhow, Result};
    use walkdir::WalkDir;

    use super::matcher as m;

    /// Note: for now implicitly assumed to be a file, not a dir.
    /// (Behavior for symlinks is currently unspecified.)
    // TODO[LATER]: try making this thin wrapper around `&Path`
    #[derive(Debug)]
    pub struct DirEntry {
        relative_path: PathBuf,
    }

    impl DirEntry {
        fn relative_path(&self) -> &Path {
            self.relative_path.as_ref()
        }
    }

    pub struct Files {
        root: PathBuf,
        matchers: Vec<Box<dyn m::Matcher>>,
    }

    impl Files {
        // TODO[LATER]: can we avoid Box in arg somehow?
        pub fn new(
            root: impl AsRef<Path>,
            matchers: impl IntoIterator<Item = Box<dyn m::Matcher>>,
        ) -> Self {
            Self {
                root: root.as_ref().into(),
                matchers: Vec::from_iter(matchers),
            }
        }

        // TODO[LATER]: also impl IntoIterator (move semantics, for loops)
        // TODO[LATER]: try moving Result to `new` func instead
        // TODO[LATER]: custom Result type instead of anyhow
        pub fn iter(&self) -> FilesIterator {
            FilesIterator {
                root: self.root.clone(),
                iter: WalkDir::new(&self.root).into_iter(),
            }
        }
    }

    // impl IntoIterator for Files {
    //     type Item = DirEntry;
    //     type IntoIter = FilesIterator;
    //     fn into_iter(self)
    // }

    pub struct FilesIterator {
        // TODO[LATER]: make `root` a `&Path`
        root: PathBuf,
        iter: walkdir::IntoIter,
    }

    impl Iterator for FilesIterator {
        type Item = Result<DirEntry>;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                // Basic iterator pass-through of errors & iteration end.
                let entry = match self.iter.next() {
                    None => return None,
                    Some(Err(err)) => return Some(Err(anyhow!(err))),
                    Some(Ok(entry)) => entry,
                };
                // Don't emit directory entries; emit error for symlinks.
                let kind = entry.file_type();
                if kind.is_dir() {
                    continue;
                } else if kind.is_symlink() {
                    return Some(Err(anyhow!("Don't know what to do with a symbolic link: {:?}", entry.path())));
                }
                // Extract relative path.
                // Note: walkdir pinky-promises that paths will have the prefix, so we should be
                // safe to just `unwrap()`; but we already have Result return type, so we can just
                // squeeze another error there instead, just in case.
                let relative_path = match entry.path().strip_prefix(&self.root) {
                    Err(err) => return Some(Err(anyhow!("Failed to split relative path: {}", err))),
                    Ok(path) => path,
                };
                return Some(Ok(DirEntry{ relative_path: relative_path.into() }))
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn constructor() {
            let _ = Files::new(".", [m::CaseInsensitiveExtensions::boxed(["jpg", "jpeg"])]);
        }
    }
}
