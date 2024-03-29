use std::ops::RangeInclusive;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{self, tree, Widget};
use iced::advanced::{image as iced_image, Clipboard, Shell};
use iced::advanced::text::{self, Text};
use iced::{alignment, mouse};
use iced::{Color, Element, event, Event, Font, Length, Point, Rectangle, Size};
use image::ImageDecoder;
use itertools::Itertools;
use tracing::{span, Level};

use crate::db;
use crate::interlude::*;

pub struct Gallery<Message> {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub selection: Selection,

    tile_w: f32,
    tile_h: f32,
    spacing: f32,
    on_select: Option<Box<dyn Fn(Selection) -> Message>>,
}

#[derive(Clone, Default, Debug)]
pub struct Selection {
    pub rowids: Vec<db::Rowid>,
    //// TODO[LATER]: usize or u32 or what?
    ///// Item clicked initially.
    //pub initial: u32,
    ///// Item clicked last.
    //pub last: u32,
}

impl Selection {
    pub fn single(rowid: db::Rowid) -> Self {
        Self {
            rowids: vec![rowid],
        }
    }

    //pub fn range(&self) -> RangeInclusive<u32> {
    //    if self.initial <= self.last {
    //        self.initial..=self.last
    //    } else {
    //        self.last..=self.initial
    //    }
    //}
}

#[derive(Default)]
struct InternalState {
    selecting_from_offset: Option<u32>,
}

impl<'a> From<&'a mut widget::Tree> for &'a mut InternalState {
    fn from(tree: &'a mut widget::Tree) -> &'a mut InternalState {
        tree.state.downcast_mut()
    }
}

impl<'a> From<&'a widget::Tree> for &'a InternalState {
    fn from(tree: &'a widget::Tree) -> &'a InternalState {
        tree.state.downcast_ref()
    }
}

impl<Message> Gallery<Message> {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self {
            db,
            selection: Default::default(),

