use iced::Element;
use iced::widget::{button, column, container, image, row};
use pdi::pixel_array::PixelArray;

fn main() {
    let _ = iced::application(CasKod::new, CasKod::update, CasKod::view)
        .title(CasKod::title)
        .run();
}
#[derive(Debug, Clone)]
enum Message {}

struct CasKod {
    image_handle: image::Handle,
}

impl CasKod {
    fn new() -> Self {
        Self {
            image_handle: image::Handle::from_path("mia.jpeg"),
        }
    }

    fn title(&self) -> String {
        String::from("Mia :3")
    }

    fn update(&mut self, message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        row![
            image(&self.image_handle),
            column![button("MIA1"), button("MIA2")]
        ]
        .spacing(10)
        .into()
    }
}
