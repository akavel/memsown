use iced::Sandbox;

fn main() -> iced::Result {
    println!("Hello preview");

    ShowPreview::run(iced::Settings::default())
}

struct ShowPreview {}

impl iced::Sandbox for ShowPreview {
    type Message = ();

    fn new() -> ShowPreview {
        ShowPreview {}
    }

    fn title(&self) -> String {
        String::from("Backer image preview") // FIXME[LATER]
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let path = r"C:\fotki\incoming - z telefonu\20170426_124522.jpg";

        // FIXME: Milestone: deorient the image [apply matrix possibly]
        // see: https://github.com/iced-rs/iced/discussions/1064
        // - write helper GUI app previewing effects on the testsuite of images from links below
        // - add local checkouted dependency of iced into the sample's Cargo.toml
        // - add Exif dependency in iced
        // - parse Exif data from memory or disk (if possible)
        // - extract Exif orientation info
        // - translate orientation info to proper transforms
        // - perform the transforms on the in-memory buffer
        //
        // SEE ALSO:
        // - https://magnushoff.com/articles/jpeg-orientation/
        // - https://github.com/recurser/exif-orientation-examples
        // - https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto

        iced::widget::image::Image::new(path).into()
    }
}
