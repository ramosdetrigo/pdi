use std::path::Path;

use iced::widget::{button, column, container, image as iced_img, row, scrollable, text};
use iced::{Element, Length, Task};
use image::RgbaImage;

fn main() {
    let _ = iced::application(CasKod::new, CasKod::update, CasKod::view)
        .title(CasKod::title)
        .run();
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    ClearImage,
    UpdateImage(RgbaImage),
    PickImage,
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

    fn update(&mut self, message: Message) -> Task<Message> {
        #[allow(clippy::single_match)]
        match message {
            Message::ClearImage => self.image_handle = None,
            Message::UpdateImage(buffer) => {
                self.image_handle = Some(iced_img::Handle::from_rgba(
                    buffer.width(),
                    buffer.height(),
                    buffer.into_raw(),
                ))
            }
            Message::PickImage => {
                // Não usar async pra impedir inputs no programa enquanto o arquivo não foi selecionado
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                    .pick_file()
                {
                    return Task::done(Message::UpdateImage(load_image(file_path)));
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Esquerda: Imagem (botão de carregar imagem quando vazio)
        let left: Element<'_, Message> = match &self.image_handle {
            Some(handle) => iced_img(handle).into(),
            None => button("Carregar imagem")
                .on_press(Message::PickImage)
                .into(),
        };

        // Direita: Filtros + botão de limpar imagem
        let mut right = column![];
        if self.image_handle.is_some() {
            right = right.push(
                button("Limpar imagem")
                    .on_press(Message::ClearImage)
                    .width(Length::Fill),
            )
        }

        for i in 0..100 {
            right = right.push(button(text(format!("Filtro {i}"))).width(Length::Fill));
        }

        // Imagem: 80% da tela na esquerda
        // Filtros: 20% da tela na direita
        row![
            container(left)
                .center_x(Length::FillPortion(4))
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

fn load_image<S>(path: S) -> RgbaImage
where
    S: AsRef<Path>,
{
    image::open(path).unwrap().into_rgba8()
}
