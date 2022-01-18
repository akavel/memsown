//! Filesystem tree walking & path matching, wrapped in a simplified API with only the
//! functionalities currently seeming to be useful for the project.

use std::ffi::{OsStr, OsString};

pub trait DirEntry {
    /// Extension of the file/directory's name, if present.
    fn extension(&self) -> Option<&OsStr>;
}

pub trait Matcher {
    fn matches(&self, entry: &dyn DirEntry) -> bool;
}

pub struct CaseInsensitiveExtensions(Vec<OsString>);

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
