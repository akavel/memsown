use std::ops::Range;
use std::sync::{Arc, Mutex};

use iced_graphics::{Backend, Background, Color, Primitive, Rectangle, Renderer};
use iced_native::event::{self, Event};
use iced_native::{layout, mouse, Clipboard, Layout, Length, Point, Size, Text, Widget};
use image::ImageDecoder;
use itertools::Itertools;
use rusqlite::params;

pub struct Gallery {
    // NOTE: when modifying, make sure to adjust Widget::hash_layout() if needed
    db: Arc<Mutex<rusqlite::Connection>>,
    tile_w: f32,
    tile_h: f32,
    spacing: f32,

    // FIXME: should expose helper `State` struct instead, to be stored in user's App (see:
    // `iced/examples/todos/`, how `text_input::State` is stored)
    // TODO[LATER]: usize or u32 or what?
    // Note: first item in tuple is "first clicked", not "smaller of two">
    // Range is inclusive on both sides.
    selection: (u32, u32),
    selecting: bool,
}

impl Gallery {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self {
            db,
            tile_w: 200.0,
            tile_h: 200.0,
            spacing: 25.0,

            selection: (0, 0),
            selecting: false,
        }
    }

    fn columns(&self, layout: &Layout) -> u32 {
        ((layout.bounds().width - self.spacing) / (self.tile_w + self.spacing)) as u32
    }

    fn xy_to_offset(&self, layout: &Layout, p: Point) -> u32 {
        // Note: all calculations in "full" layout coordinates, not in a virtual viewport window.
        let x_without_left_margin = 0f32.max(p.x - self.spacing);
        let col_w = self.tile_w + self.spacing;
        let col = (x_without_left_margin / col_w) as u32;

        let y_without_top_margin = 0f32.max(p.y - self.spacing);
        let row_h = self.tile_h + self.spacing;
        let row = (y_without_top_margin / row_h) as u32;

        row * self.columns(&layout) + col
    }

    fn offset_selected(&self, offset: u32) -> bool {
        let s = self.selection;
        (s.0 <= offset && offset <= s.1) || (s.1 <= offset && offset <= s.0)
    }
}

