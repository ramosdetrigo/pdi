use iced::widget::{button, column, container, image as iced_img, row, scrollable, text};
use iced::{Element, Length};
use image::RgbaImage;

fn main() {
    let _ = iced::application(CasKod::new, CasKod::update, CasKod::view)
        .title(CasKod::title)
        .run();
}

#[derive(Debug, Clone)]
enum Message {
    EraseImage,
    UpdateImage(RgbaImage),
}

struct CasKod {
    image_handle: Option<iced_img::Handle>,
}

impl CasKod {
    fn new() -> Self {
        Self { image_handle: None }
    }

    fn title(&self) -> String {
        String::from("Mia :3")
    }

    fn update(&mut self, message: Message) {
        #[allow(clippy::single_match)]
        match message {
            Message::EraseImage => self.image_handle = None,
            Message::UpdateImage(buffer) => {
                self.image_handle = Some(iced_img::Handle::from_rgba(
                    buffer.width(),
                    buffer.height(),
                    buffer.into_raw(),
                ))
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let left: Element<'_, Message> = match &self.image_handle {
            Some(handle) => iced_img(handle).into(),
            None => button("Carregar imagem")
                .on_press(Message::UpdateImage(
                    image::open("mia.jpeg").unwrap().into_rgba8(),
                ))
                .into(),
        };

        let mut right = column![];
        if self.image_handle.is_some() {
            right = right.push(
                button("Limpar imagem")
                    .on_press(Message::EraseImage)
                    .width(Length::Fill),
            )
        }

        for i in 0..100 {
            right = right.push(button(text(format!("Filtro {i}"))).width(Length::Fill));
        }

        row![
            container(left)
                .center_x(Length::FillPortion(3))
                .center_y(Length::Fill),
            scrollable(right)
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
