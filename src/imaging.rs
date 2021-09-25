use exif::{Exif, Tag};
use chrono::naive::{NaiveDate, NaiveDateTime};

pub trait ExifExt {
    fn try_timestamp<T>(&self, tags: T) -> Option<NaiveDateTime>
        where T: IntoIterator<Item=Tag>;
}

impl ExifExt for Exif {
    fn try_timestamp<T>(&self, tags: T) -> Option<NaiveDateTime>
        where T: IntoIterator<Item=Tag>
    {
        for tag in tags {
            if let Some(d) = exif_date_from(&self, tag) {
                if let Some(naive) = exif_date_to_naive(&d) {
                    return Some(naive);
                }
            }
        }
        None
    }
}

fn exif_date_from(exif: &Exif, tag: Tag) -> Option<::exif::DateTime> {
    use exif::{DateTime, Field, In, Value};

    match exif.get_field(tag, In::PRIMARY) {
        Some(Field {
            value: Value::Ascii(ref vec),
            ..
        }) if !vec.is_empty() => DateTime::from_ascii(&vec[0]).ok(),
        _ => None,
    }
}

fn exif_date_to_naive(d: &::exif::DateTime) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(d.year.into(), d.month.into(), d.day.into())
        .and_then(|date| date.and_hms_opt(d.hour.into(), d.minute.into(), d.second.into()))
}
