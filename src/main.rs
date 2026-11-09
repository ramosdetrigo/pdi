#![allow(clippy::chunks_exact_to_as_chunks)]
use iced::widget::{
    Space, button, column, container, image as iced_img, row, scrollable, text, text_input,
};
use iced::{Element, Length, Task};
use image::{ColorType, DynamicImage};
use pdi::pixel_array::{self, PixelArray};
use pdi::rgb_pixel_array::{self, RGBPixelArray};
use pdi::transforms::Transform;
use std::path::Path;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() {
    let _ = iced::application(CasKod::new, CasKod::update, CasKod::view)
        .title(CasKod::title)
        .run();
}

#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
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
    Rotation,
    GradientEdge,
    Sobel,
    Laplacian,
    HighBoost,

    RgbToHsi,
    HsiToRgb,
    HsiHistogramPlot,
    HsiHistogramEqualizer,
    HsiAdjust,
    GrayscaleSimple,
    GrayscaleWeighted,
    Sepia,
    ChromaKey,
}

enum Tos {
    RgbToGrayscale,
    GrayscaleToRgb,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    ClearImage,
    SaveImage,
    UpdateImage(DynamicImage),
    PickImage,
    PickChromaKeyOverlay,
    ExecuteFunction(Filters),
    ParameterChanged(usize, String),
    ApplyFilter,
    CancelFilter,
}

struct CasKod {
    image_handle: Option<iced_img::Handle>,
    transformer: Option<pixel_array::PixelArray>,
    transformer_rgb: Option<rgb_pixel_array::RGBPixelArray>,
    image_was_grayscale: bool,
    pending_filter: Option<Filters>,
    parameters: Vec<String>,
    chroma_key_overlay: Option<image::RgbImage>,
}

