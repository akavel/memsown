use chrono::naive::{NaiveDate, NaiveDateTime};
use exif::{DateTime as ExifDateTime, Exif, Field, In, Tag, Value};

pub trait ExifExt {
    fn datetime(&self, tag: Tag) -> Option<ExifDateTime>;

    // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
    // for meaning, see: https://magnushoff.com/articles/jpeg-orientation/
    // TODO[LATER]: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
    // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)
    fn orientation(&self) -> Option<u16>;
}

/// Macro making retrieval of Exif fields less visually cluttered.
macro_rules! exif_field {
    ($exif:ident [ $tag:expr ] as $val:path { ref $v:tt } => $body:block) => {
        // TODO: match &$exif ... ?
        match $exif.get_field($tag, In::PRIMARY) {
            Some(Field {
                value: $val(ref vec),
                ..
            }) if !vec.is_empty() => {
                let $v = &vec[0];
                $body
            }
            _ => None,
        }
    };
}

impl ExifExt for Exif {
    fn datetime(&self, tag: Tag) -> Option<ExifDateTime> {
        exif_field! {
            self[tag] as Value::Ascii{ref v} => {
                ExifDateTime::from_ascii(v).ok()
            }
        }
    }

    fn orientation(&self) -> Option<u16> {
        exif_field! {
            self[Tag::Orientation] as Value::Short{ref v} => {
                Some(*v)
            }
        }
    }
}

pub trait ExifDateTimeExt {
    fn to_naive_opt(&self) -> Option<NaiveDateTime>;
}

impl ExifDateTimeExt for ExifDateTime {
    fn to_naive_opt(&self) -> Option<NaiveDateTime> {
        NaiveDate::from_ymd_opt(self.year.into(), self.month.into(), self.day.into()).and_then(
            |date| date.and_hms_opt(self.hour.into(), self.minute.into(), self.second.into()),
        )
    }
}
