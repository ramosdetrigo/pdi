use iced::widget::{button, column, container, image as iced_img, row, scrollable, text};
use iced::{Element, Length, Task};
use image::RgbaImage;
use pdi::pixel_array::{self, PixelArray};
use pdi::transforms::Transform;
use std::path::Path;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() {
    let _ = iced::application(CasKod::new, CasKod::update, CasKod::view)
        .title(CasKod::title)
        .run();
}

#[derive(EnumIter, Debug, Clone)]
enum Filters {
    Threshold,
    Negative,
    Brightness,
    Gamma,
    Hide,
    ContrastStretching,
    HistogramPlot,
    HistogramEqualizer,
    GenericConvolution,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    ClearImage,
    UpdateImage(RgbaImage),
    PickImage,
    ExecuteFunction(Filters),
}

struct CasKod {
    image_handle: Option<iced_img::Handle>,
    transformer: Option<pixel_array::PixelArray>,
}

impl CasKod {
    fn new() -> Self {
        Self {
            image_handle: None,
            transformer: None,
        }
    }

    fn title(&self) -> String {
        String::from("Mia :3")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        #[allow(clippy::single_match)]
        match message {
            Message::ClearImage => {
                self.image_handle = None;
                self.transformer = None;
            }

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
            Message::ExecuteFunction(filter) => match filter {
                Filters::Threshold => self.transformer.as_mut().unwrap().threshold(0.5),
                Filters::Negative => self.transformer.as_mut().unwrap().neg(),
                Filters::Brightness => self.transformer.as_mut().unwrap().brightness(0.5),
                Filters::Gamma => self.transformer.as_mut().unwrap().gamma(0.5, 0.5),
                Filters::Hide => self.transformer.as_mut().unwrap().hide("LULA 13"),
                Filters::ContrastStretching => self
                    .transformer
                    .as_mut()
                    .unwrap()
                    .contrast_stretching([0.4, 0.2], [0.6, 0.8]),
                Filters::HistogramPlot => self
                    .transformer
                    .as_mut()
                    .unwrap()
                    .hist_plot()
                    .expect("REASON"),
                Filters::HistogramEqualizer => self.transformer.as_mut().unwrap().hist_eq(),
                Filters::GenericConvolution => self
                    .transformer
                    .as_mut()
                    .unwrap()
                    .gen_convolution(3, vec![-1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            },
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

        for filter in Filters::iter() {
            right = right.push(
                button(text(format!("Filtro {:?}", filter)))
                    .on_press(Message::ExecuteFunction(filter))
                    .width(Length::Fill),
            );
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
