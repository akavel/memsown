use iced_graphics::{Backend, Background, Color, Primitive, Renderer};
use iced_native::{
    layout, mouse,
    Layout, Length, Point, Size, Widget,
};

pub struct Gallery {
    // NOTE: when modifying, make sure to adjust Widget::hash_layout() if needed
    tmp_img: iced_native::image::Handle,
}

impl Gallery {
    pub fn new(tmp_img: iced_native::image::Handle) -> Self {
        Self { tmp_img }
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

        println!("{:?} {:?}", layout.bounds(), &viewport);

        (
            Primitive::Image { handle: self.tmp_img.clone(), bounds: *viewport },
            // Primitive::Quad {
            //     bounds: *viewport,
            //     // background: Background::Color(Color::BLACK),
            //     background: Background::Color(Color::from_rgb(0.,0.5,0.)),
            //     // border_radius: self.radius,
            //     border_radius: viewport.width/2.0,
            //     border_width: 0.0,
            //     border_color: Color::TRANSPARENT,
            // },
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
