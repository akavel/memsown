use iced::Sandbox;

fn main() -> iced::Result {
    println!("Hello preview");

    Preview::run(iced::Settings::default())
}

struct Preview {}

impl iced::Sandbox for Preview {
    type Message = ();

    fn new() -> Preview {
        Preview {}
    }

    fn title(&self) -> String {
        String::from("Backer image preview") // FIXME[LATER]
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let path = r"C:\fotki\incoming - z telefonu\20170426_124522.jpg";

        iced::widget::image::Image::new(path).into()
    }
}