impl CasKod {
    fn new() -> Self {
        Self {
            image_handle: None,
            transformer: None,
            transformer_rgb: None,
            image_was_grayscale: false,
            pending_filter: None,
            parameters: Vec::new(),
            chroma_key_overlay: None,
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
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                    .save_file()
                {
                    if self.image_was_grayscale {
                        if let Some(transformer) = self.transformer.as_ref() {
                            let _ = transformer.clone().into_luma8().save(path);
                        }
                    } else if let Some(transformer_rgb) = self.transformer_rgb.as_ref() {
                        let _ = transformer_rgb.clone().into_rgb().save(path);
                    }
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
                    self.transformer_rgb = Some(RGBPixelArray::from(image.clone()));
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

            Message::PickChromaKeyOverlay => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                    .pick_file()
                {
                    self.chroma_key_overlay = image::open(path).ok().map(|image| image.to_rgb8());
                }

                return iced::Task::none();
            }

            Message::ExecuteFunction(filter) => {
                self.parameters = match filter {
                    Filters::Threshold | Filters::Brightness => vec!["0.5".into()],
                    Filters::Gamma => vec!["0.5".into(), "0.5".into()],
                    Filters::ContrastStretching => {
                        vec!["0.4".into(), "0.2".into(), "0.6".into(), "0.8".into()]
                    }
                    Filters::Hide => vec!["LULA 13".into()],
                    Filters::GenericConvolution => vec![
                        "3".into(), // tamanho
                        "-1.0".into(),
                        "-1.0".into(),
                        "-1.0".into(),
                        "0.0".into(),
                        "0.0".into(),
                        "0.0".into(),
                        "1.0".into(),
                        "1.0".into(),
                        "1.0".into(),
                    ],
                    Filters::Scale => vec!["2.0".into(), "2.0".into(), "Nearest".into()],
                    Filters::Rotation => vec![
                        "0.0".into(),     // theta, em radianos
                        "0".into(),       // px
                        "0".into(),       // py
                        "Nearest".into(), // interpolação
                    ],
                    Filters::GradientEdge | Filters::Sobel => vec![],
                    Filters::Laplacian | Filters::HighBoost | Filters::ChromaKey => {
                        vec!["1.0".into()]
                    }
                    Filters::HsiAdjust => vec!["0.5".into(), "0.5".into(), "0.5".into()],
                    _ => vec![],
                };

                self.pending_filter = Some(filter);
                return iced::Task::none();
            }

            Message::ParameterChanged(index, value) => {
                let is_convolution = matches!(
                    self.pending_filter.as_ref(),
                    Some(Filters::GenericConvolution)
                );

                if is_convolution && index == 0 {
                    self.parameters[0] = value.clone();

                    if let Ok(size) = value.parse::<usize>() {
                        // Limite para evitar gerar campos demais.
                        if (1..=15).contains(&size) {
                            let coefficient_count = size * size;
                            self.parameters.resize(coefficient_count + 1, "0.0".into());
                        }
                    }
                } else if let Some(parameter) = self.parameters.get_mut(index) {
                    *parameter = value;
                }

                return iced::Task::none();
            }

            Message::CancelFilter => {
                self.pending_filter = None;
                return iced::Task::none();
            }

            Message::ApplyFilter => {
                let Some(filter) = self.pending_filter.take() else {
                    return iced::Task::none();
                };

                let parameters = self.parameters.clone();
                let is_grayscale = self.image_was_grayscale;

                if is_grayscale {
                    self.apply_gray_filter(filter, &parameters);

                    let Some(transformer) = self.transformer.as_ref() else {
                        return iced::Task::none();
                    };

                    let gray = transformer.clone().into_luma8();

                    return iced::Task::done(Message::UpdateImage(
                        image::DynamicImage::ImageLuma8(gray),
                    ));
                } else {
                    self.apply_rgb_filter(filter, &parameters);

                    let Some(transformer) = self.transformer_rgb.as_ref() else {
                        return iced::Task::none();
                    };

                    let rgb = transformer.clone().into_rgb();

                    return iced::Task::done(Message::UpdateImage(image::DynamicImage::ImageRgb8(
                        rgb,
                    )));
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if let Some(filter) = self.pending_filter {}

        // Tela principal: imagem e botões.
        let left: Element<'_, Message> = match &self.image_handle {
            Some(handle) => iced_img(handle).into(),
            None => button("Carregar imagem")
                .on_press(Message::PickImage)
                .into(),
        };

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
                );
        }

        const RGB_ONLY_FILTERS: [Filters; 9] = [
            Filters::RgbToHsi,
            Filters::HsiToRgb,
            Filters::HsiHistogramPlot,
            Filters::HsiHistogramEqualizer,
            Filters::HsiAdjust,
            Filters::GrayscaleSimple,
            Filters::GrayscaleWeighted,
            Filters::Sepia,
            Filters::ChromaKey,
        ];

        for filter in Filters::iter() {
            if self.image_was_grayscale && RGB_ONLY_FILTERS.contains(&filter) {
                continue;
            }
            let nome_filtro = filter_name(&filter);
            right = right.push(
                button(text(nome_filtro))
                    .on_press(Message::ExecuteFunction(filter))
                    .width(Length::Fill),
            );
        }

        let parameters = self.parameter_container();

        row![
            container(left)
                .center_x(Length::FillPortion(3))
                .center_y(Length::Fill),
            column![
                scrollable(right)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill),
                parameters
            ]
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn apply_gray_filter(&mut self, filter: Filters, parameters: &[String]) {
        let Some(transformer) = self.transformer.as_mut() else {
            return;
        };

        match filter {
            Filters::Threshold => {
                transformer.threshold(param_f64(parameters, 0, 0.5));
            }
            Filters::Negative => transformer.neg(),
            Filters::Brightness => {
                transformer.brightness(param_f64(parameters, 0, 0.5));
            }
            Filters::Gamma => {
                transformer.gamma(param_f64(parameters, 0, 0.5), param_f64(parameters, 1, 0.5));
            }
            Filters::Hide => {
                if let Some(text) = parameters.first() {
                    transformer.hide(text);
                }
            }
            Filters::ContrastStretching => {
                transformer.contrast_stretching(
                    [param_f64(parameters, 0, 0.4), param_f64(parameters, 1, 0.2)],
                    [param_f64(parameters, 2, 0.6), param_f64(parameters, 3, 0.8)],
                );
            }
            Filters::HistogramPlot => {
                transformer.hist_plot().expect("falha ao gerar histograma");
            }
            Filters::HistogramEqualizer => transformer.hist_eq(),
            Filters::GenericConvolution => {
                let Some(size) = parameters
                    .first()
                    .and_then(|value| value.parse::<isize>().ok())
                else {
                    return;
                };

                // Exige tamanho positivo e ímpar: 1, 3, 5, 7...
                if size <= 0 || size % 2 == 0 {
                    return;
                }

                let coefficient_count = (size * size) as usize;

                if parameters.len() < coefficient_count + 1 {
                    return;
                }

                let Some(kernel) = parameters[1..=coefficient_count]
                    .iter()
                    .map(|value| value.parse::<f64>().ok())
                    .collect::<Option<Vec<_>>>()
                else {
                    return;
                };

                transformer.convolve(size, kernel);
            }
            Filters::MedianFilter => {
                transformer.median_filter(param_isize(parameters, 0, 3));
            }
            Filters::MeanFilter => {
                transformer.mean_filter(param_isize(parameters, 0, 3));
            }
            Filters::PonderatedMeanFilter => transformer.ponderated_mean_filter(),
            Filters::Scale => {
                transformer.scale(
                    param_f64(parameters, 0, 2.0),
                    param_f64(parameters, 1, 2.0),
                    pdi::transforms::Interpolation::Bilinear,
                );
            }
            Filters::Rotation => {
                let theta = param_f64(parameters, 0, 0.0);

                let Some(px) = parameters
                    .get(1)
                    .and_then(|value| value.parse::<i32>().ok())
                else {
                    return;
                };

                let Some(py) = parameters
                    .get(2)
                    .and_then(|value| value.parse::<i32>().ok())
                else {
                    return;
                };

                let interpolation = match parameters.get(3).map(String::as_str) {
                    Some("Bilinear") => pdi::transforms::Interpolation::Bilinear,
                    _ => pdi::transforms::Interpolation::Nearest,
                };

                transformer.rotation(theta, px, py, interpolation);
            }
            Filters::GradientEdge => transformer.gradient_edge(),
            Filters::Sobel => transformer.sobel(),
            Filters::Laplacian => {
                transformer.laplacian(param_f64(parameters, 0, 1.0));
            }
            Filters::HighBoost => {
                transformer.high_boost(param_f64(parameters, 0, 1.0));
            }
            Filters::RgbToHsi
            | Filters::HsiHistogramPlot
            | Filters::HsiHistogramEqualizer
            | Filters::HsiAdjust
            | Filters::HsiToRgb
            | Filters::ChromaKey
            | Filters::GrayscaleSimple
            | Filters::GrayscaleWeighted
            | Filters::Sepia => {}
        }
    }

    fn apply_rgb_filter(&mut self, filter: Filters, parameters: &[String]) {
        let Some(transformer) = self.transformer_rgb.as_mut() else {
            return;
        };

        match filter {
            Filters::Threshold => {
                transformer.threshold(param_f64(parameters, 0, 0.5));
            }
            Filters::Negative => transformer.neg(),
            Filters::Brightness => {
                transformer.brightness(param_f64(parameters, 0, 0.5));
            }
            Filters::Gamma => {
                transformer.gamma(param_f64(parameters, 0, 0.5), param_f64(parameters, 1, 0.5));
            }
            Filters::Hide => {
                if let Some(text) = parameters.first() {
                    transformer.hide(text);
                }
            }
            Filters::ContrastStretching => {
                transformer.contrast_stretching(
                    [param_f64(parameters, 0, 0.4), param_f64(parameters, 1, 0.2)],
                    [param_f64(parameters, 2, 0.6), param_f64(parameters, 3, 0.8)],
                );
            }
            Filters::HistogramPlot => {
                transformer.hist_plot().expect("falha ao gerar histograma");
            }
            Filters::HistogramEqualizer => transformer.hist_eq(),
            Filters::GenericConvolution => {
                let size = param_isize(parameters, 0, 0);

                // Exige tamanho positivo e ímpar: 1, 3, 5, 7...
                if size <= 0 || size % 2 == 0 {
                    return;
                }

                let coefficient_count = (size * size) as usize;

                if parameters.len() < coefficient_count + 1 {
                    return;
                }

                let Some(kernel) = parameters[1..=coefficient_count]
                    .iter()
                    .map(|value| value.parse::<f64>().ok())
                    .collect::<Option<Vec<_>>>()
                else {
                    return;
                };

                transformer.convolve(size, kernel);
            }
            Filters::MedianFilter => {
                transformer.median_filter(param_isize(parameters, 0, 3));
            }
            Filters::MeanFilter => {
                transformer.mean_filter(param_isize(parameters, 0, 3));
            }
            Filters::PonderatedMeanFilter => transformer.ponderated_mean_filter(),
            Filters::Scale => {
                let interpolation = match parameters.get(2).map(String::as_str) {
                    Some("Bilinear") => pdi::transforms::Interpolation::Bilinear,
                    _ => pdi::transforms::Interpolation::Nearest,
                };

                transformer.scale(
                    param_f64(parameters, 0, 2.0),
                    param_f64(parameters, 1, 2.0),
                    interpolation,
                );
            }
            Filters::Rotation => {
                let interpolation = match parameters.get(3).map(String::as_str) {
                    Some("Bilinear") => pdi::transforms::Interpolation::Bilinear,
                    _ => pdi::transforms::Interpolation::Nearest,
                };

                transformer.rotation(
                    param_f64(parameters, 0, 0.0),
                    param_isize(parameters, 1, 0) as i32,
                    param_isize(parameters, 2, 0) as i32,
                    interpolation,
                );
            }
            Filters::GradientEdge => transformer.gradient_edge(),
            Filters::Sobel => transformer.sobel(),
            Filters::Laplacian => {
                transformer.laplacian(param_f64(parameters, 0, 1.0));
            }
            Filters::HighBoost => {
                transformer.high_boost(param_f64(parameters, 0, 1.0));
            }
            Filters::RgbToHsi => transformer.rgb_to_hsi(),
            Filters::HsiToRgb => transformer.hsi_to_rgb(),
            Filters::GrayscaleSimple => transformer.grayscale_simple(),
            Filters::GrayscaleWeighted => transformer.grayscale_weighted(),
            Filters::Sepia => transformer.sepia(),
            Filters::HsiHistogramPlot => {
                transformer
                    .hist_plot_hsi()
                    .expect("falha ao gerar histograma HSI");
            }
            Filters::HsiHistogramEqualizer => transformer.hist_eq_hsi(),
            Filters::HsiAdjust => transformer.adjust_hsi(
                param_f64(parameters, 0, 0.5),
                param_f64(parameters, 1, 0.5),
                param_f64(parameters, 2, 0.5),
            ),
            Filters::ChromaKey => {
                if let (Some(transformer), Some(overlay)) = (
                    self.transformer_rgb.as_mut(),
                    self.chroma_key_overlay.as_ref(),
                ) {
                    transformer.chroma_key(param_f64(parameters, 0, 0.5), overlay);
                }
            }
        }
    }

    fn parameter_container(&self) -> Element<'_, Message> {
        let mut ret = container(text("Selecione um filtro"));
        if let Some(filter) = self.pending_filter {
            if matches!(filter, Filters::GenericConvolution) {
                let size = self
                    .parameters
                    .first()
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|&size| (1..=15).contains(&size))
                    .unwrap_or(3);

                let matrix_rows: Vec<Element<'_, Message>> = (0..size)
                    .map(|row_index| {
                        let cells: Vec<Element<'_, Message>> = (0..size)
                            .map(|column_index| {
                                let index = 1 + row_index * size + column_index;
                                let value = self
                                    .parameters
                                    .get(index)
                                    .map(String::as_str)
                                    .unwrap_or("0.0");

                                text_input("0.0", value)
                                    .on_input(move |value| Message::ParameterChanged(index, value))
                                    .width(Length::Fill)
                                    .into()
                            })
                            .collect();

                        row(cells).spacing(8).into()
                    })
                    .collect();

                ret = container(
                    column![
                        text("Tamanho do kernel"),
                        text_input(
                            "Ex.: 3",
                            self.parameters.first().map(String::as_str).unwrap_or("3"),
                        )
                        .on_input(|value| Message::ParameterChanged(0, value)),
                        text("Coeficientes do kernel"),
                        scrollable(column(matrix_rows).spacing(8)).height(Length::FillPortion(3)),
                        button("Aplicar").on_press(Message::ApplyFilter),
                        button("Cancelar").on_press(Message::CancelFilter),
                    ]
                    .spacing(10),
                );
            } else if matches!(filter, Filters::ChromaKey) {
                let overlay_label = if self.chroma_key_overlay.is_some() {
                    "Trocar imagem de overlay"
                } else {
                    "Selecionar imagem de overlay"
                };

                ret = container(
                    column![
                        text(filter_name(&filter)),
                        text("Limiar"),
                        text_input(
                            "Digite um valor",
                            self.parameters.first().map(String::as_str).unwrap_or("0.1"),
                        )
                        .on_input(|value| Message::ParameterChanged(0, value)),
                        button(overlay_label).on_press(Message::PickChromaKeyOverlay),
                        button("Aplicar").on_press(Message::ApplyFilter),
                        button("Cancelar").on_press(Message::CancelFilter),
                    ]
                    .spacing(10),
                )
            } else {
                let inputs: Vec<Element<'_, Message>> = self
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        row![
                            text(parameter_name(&filter, index)).width(Length::Fixed(160.0)),
                            text_input("Digite um valor", value)
                                .on_input(move |value| { Message::ParameterChanged(index, value) }),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center)
                        .into()
                    })
                    .collect();