impl<Message, B> Widget<Message, Renderer<B>> for Gallery
where
    B: Backend + iced_graphics::backend::Text,
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn hash_layout(&self, hasher: &mut iced_native::Hasher) {
        use std::hash::Hash;

        let db = self.db.lock().unwrap();
        let n_files: i32 = db
            .query_row("SELECT COUNT(*) FROM file", [], |row| row.get(0))
            .unwrap();
        drop(db);

        n_files.hash(hasher);
    }

    fn layout(&self, _: &Renderer<B>, limits: &layout::Limits) -> layout::Node {
        // println!("MCDBG Gallery::layout(limits: {:?})", limits);

        let db = self.db.lock().unwrap();
        let n_files: u32 = db
            .query_row("SELECT COUNT(*) FROM file", [], |row| row.get(0))
            .unwrap();
        drop(db);

        let width = limits.max().width;
        // println!("MCDBG width={:?} limits={:?}", width, limits);
        let columns = ((width - self.spacing) / (self.tile_w + self.spacing)) as u32;
        let rows: u32 = (n_files + columns - 1) / columns;

        let height = (self.spacing as u32) + rows * (self.tile_h + self.spacing) as u32;
        // println!("MCDBG n={} x={} y={} h={}", n_files, columns, rows, height);
        layout::Node::new(Size::new(width, height as f32))
    }

    fn draw(
        &self,
        renderer: &mut Renderer<B>,
        _: &iced_graphics::Defaults,
        layout: Layout<'_>,
        cursor: Point,
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

        let columns = self.columns(&layout);

        // Index of first thumbnail to draw in top-left corner
        let offset = self.xy_to_offset(&layout, Point::new(0., viewport.y));
        let limit = (2 + (viewport.height / (self.tile_h + self.spacing)) as u32) * columns;

        let db = self.db.lock().unwrap();

        // FIXME: calculate LIMIT & OFFSET based on viewport vs. layout.bounds
        // TODO[LATER]: think whether to remove .unwrap()
        let mut query = db
            .prepare_cached(
                r"SELECT hash, date, thumbnail
                    FROM file
                    ORDER BY date
                    LIMIT ? OFFSET ?",
            )
            .unwrap();
        let file_iter = query
            .query_map(params!(limit, offset), |row| {
                Ok(crate::model::FileInfo {
                    hash: row.get_unwrap(0),
                    date: row.get_unwrap(1),
                    thumb: row.get_unwrap(2),
                })
            })
            .unwrap();

        // println!("{:?} {:?}", layout.bounds(), &viewport);

        let mut last_date = String::new();
        let mut view = vec![];
        let mut x = self.spacing;
        let mut y = self.spacing + (offset / columns) as f32 * (self.tile_h + self.spacing);
        for (i, row) in file_iter.enumerate() {
            // Mark tile as selected when appropriate.
            if self.offset_selected(offset + i as u32) {
                view.push(Primitive::Quad {
                    bounds: Rectangle {
                        x: x - self.spacing / 2.,
                        y: y - self.spacing / 2.,
                        width: self.tile_w + self.spacing,
                        height: self.tile_h + self.spacing,
                    },
                    background: Background::Color(Color::from_rgb(0.5, 0.5, 1.)),
                    border_radius: 0.,
                    border_width: 0.,
                    border_color: Color::WHITE,
                })
            }

            let file = row.unwrap();

            // Extract dimensions of thumbnail
            let (w, h) = image::jpeg::JpegDecoder::new(std::io::Cursor::new(&file.thumb))
                .unwrap()
                .dimensions();
            let (w, h) = (w as f32, h as f32);
            // Calculate scale, keeping aspect ratio
            let scale = 1_f32.min((w / self.tile_w).max(h / self.tile_h));
            // Calculate alignment so that the thumbnail is centered in its space
            let align_x = (self.tile_w - w / scale) / 2.0;
            let align_y = (self.tile_h - h / scale) / 2.0;

            view.push(Primitive::Image {
                handle: iced_native::image::Handle::from_memory(file.thumb),
                bounds: Rectangle {
                    x: x + align_x,
                    y: y + align_y,
                    width: w,
                    height: h,
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
                    horizontal_alignment: iced_graphics::alignment::Horizontal::Left,
                    vertical_alignment: iced_graphics::alignment::Vertical::Top,
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

        // Show locations of image file in a hovering tooltip at cursor position.
        // println!("cursor: {:?}", cursor);
        let cursor_over_gallery = cursor.x >= 0.0 && cursor.y >= 0.0;
        if cursor_over_gallery {
            let hovered_offset = self.xy_to_offset(&layout, cursor);
            // println!("hovered_offset: {:?}", hovered_offset);
            let locations = db
                .prepare_cached(
                    r"SELECT backend_tag, path
                        FROM location
                        WHERE file_id = (SELECT rowid
                            FROM file
                            ORDER BY date
                            LIMIT 1 OFFSET ?)
                        ORDER BY backend_tag ASC, path ASC",
                )
                .unwrap()
                .query_map([hovered_offset], |row| {
                    let backend: String = row.get_unwrap(0);
                    let path: String = row.get_unwrap(1);
                    Ok(backend + ": " + path.as_str())
                })
                .unwrap()
                .map(|x: Result<String, _>| x.unwrap())
                .join("\n");
            // Note: taken from Tooltip widget
            let text_layout = Widget::<(), Renderer<B>>::layout(
                &Text::new(locations.as_str()),
                renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
                // .pad(Padding::new(padding)),
            );
            let text_bounds = text_layout.bounds();
            let tooltip_bounds = Rectangle {
                x: cursor.x,
                y: cursor.y,
                width: text_bounds.width,
                height: text_bounds.height,
            };
            view.push(Primitive::Quad {
                bounds: tooltip_bounds,
                background: Background::Color(Color::from_rgb(0.9, 0.9, 0.7)),
                border_radius: 0.,
                border_width: 0.,
                border_color: Color::WHITE,
            });
            view.push(Primitive::Text {
                content: locations,
                bounds: tooltip_bounds,
                color: Color::BLACK,
                size: 12.0,
                font: iced_graphics::Font::Default,
                horizontal_alignment: iced_graphics::alignment::Horizontal::Left,
                vertical_alignment: iced_graphics::alignment::Vertical::Top,
            });
        }

        // TODO[LATER]: show text message if no thumbnails in DB
        (
            Primitive::Group { primitives: view },
            mouse::Interaction::default(),
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer<B>,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        use iced::mouse::{Button, Event::*};
        match event {
            Event::Mouse(ButtonPressed(Button::Left)) => {
                let i = self.xy_to_offset(&layout, cursor_position);
                self.selection = (i, i);
                self.selecting = true;
                // self.selection = Some((
                // println!("PRESS: {:?}", cursor_position);
            }
            Event::Mouse(CursorMoved { .. }) => {
                if self.selecting {
                    let i = self.xy_to_offset(&layout, cursor_position);
                    self.selection.1 = i;
                }
                // println!(" MOVE: {:?}", cursor_position);
                // println!("bounds: {:?} pos: {:?}", layout.bounds(), layout.position());
            }
            Event::Mouse(ButtonReleased(Button::Left)) => {
                self.selecting = false;
                // println!("RLASE: {:?}", cursor_position);
            }
            // FIXME: cancel selection when cursor exits window
            _ => return event::Status::Ignored,
        };
        // TODO: do we need to "invalidate" a region to ask to redraw?
        event::Status::Captured
    }
}

impl<'a, Message, B> From<Gallery> for iced_native::Element<'a, Message, Renderer<B>>
where
    B: Backend + iced_graphics::backend::Text,
{
    fn from(v: Gallery) -> iced_native::Element<'a, Message, Renderer<B>> {
        iced_native::Element::new(v)
    }
}
