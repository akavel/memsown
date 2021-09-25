use chrono::naive::{NaiveDate, NaiveDateTime};
use exif::{DateTime as ExifDateTime, Exif, Tag};
use if_chain::if_chain;

pub trait ExifExt {
    fn datetime(&self, tag: Tag) -> Option<ExifDateTime>;

    // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
    // for meaning, see: https://magnushoff.com/articles/jpeg-orientation/
    // TODO[LATER]: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
    // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)
    fn orientation(&self) -> Option<u16>;
}

impl ExifExt for Exif {
    fn datetime(&self, tag: Tag) -> Option<ExifDateTime> {
        use exif::{Field, In, Value};

        if_chain! {
            if let Some(field) = self.get_field(tag, In::PRIMARY);
            if let Field { value: Value::Ascii(ref vec), .. } = field;
            if !vec.is_empty();
            then {
                ExifDateTime::from_ascii(&vec[0]).ok()
            } else {
                None
            }
        }
    }

    fn orientation(&self) -> Option<u16> {
        use exif::{Field, In, Value};

        let tag = Tag::Orientation;
        if_chain! {
            if let Some(field) = self.get_field(tag, In::PRIMARY);
            if let Field { value: Value::Short(ref vec), .. } = field;
            if !vec.is_empty();
            then {
                Some(vec[0])
            } else {
                None
            }
        }
    }

}

pub trait ExifDateTimeExt {
    fn to_naive_opt(&self) -> Option<NaiveDateTime>;
}

impl ExifDateTimeExt for ExifDateTime {
    fn to_naive_opt(&self) -> Option<NaiveDateTime> {
        NaiveDate::from_ymd_opt(self.year.into(), self.month.into(), self.day.into())
            .and_then(|date| date.and_hms_opt(self.hour.into(), self.minute.into(), self.second.into()))
    }
}