                let nome_filtro = filter_name(&filter);
                ret = container(
                    column![
                        text(nome_filtro),
                        column(inputs).spacing(8),
                        button("Aplicar").on_press(Message::ApplyFilter),
                        button("Cancelar").on_press(Message::CancelFilter),
                    ]
                    .spacing(10),
                );
            }
        }

        ret.padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn load_image<S>(path: S) -> image::DynamicImage
where
    S: AsRef<Path>,
{
    image::open(path).unwrap()
}

fn param_f64(parameters: &[String], index: usize, default: f64) -> f64 {
    parameters
        .get(index)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn param_isize(parameters: &[String], index: usize, default: isize) -> isize {
    parameters
        .get(index)
        .and_then(|value| value.parse::<isize>().ok())
        .unwrap_or(default)
}

fn filter_name(filter: &Filters) -> String {
    match filter {
        Filters::Threshold => String::from("Limiar"),
        Filters::Negative => String::from("Negativo"),
        Filters::Brightness => String::from("Brilho"),
        Filters::Gamma => String::from("Gamma???"),
        Filters::Hide => String::from("Esconder mensagem"),
        Filters::ContrastStretching => String::from("Ajuste de contraste"),
        Filters::HistogramPlot => String::from("Plot de histograma"),
        Filters::HistogramEqualizer => String::from("Equalizador de histograma"),
        Filters::GenericConvolution => String::from("Convolução genérica"),
        Filters::MedianFilter => String::from("Filtro da mediana"),
        Filters::MeanFilter => String::from("Filtro da média simples"),
        Filters::PonderatedMeanFilter => String::from("Filtro da média ponderada"),
        Filters::Scale => String::from("Escala"),
        Filters::Rotation => String::from("Rotação"),
        Filters::GradientEdge => String::from("Gradiente???"),
        Filters::Sobel => String::from("Filtro de Sobel"),
        Filters::Laplacian => String::from("Filtro Laplaciano"),
        Filters::HighBoost => String::from("Filtro High Boost"),
        Filters::RgbToHsi => String::from("RGB para HSI"),
        Filters::HsiToRgb => String::from("HSI para RGB"),
        Filters::HsiHistogramPlot => String::from("Plot de histograma do HSI"),
        Filters::HsiHistogramEqualizer => String::from("Equalizador de histograma do HSI"),
        Filters::HsiAdjust => String::from("Ajuste de cor via HSI"),
        Filters::GrayscaleSimple => String::from("RGB para Escala de cinza simples"),
        Filters::GrayscaleWeighted => String::from("RGB para Escala de cinza ponderada"),
        Filters::Sepia => String::from("Sepia"),
        Filters::ChromaKey => String::from("Chroma Key"),
    }
}

fn parameter_name(filter: &Filters, index: usize) -> String {
    match filter {
        Filters::Threshold => "Limiar".into(),
        Filters::Brightness => "Brilho".into(),

        Filters::Gamma => match index {
            0 => "Gamma".into(),
            1 => "Correção".into(),
            _ => format!("Parâmetro {}", index + 1),
        },

        Filters::Hide => "Texto".into(),

        Filters::ContrastStretching => match index {
            0 => "Ponto de entrada A".into(),
            1 => "Ponto de saída A".into(),
            2 => "Ponto de entrada B".into(),
            3 => "Ponto de saída B".into(),
            _ => format!("Parâmetro {}", index + 1),
        },

        Filters::MedianFilter | Filters::MeanFilter => "Tamanho do filtro".into(),

        Filters::Scale => match index {
            0 => "Escala X".into(),
            1 => "Escala Y".into(),
            2 => "Interpolação (Nearest/Bilinear)".into(),
            _ => format!("Parâmetro {}", index + 1),
        },

        Filters::Rotation => match index {
            0 => "Ângulo (radianos)".into(),
            1 => "Pivô X".into(),
            2 => "Pivô Y".into(),
            3 => "Interpolação (Nearest/Bilinear)".into(),
            _ => format!("Parâmetro {}", index + 1),
        },

        Filters::HsiAdjust => match index {
            0 => "Matiz".into(),
            1 => "Saturação".into(),
            2 => "Brilho".into(),
            _ => format!("Parâmetro {}", index + 1),
        },

        Filters::Laplacian | Filters::HighBoost => "Intensidade".into(),

        _ => format!("Parâmetro {}", index + 1),
    }
}
