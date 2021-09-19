use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{
    layout, mouse,
    Layout, Length, Point, Size, Widget,
};

pub struct Gallery {
    // NOTE: when modifying, make sure to adjust Widget::hash_layout() if needed
}

impl Gallery {
    pub fn new() -> Self {
        Self { }
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
        _layout: Layout<'_>,
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

        let line_svg = format!(r#"
            <svg width="{0}" height="{1}">
                <line x1="0" y1="0" x2="{0}" y2="{1}" style="stroke:rgb(255,0,0);stroke-width:2" />
            </svg>"#, viewport.width, viewport.height);
        let line = iced_graphics::widget::svg::Handle::from_memory(line_svg);
        (Primitive::Svg { handle: line, bounds: *viewport }, mouse::Interaction::default())
    }
}
