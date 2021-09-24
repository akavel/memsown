use std::sync::{Arc, Mutex};

use iced_graphics::{
    Backend, Color, Primitive, Rectangle, Renderer,
};
use iced_native::{
    layout, mouse,
    Layout, Length, Point, Size, Widget,
};
use image::ImageDecoder;
use rusqlite::params;


pub struct Gallery {
    // NOTE: when modifying, make sure to adjust Widget::hash_layout() if needed
    db: Arc<Mutex<rusqlite::Connection>>,
    tile_w: f32,
    tile_h: f32,
    spacing: f32,
}

impl Gallery {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self {
            db,
            tile_w: 200.0,
            tile_h: 200.0,
            spacing: 25.0,
        }
    }
}

impl<Message, B> Widget<Message, Renderer<B>> for Gallery
where B: Backend,
{
    fn width(&self) -> Length { Length::Fill }

    fn height(&self) -> Length { Length::Fill }

    fn hash_layout(&self, hasher: &mut iced_native::Hasher) {
        use std::hash::Hash;

        let db = self.db.lock().unwrap();
        let n_files: i32 = db.query_row(
            "SELECT COUNT(*) FROM file", [],
            |row| row.get(0)).unwrap();
        drop(db);

        n_files.hash(hasher);
    }

    fn layout(
        &self,
        _: &Renderer<B>,
        limits: &layout::Limits,
    ) -> layout::Node {
        // println!("MCDBG limits: {:?}", limits);

        let db = self.db.lock().unwrap();
        let n_files: u32 = db.query_row(
            "SELECT COUNT(*) FROM file", [],
            |row| row.get(0)).unwrap();
        drop(db);

        let columns = ((limits.max().width - self.spacing) / (self.tile_w + self.spacing)) as u32;
        let rows: u32 = (n_files + columns - 1) / columns;

        let height = (self.spacing as u32) + rows * (self.tile_h + self.spacing) as u32;
        // println!("MCDBG n={} x={} y={} h={}", n_files, columns, rows, height);
        layout::Node::new(Size::new(
            limits.max().width,
            height as f32))
    }

    fn draw(
        &self,
        _: &mut Renderer<B>,
        _: &iced_graphics::Defaults,
        layout: Layout<'_>,
        _cursor: Point,
        viewport: &iced_graphics::Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        // TODO(akavel): contribute below explanation to iced_native::Widget docs
        // Note(akavel): from discord discussion:
        //  hecrj: viewport is the visible area of the layout bounds.
        //  Zuris: I see
        //  Zuris: So, while layout holds the full bounds of the widget, viewport specifies the area
        //         inside of those bounds to actually draw?
        //  hecrj: The visible part, yes. You can draw outside of it, but it won't be visible.
        //  akavel: @hecrj thanks! just to make sure: I assume the viewport's bounds are in the
        //          same coordinate system as layout.bounds(), not relative to them?
        //  hecrj: Yes, same system.

        let columns = ((layout.bounds().width - self.spacing) / (self.tile_w + self.spacing)) as u32;

        // Index of first thumbnail to draw in top-left corner
        let offset = columns * ((viewport.y - self.spacing) / (self.tile_h + self.spacing)) as u32;
        let limit = (2 + (viewport.height / (self.tile_h + self.spacing)) as u32) * columns;

        let db = self.db.lock().unwrap();

        // FIXME: calculate LIMIT & OFFSET based on viewport vs. layout.bounds
        // TODO[LATER]: think whether to remove .unwrap()
        let mut q = db.prepare_cached(r"
            SELECT hash, date, thumbnail
                FROM file
                ORDER BY date
                LIMIT ? OFFSET ?").unwrap();
        let file_iter = q.query_map(
            params!(limit, offset),
            |row| Ok(crate::model::FileInfo {
                hash: row.get_unwrap(0),
                date: row.get_unwrap(1),
                thumb: row.get_unwrap(2),
            })).unwrap();

        // println!("{:?} {:?}", layout.bounds(), &viewport);

        let mut last_date = String::new();
        let mut view = vec![];
        let mut x = self.spacing;
        let mut y = self.spacing + (offset / columns) as f32 * (self.tile_h + self.spacing);
        for row in file_iter {
            let file = row.unwrap();

            // Extract dimensions of thumbnail
            let (w, h) = match image::jpeg::JpegDecoder::new(std::io::Cursor::new(&file.thumb)).unwrap().dimensions() {
                (w, h) => (w as f32, h as f32)
            };
            // Calculate scale, keeping aspect ratio
            let scale = (1 as f32).min((w / self.tile_w).max(h / self.tile_h));
            // Calculate alignment so that the thumbnail is centered in its space
            let align_x = (self.tile_w - w/scale) / 2.0;
            let align_y = (self.tile_h - h/scale) / 2.0;

            view.push(Primitive::Image {
                handle: iced_native::image::Handle::from_memory(file.thumb),
                bounds: Rectangle {
                    x: x+align_x, y: y+align_y,
                    width: w, height: h,
                },
            });

            // Display date header if necessary
            // TODO[LATER]: start 1 row earlier to make sure date is not displayed too greedily
            let date = match file.date {
                Some(d) => d.format("%Y-%m-%d").to_string(),
                None => "Unknown date".to_owned(),
            };
            if date != last_date {
                last_date = date;
                view.push(Primitive::Text {
                    content: last_date.clone(),
                    bounds: Rectangle {
                        x: x - 5.0,
                        y: y - self.spacing + 5.0,
                        width: self.tile_w,
                        height: self.spacing - 5.0,
                    },
                    color: Color::BLACK,
                    size: 20.0,
                    font: iced_graphics::Font::Default,
                    horizontal_alignment: iced_graphics::HorizontalAlignment::Left,
                    vertical_alignment: iced_graphics::VerticalAlignment::Top,
                });
            }

            // Calculate x and y for next image
            x += self.tile_w + self.spacing;
            if x + self.tile_w > viewport.width {
                x = self.spacing;
                y += self.tile_h + self.spacing;
                if y >= viewport.y + viewport.height {
                    break;
                }
            }
        }


        // TODO[LATER]: show text message if no thumbnails in DB
        (
            Primitive::Group { primitives: view },
            mouse::Interaction::default(),
        )
    }
}


impl<'a, Message, B> Into<iced_native::Element<'a, Message, Renderer<B>>> for Gallery
where
    B: Backend,
{
    fn into(self) -> iced_native::Element<'a, Message, Renderer<B>> {
        iced_native::Element::new(self)
    }
}
