use iced_graphics::{Backend, Renderer};
use iced_native::{
    layout, mouse,
    Layout, Length, Point, Widget,
};

pub struct Gallery {
}

impl Gallery {
    pub fn new() -> Self {
        Self { }
    }
}

impl<Message, B> Widget<Message, Renderer<B>> for Gallery
where B: Backend,
{
    fn width(&self) -> Length { todo!() }

    fn height(&self) -> Length { todo!() }

    fn layout(&self, _: &Renderer<B>, _: &layout::Limits) -> layout::Node { todo!() }

    fn hash_layout(&self, _: &mut iced_native::Hasher) { todo!() }

    fn draw(
        &self,
        _: &mut Renderer<B>,
        _: &iced_graphics::Defaults,
        _: Layout<'_>,
        _cursor: Point,
        _viewport: &iced_graphics::Rectangle,
    ) -> (iced_graphics::Primitive, mouse::Interaction) {
        todo!()
    }
}
