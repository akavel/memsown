use std::sync::{Arc, Mutex};

use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{
    layout, mouse,
    Layout, Length, Point, Size, Widget,
};
use image::ImageDecoder;
use rusqlite::params;


pub struct Gallery {
    // NOTE: when modifying, make sure to adjust Widget::hash_layout() if needed
    db_connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Gallery {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db_connection: db }
    }
}

impl<Message, B> Widget<Message, Renderer<B>> for Gallery
where B: Backend,
{
    fn width(&self) -> Length { Length::Fill }

    fn height(&self) -> Length { Length::Fill }

    fn hash_layout(&self, _: &mut iced_native::Hasher) {
        // TODO(akavel): if needed, fill in as appropriate once some internal state is added
    }

    fn layout(
        &self,
        _: &Renderer<B>,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Note(akavel): not 100% sure what I'm doing here yet; general idea based off:
        // https://github.com/iced-rs/iced/blob/f78108a514563411e617715443bba53f4f4610ec/examples/geometry/src/main.rs#L47-L49
        // TODO(akavel): see what happens if I use bigger Size in resolve()
        let size = limits.width(Length::Fill).height(Length::Fill).resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _: &mut Renderer<B>,
        _: &iced_graphics::Defaults,
        layout: Layout<'_>,
        _cursor: Point,
        viewport: &iced_graphics::Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        // TODO(akavel): try looking into Column (in iced_wgpu?) to understand viewport? [via Zuris@discord]

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

        // TODO[LATER]: make parametrizable
        let (tile_w, tile_h) = (200.0, 200.0);
        let spacing = 25.0;

        let db = self.db_connection.lock().unwrap();

        // TODO[LATER]: think whether to remove .unwrap()
        let mut q = db.prepare_cached(r"
            SELECT hash, date, thumbnail
                FROM file
                ORDER BY date
                LIMIT ? OFFSET ?").unwrap();
        let file_iter = q.query_map(
            params!(100, 0),
            |row| Ok(crate::model::FileInfo {
                hash: row.get_unwrap(0),
                date: row.get_unwrap(1),
                thumb: row.get_unwrap(2),
            })).unwrap();

        println!("{:?} {:?}", layout.bounds(), &viewport);

        let mut view = vec![];
        let (mut x, mut y) = (spacing, spacing);
        for row in file_iter {
            let file = row.unwrap();
            // Extract dimensions of thumbnail
            let (w, h) = match image::jpeg::JpegDecoder::new(std::io::Cursor::new(&file.thumb)).unwrap().dimensions() {
                (w, h) => (w as f32, h as f32)
            };
            // Calculate scale, keeping aspect ratio
            let scale = (1 as f32).min((w / tile_w).max(h / tile_h));
            // Calculate alignment so that the thumbnail is centered in its space
            let align_x = (tile_w - w/scale) / 2.0;
            let align_y = (tile_h - h/scale) / 2.0;

            view.push(Primitive::Image {
                handle: iced_native::image::Handle::from_memory(file.thumb),
                bounds: iced_graphics::Rectangle{
                    x: x+align_x, y: y+align_y,
                    width: w, height: h,
                },
            });

            x += tile_w + spacing;
            if x + tile_w > viewport.width {
                x = spacing;
                y += tile_h + spacing;
                if y >= viewport.height {
                    break;
                }
            }
        }


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
