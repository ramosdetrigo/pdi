use iced::widget::{button, column, container, image as iced_img, row, scrollable, text};
use iced::{Element, Length, Task};
use image::{ColorType, DynamicImage};
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
    MedianFilter,
    MeanFilter,
    PonderatedMeanFilter,
    Scale,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    ClearImage,
    SaveImage,
    UpdateImage(DynamicImage),
    PickImage,
    ExecuteFunction(Filters),
}

struct CasKod {
    image_handle: Option<iced_img::Handle>,
    transformer: Option<pixel_array::PixelArray>,
    image_was_grayscale: bool,
}

impl CasKod {
    fn new() -> Self {
        Self {
            image_handle: None,
            transformer: None,
            image_was_grayscale: false,
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

            Message::SaveImage => {
                if let (Some(path), Some(transformer)) = (
                    rfd::FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                        .save_file(),
                    self.transformer.as_ref(),
                ) {
                    // TODO: salvar imagem rgba
                    let gray = transformer.clone().into_luma8();
                    let _ = gray.save(path);
                }
            }

            Message::UpdateImage(image) => {
                self.image_was_grayscale = matches!(
                    image.color(),
                    ColorType::L8 | ColorType::L16 | ColorType::La8 | ColorType::La16
                );

                if self.image_was_grayscale {
                    self.transformer = Some(PixelArray::from(image.clone()));
                } else {
                    todo!() // RGBA
                }

                let rgba = image.into_rgba8();

                self.image_handle = Some(iced_img::Handle::from_rgba(
                    rgba.width(),
                    rgba.height(),
                    rgba.into_raw(),
                ));
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
            Message::ExecuteFunction(filter) => {
                if self.image_was_grayscale {
                    self.apply_gray_filter(filter);
                    let gray = self.transformer.as_ref().unwrap().clone().into_luma8();
                    return Task::done(Message::UpdateImage(image::DynamicImage::ImageLuma8(gray)));
                }
                todo!() // RGBA
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
            right = right
                .push(
                    button("Limpar imagem")
                        .on_press(Message::ClearImage)
                        .width(Length::Fill),
                )
                .push(
                    button("Salvar imagem")
                        .on_press(Message::SaveImage)
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
    fn apply_gray_filter(&mut self, filter: Filters) {
        let Some(transformer) = self.transformer.as_mut() else {
            return;
        };

        match filter {
            Filters::Threshold => transformer.threshold(0.5),
            Filters::Negative => transformer.neg(),
            Filters::Brightness => transformer.brightness(0.5),
            Filters::Gamma => transformer.gamma(0.5, 0.5),
            Filters::Hide => transformer.hide("LULA 13"),
            Filters::ContrastStretching => transformer.contrast_stretching([0.4, 0.2], [0.6, 0.8]),
            Filters::HistogramPlot => {
                transformer.hist_plot().expect("REASON");
            }
            Filters::HistogramEqualizer => transformer.hist_eq(),
            Filters::GenericConvolution => {
                transformer.gen_convolution(3, vec![-1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
            }
            Filters::MedianFilter => transformer.median_filter(3),
            Filters::MeanFilter => transformer.mean_filter(3),
            Filters::PonderatedMeanFilter => transformer.ponderated_mean_filter(),
            Filters::Scale => transformer.scale(2.0, 2.0, pdi::transforms::Interpolation::Bilinear),
        }
    }
}

fn load_image<S>(path: S) -> image::DynamicImage
where
    S: AsRef<Path>,
{
    image::open(path).unwrap()
}