            tile_w: 200.0,
            tile_h: 200.0,
            spacing: 25.0,
            on_select: None,
        }
    }

    pub fn with_selection(mut self, s: Selection) -> Self {
        self.selection = s;
        self
    }

    pub fn on_select(mut self, f: impl Fn(Selection) -> Message + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    fn columns(&self, layout: &Layout) -> u32 {
        ((layout.bounds().width - self.spacing) / (self.tile_w + self.spacing)) as u32
    }

    fn offset_from_xy(&self, layout: &Layout, p: Point) -> Option<u32> {
        // Note: all calculations in "full" layout coordinates, not in a virtual viewport window.

        if p.x < 0.0 || p.y < 0.0 {
            return None;
        }

        let x_without_left_margin = 0f32.max(p.x - self.spacing);
        let col_w = self.tile_w + self.spacing;
        let col = (x_without_left_margin / col_w) as u32;

        let y_without_top_margin = 0f32.max(p.y - self.spacing);
        let row_h = self.tile_h + self.spacing;
        let row = (y_without_top_margin / row_h) as u32;

        Some(row * self.columns(layout) + col)
    }

    fn rowid_from_offset(&self, offset: u32) -> Option<db::Rowid> {
        self.rowids_from_offsets(offset..=offset)
            .first()
            .map(|v| *v)
    }

    // FIXME: use i64, per Sqlite internals I think, instead of u32
    fn rowids_from_offsets(&self, range: RangeInclusive<u32>) -> Vec<db::Rowid> {
        let prof_span = span!(Level::TRACE, "gallery::rowids_from_offsets");
        let _enter = prof_span.enter();

        let len = range.end() - range.start() + 1;
        let oal = db::OffsetAndLimit::new((*range.start()).into(), len.into());
        // TODO: is it avoidable locking DB on every mouse move?
        let db = self.db.lock().unwrap();
        db::visible_files_rowids(&db, oal)
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Gallery<Message>
where
    Renderer: text::Renderer<Font = iced::Font>
        + iced_image::Renderer<Handle = iced_image::Handle>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InternalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InternalState::default())
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let prof_span = span!(Level::TRACE, "gallery::layout");
        let _enter = prof_span.enter();

        // println!("MCDBG Gallery::layout(limits: {:?})", limits);

        let db = self.db.lock().unwrap();
        let n_files = crate::db::n_files(&db);
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
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        // cursor_position: Point,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let prof_span = span!(Level::TRACE, "gallery::draw");
        let _enter = prof_span.enter();

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
        let offset = self
            .offset_from_xy(&layout, Point::new(0., viewport.y))
            .unwrap_or(0);
        let limit = (2 + (viewport.height / (self.tile_h + self.spacing)) as u32) * columns;

        let span_dblock = span!(Level::TRACE, "draw/dblock");
        let guard_dblock = span_dblock.enter();
        let db = self.db.lock().unwrap();
        drop(guard_dblock);

        // FIXME: calculate LIMIT & OFFSET based on viewport vs. layout.bounds
        // TODO[LATER]: think whether to remove .unwrap()
        let span_filequery_init = span!(Level::TRACE, "draw/filequery_init");
        let guard_filequery_init = span_filequery_init.enter();
        let mut query = crate::db::visible_files_in_limit_and_offset(&db);
        let file_iter = query.run((limit.into(), offset.into())).map(|v| v.unwrap());
        drop(guard_filequery_init);

        // println!("{:?} {:?}", layout.bounds(), &viewport);

        let span_fenumerate = span!(Level::TRACE, "draw/fenumerate");
        let guard_fenumerate = span_fenumerate.enter();
        let mut last_date = String::new();
        let mut x = self.spacing;
        let mut y = self.spacing + (offset / columns) as f32 * (self.tile_h + self.spacing);
        for (rowid, file) in file_iter {
            let span_fileiter = span!(Level::TRACE, "draw/fileiter");
            let _guard_fileiter = span_fileiter.enter();

            // Mark tile as selected when appropriate.
            // FIXME: O(n)!!!
            if self.selection.rowids.contains(&rowid) {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: x - self.spacing / 2.,
                            y: y - self.spacing / 2.,
                            width: self.tile_w + self.spacing,
                            height: self.tile_h + self.spacing,
                        },
                        border_radius: 0.0.into(),
                        border_width: 0.,
                        border_color: Color::WHITE,
                    },
                    Color::from_rgb(0.5, 0.5, 1.),
                );
            }

            // Extract dimensions of thumbnail
            let span_jpegdec = span!(Level::TRACE, "draw/jpegdec");
            let guard_jpegdec = span_jpegdec.enter();
            let (w, h) = image::jpeg::JpegDecoder::new(std::io::Cursor::new(&file.thumb))
                .unwrap()
                .dimensions();
            drop(guard_jpegdec);
            let (w, h) = (w as f32, h as f32);
            // Calculate scale, keeping aspect ratio
            let scale = 1_f32.min((w / self.tile_w).max(h / self.tile_h));
            // Calculate alignment so that the thumbnail is centered in its space
            let align_x = (self.tile_w - w / scale) / 2.0;
            let align_y = (self.tile_h - h / scale) / 2.0;

            let span_imagethumb = span!(Level::TRACE, "draw/imagethumb");
            let guard_imagethumb = span_imagethumb.enter();
            renderer.draw(
                iced_image::Handle::from_memory(file.thumb),
                Rectangle {
                    x: x + align_x,
                    y: y + align_y,
                    width: w,
                    height: h,
                },
            );
            drop(guard_imagethumb);

            // Display date header if necessary
            // TODO[LATER]: start 1 row earlier to make sure date is not displayed too greedily
            let date = match file.date {
                Some(d) => d.format("%Y-%m-%d").to_string(),
                None => "Unknown date".to_owned(),
            };
            if date != last_date {
                last_date = date;
                renderer.fill_text(Text {
                    content: last_date.as_str(),
                    bounds: Rectangle {
                        x: x - 5.0,
                        y: y - self.spacing + 5.0,
                        width: self.tile_w,
                        height: self.spacing - 5.0,
                    },
                    size: 20.0,
                    line_height: Default::default(),
                    color: Color::BLACK,
                    font: Font::DEFAULT,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    // TODO: or Advanced?
                    shaping: Default::default(),
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
        drop(guard_fenumerate);

        // Show locations of image file in a hovering tooltip at cursor position.
        // println!("cursor: {:?}", cursor);
        if let Some(hovered_offset) = cursor.position().and_then(|p| self.offset_from_xy(&layout, p)) {
            let span_draw_tooltip = span!(Level::TRACE, "draw/tooltip");
            let _guard_draw_tooltip = span_draw_tooltip.enter();

            // println!("hovered_offset: {:?}", hovered_offset);
            let span_locations = span!(Level::TRACE, "draw/locations");
            let guard_locations = span_locations.enter();
            let mut locations_query = crate::db::locations_of_file_at_offset(&db);
            let locations = locations_query
                .run((hovered_offset.into(),))
                .map(|v| v.unwrap())
                .map(|(backend, path)| backend + ": " + path.as_str())
                .join("\n");
            drop(guard_locations);
            let text = {
                let content = locations.as_str();
                let size = 12.0;
                let line_height = Default::default();
                let font = Font::DEFAULT;
                let bounds = Size::INFINITY;
                // TODO: or Advanced?
                let shaping = Default::default();
                let measure = renderer.measure(content, size, line_height, font, bounds, shaping);
                Text {
                    content,
                    bounds: Rectangle::new(cursor.position().unwrap(), measure),
                    size,
                    line_height,
                    color: Color::BLACK,
                    font,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping,
                }
            };
            renderer.with_layer(Rectangle::with_size(Size::INFINITY), |renderer| {
                renderer.fill_quad(
                    Quad {
                        // bounds: tooltip_bounds,
                        bounds: text.bounds,
                        border_radius: 0.0.into(),
                        border_width: 0.,
                        border_color: Color::WHITE,
                    },
                    Color::from_rgb(0.9, 0.9, 0.7),
                );
                renderer.fill_text(text);
            });
        }
        // TODO[LATER]: show text message if no thumbnails in DB
    }

    fn on_event(
        &mut self,
        state: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle<f32>,
    ) -> event::Status {
        use iced::mouse::{Button, Event::*};
        let state: &mut InternalState = state.into();
        match event {
            Event::Mouse(ButtonPressed(Button::Left)) => 'handler: {
                let Some(off) = cursor.position().and_then(|p| self.offset_from_xy(&layout, p)) else {
                    break 'handler;
                };
                let Some(rowid) = self.rowid_from_offset(off) else {
                    break 'handler;
                };
                self.selection = Selection::single(rowid);
                // FIXME: what if new images get added on screen while dragging mouse? maybe we
                // should cancel selection?
                state.selecting_from_offset = Some(off);
                // println!("PRESS: {:?} off={}", cursor_position, off);
            }
            Event::Mouse(CursorMoved { .. }) => 'handler: {
                let Some(initial_off) = state.selecting_from_offset else {
                    break 'handler;
                };
                // FIXME: what if new images get added on screen while dragging mouse? maybe we
                // should cancel selection?
                let Some(off) = cursor.position().and_then(|p| self.offset_from_xy(&layout, p)) else {
                    break 'handler;
                };
                let rowids = self.rowids_from_offsets(initial_off..=off);
                self.selection.rowids = rowids;
                // println!(" MOVE: {:?}", cursor_position);
                // println!("bounds: {:?} pos: {:?}", layout.bounds(), layout.position());
            }
            Event::Mouse(ButtonReleased(Button::Left)) => {
                state.selecting_from_offset = None;
                if let Some(on_select) = &self.on_select {
                    shell.publish(on_select(self.selection.clone()));
                }
                // println!("RLASE: {:?}", cursor_position);
            }
            // FIXME: cancel selection when cursor exits window
            _ => return event::Status::Ignored,
        };
        // TODO: do we need to "invalidate" a region to ask to redraw?
        event::Status::Captured
    }
}

impl<'a, Message> From<Gallery<Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(v: Gallery<Message>) -> Element<'a, Message> {
        Element::new(v)
    }
}
