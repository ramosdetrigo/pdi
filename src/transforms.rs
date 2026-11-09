use crate::pixel_array::PixelArray;
use crate::rgb_pixel_array::RGBPixelArray;
use plotters::prelude::*;
use rayon::prelude::*;
use std::f64::consts::TAU;

pub enum Interpolation {
    Nearest,
    Bilinear,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Complex {
    re: f64,
    im: f64,
}

pub trait Transform {
    type Output;
    type Output2;

    fn threshold(&mut self, threshold: f64);

    fn neg(&mut self);

    fn brightness(&mut self, b: f64);

    fn gamma(&mut self, c: f64, y: f64);

    fn hide(&mut self, msg: &str);

    fn read(&self) -> String;

    fn contrast_stretching(&mut self, a: [f64; 2], b: [f64; 2]);

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn hist_eq(&mut self);

    fn convolve(&mut self, size_filter: isize, filter: Vec<f64>);

    fn median_filter(&mut self, size_filter: isize);

    fn mean_filter(&mut self, size_filter: isize);

    fn ponderated_mean_filter(&mut self);

    fn scale(&mut self, sx: f64, sy: f64, interpolation: Interpolation);

    fn rotation(&mut self, theta: f64, px: i32, py: i32, interpolation: Interpolation);

    fn derivative_x(&self, x: u32, y: u32) -> Self::Output2;

    fn derivative_y(&self, x: u32, y: u32) -> Self::Output2;

    fn gradient_edge(&mut self);

    fn sobel(&mut self);

    fn laplacian(&mut self, amount: f64);

    fn high_boost(&mut self, amount: f64);

    fn discrete_fourier_transform(&mut self) -> Result<Self::Output, &'static str>;

    fn inverse_discrete_fourier_transform(
        &mut self,
        spectrum: &Self::Output,
    ) -> Result<(), &'static str>;
}

impl Transform for PixelArray {
    type Output = Vec<Complex>;
    type Output2 = f64;

    fn brightness(&mut self, c: f64) {
        self.apply(|p| *p *= c);
    }

    fn gamma(&mut self, c: f64, y: f64) {
        self.apply(|p| *p = p.powf(y) * c);
    }

    fn neg(&mut self) {
        self.apply(|p| *p = 1.0 - *p);
    }

    fn threshold(&mut self, threshold: f64) {
        self.apply(|p| *p = if *p > threshold { 1.0 } else { 0.0 });
    }

    fn hide(&mut self, msg: &str) {
        if !msg.is_ascii() {
            panic!("Can only encode ascii characters!");
        };

        // Joga cada bit do byte no LSB
        let mut bytes = msg.to_string().into_bytes();
        bytes.push(0); // null-terminated string
        let bits: Vec<u8> = bytes
            .into_par_iter()
            .flat_map(|b| (0..8).map(|i| (b & (1 << i)) >> i).collect::<Vec<u8>>())
            .collect();

        // Esconde os bits na imagem pixel por pixel
        for i in 0..bits.len() {
            if i > self.len() {
                // para de esconder se passou do tamanho da imagem
                break;
            }
            let mut b = (self[i].clamp(0.0, 1.0) * 255.0) as u8;
            b = (b & 0b11111110) + bits[i]; // apaga o último bit e sobrescreve
            self[i] = b as f64 / 255.0
        }
    }

    fn read(&self) -> String {
        let mut chars: Vec<u8> = Vec::new();

        #[allow(clippy::chunks_exact_to_as_chunks)]
        for pixels in self.data().chunks_exact(8) {
            let mut byte = 0u8;
            for i in 0..8 {
                let pixel = (pixels[i].clamp(0.0, 1.0) * 255.0) as u8;
                let bit = pixel & 1;
                byte |= bit << i;
            }
            if byte == 0 {
                break;
            }
            chars.push(byte);
        }

        String::from_utf8_lossy(&chars).into_owned()
    }

    fn contrast_stretching(&mut self, a: [f64; 2], b: [f64; 2]) {
        assert!(
            a.iter().chain(b.iter()).all(|value| value.is_finite())
                && 0.0 <= a[0]
                && a[0] < b[0]
                && b[0] <= 1.0
                && 0.0 <= a[1]
                && a[1] < b[1]
                && b[1] <= 1.0,
            "Os pontos de contraste devem estar ordenados dentro do intervalo [0, 1]"
        );

        self.apply(|p| {
            *p = if *p < a[0] {
                *p * (a[1] / a[0])
            } else if *p < b[0] {
                a[1] + (*p - a[0]) * ((b[1] - a[1]) / (b[0] - a[0]))
            } else {
                b[1] + (*p - b[0]) * ((1.0 - b[1]) / (1.0 - b[0]))
            };
        });
    }

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.hist();
        let max_count = data.iter().copied().max().unwrap_or(0);
        let y_max = max_count.saturating_add(1).max(1);

        let root = BitMapBackend::new("hist_plot.png", (900, 600)).into_drawing_area();

        root.fill(&RGBColor(248, 250, 252))?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Distribuição de frequências",
                ("sans-serif", 32).into_font().color(&RGBColor(30, 41, 59)),
            )
            .margin(20)
            .x_label_area_size(45)
            .y_label_area_size(60)
            .build_cartesian_2d((0usize..data.len()).into_segmented(), 0usize..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Bucket")
            .y_desc("Frequência")
            .axis_desc_style(("sans-serif", 16).into_font())
            .label_style(("sans-serif", 13).into_font().color(&RGBColor(71, 85, 105)))
            .axis_style(RGBColor(148, 163, 184))
            .light_line_style(RGBColor(226, 232, 240))
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RGBColor(59, 130, 246).mix(0.8).filled().stroke_width(1))
                .data(
                    data.iter()
                        .enumerate()
                        .map(|(bucket, count)| (bucket, *count)),
                ),
        )?;

        root.present()?;

        println!("Gráfico salvo em hist_plot.png");
        Ok(())
    }

    fn hist_eq(&mut self) {
        let hist = self.hist();
        let size: f64 = self.data().par_iter().sum();
        let prob: [f64; 256] = std::array::from_fn(|i| hist[i] as f64 / size);
        let prob_acc: [f64; 256] = std::array::from_fn(|i| prob[0..=i].par_iter().sum());
        self.apply(|p| *p *= prob_acc[(*p * 255.0) as usize]);
    }

    fn convolve(&mut self, size_filter: isize, filter: Vec<f64>) {
        assert_eq!(filter.len() as isize, size_filter * size_filter);
        let mut output = vec![0.0; self.data().len()];
        let dimensions = self.get_dimensions();
        let size_filter_half = size_filter / 2;
        output
            .par_chunks_mut(dimensions.0 as usize)
            .enumerate()
            .for_each(|(y, output_col)| {
                output_col
                    .iter_mut()
                    .enumerate()
                    .for_each(|(x, output_pix)| {
                        let mut sum = 0.0;
                        for filter_y in 0..size_filter {
                            for filter_x in 0..size_filter {
                                let pos_x = x as isize + filter_x - size_filter_half;
                                let pos_y = y as isize + filter_y - size_filter_half;
                                if pos_x >= 0
                                    && pos_x < dimensions.0 as isize
                                    && pos_y >= 0
                                    && pos_y < dimensions.1 as isize
                                {
                                    let filter_index = (filter_y * size_filter + filter_x) as usize;
                                    sum += self.get(pos_y as usize, pos_x as usize)
                                        * filter[filter_index];
                                }
                            }
                        }
                        *output_pix = sum;
                    });
            });
        self.data_mut().copy_from_slice(&output);
    }

    fn median_filter(&mut self, size_filter: isize) {
        assert!(size_filter > 0 && size_filter % 2 == 1);

        let mut output = vec![0.0; self.data().len()];
        let dimensions = self.get_dimensions();
        let half = size_filter / 2;

        output
            .par_chunks_mut(dimensions.0 as usize)
            .enumerate()
            .for_each(|(y, output_row)| {
                output_row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(x, output_pixel)| {
                        let mut values = Vec::with_capacity((size_filter * size_filter) as usize);

                        for filter_y in 0..size_filter {
                            for filter_x in 0..size_filter {
                                let pos_x = x as isize + filter_x - half;
                                let pos_y = y as isize + filter_y - half;

                                if pos_x >= 0
                                    && pos_x < dimensions.0 as isize
                                    && pos_y >= 0
                                    && pos_y < dimensions.1 as isize
                                {
                                    values.push(self.get(pos_y as usize, pos_x as usize));
                                }
                            }
                        }

                        let median_index = values.len() / 2;
                        values.select_nth_unstable_by(median_index, f64::total_cmp);
                        *output_pixel = values[median_index];
                    });
            });

        self.data_mut().copy_from_slice(&output);
    }

    fn mean_filter(&mut self, size_filter: isize) {
        assert!(size_filter > 0 && size_filter % 2 == 1);

        let coefficient_count = (size_filter * size_filter) as usize;
        let weight = 1.0 / coefficient_count as f64;
        let filter = vec![weight; coefficient_count];

        self.convolve(size_filter, filter);
    }

    fn ponderated_mean_filter(&mut self) {
        let one = 1.0 / 16.0;
        let two = 2.0 / 16.0;
        let four = 4.0 / 16.0;
        let filter = vec![one, two, one, two, four, two, one, two, one];
        self.convolve(3, filter);
    }

    fn scale(&mut self, sx: f64, sy: f64, interpolation: Interpolation) {
        assert!(sx > 0.0 && sy > 0.0, "scale factors must be positive");

        let (old_width, old_height) = self.get_dimensions();
        let old_width = old_width as usize;
        let old_height = old_height as usize;
        assert!(old_width > 0 && old_height > 0, "image must not be empty");

        let new_width = ((old_width as f64 * sx).round() as usize).max(1);
        let new_height = ((old_height as f64 * sy).round() as usize).max(1);
        let mut output = vec![0.0; new_width * new_height];

        output.par_iter_mut().enumerate().for_each(|(pos, pixel)| {
            let row = pos / new_width;
            let col = pos % new_width;

            let x = ((col as f64 + 0.5) / sx - 0.5).clamp(0.0, (old_width - 1) as f64);
            let y = ((row as f64 + 0.5) / sy - 0.5).clamp(0.0, (old_height - 1) as f64);

            *pixel = match interpolation {
                Interpolation::Nearest => self.get(y.round() as usize, x.round() as usize),
                Interpolation::Bilinear => {
                    let x0 = x.floor() as usize;
                    let y0 = y.floor() as usize;
                    let x1 = (x0 + 1).min(old_width - 1);
                    let y1 = (y0 + 1).min(old_height - 1);

                    let wx = x - x0 as f64;
                    let wy = y - y0 as f64;

                    let top = (1.0 - wx) * self.get(y0, x0) + wx * self.get(y0, x1);
                    let bottom = (1.0 - wx) * self.get(y1, x0) + wx * self.get(y1, x1);

                    (1.0 - wy) * top + wy * bottom
                }
            };
        });

        self.set_dimensions(new_width as u32, new_height as u32);
        *self.data_mut_vec() = output;
    }

    fn rotation(&mut self, theta: f64, px: i32, py: i32, interpolation: Interpolation) {
        assert!(theta.is_finite(), "angle must be finite");

        let (width, height) = self.get_dimensions();
        let (width, height) = (width as usize, height as usize);

        if width == 0 || height == 0 {
            return;
        }

        let mut output = vec![0.0; self.data().len()];
        let (cos_theta, sin_theta) = (theta.cos(), theta.sin());
        let (pivot_x, pivot_y) = (px as f64, py as f64);

        // mapeia o pixel de saída à uma posição na imagem original
        output.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as f64;
            let y = (i / width) as f64;

            let dx = x - pivot_x;
            let dy = y - pivot_y;

            let source_x = pivot_x + cos_theta * dx + sin_theta * dy;
            let source_y = pivot_y - sin_theta * dx + cos_theta * dy;

            if source_x < 0.0
                || source_y < 0.0
                || source_x > (width - 1) as f64
                || source_y > (height - 1) as f64
            {
                *pixel = 0.0;
                return;
            }

            *pixel = match &interpolation {
                Interpolation::Nearest => {
                    self.get(source_y.round() as usize, source_x.round() as usize)
                }
                Interpolation::Bilinear => {
                    let x0 = source_x.floor() as usize;
                    let y0 = source_y.floor() as usize;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);

                    let wx = source_x - x0 as f64;
                    let wy = source_y - y0 as f64;

                    let top = (1.0 - wx) * self.get(y0, x0) + wx * self.get(y0, x1);
                    let bottom = (1.0 - wx) * self.get(y1, x0) + wx * self.get(y1, x1);

                    (1.0 - wy) * top + wy * bottom
                }
            };
        });

        *self.data_mut_vec() = output;
    }

    fn derivative_x(&self, x: u32, y: u32) -> Self::Output2 {
        let (x, y) = (x as usize, y as usize);
        let width = self.width() as usize;

        if width <= 1 {
            0.0
        } else if x == 0 {
            self.get(y, 1) - self.get(y, 0)
        } else if x == width - 1 {
            self.get(y, x) - self.get(y, x - 1)
        } else {
            (self.get(y, x + 1) - self.get(y, x - 1)) * 0.5
        }
    }

    fn derivative_y(&self, x: u32, y: u32) -> Self::Output2 {
        let (x, y) = (x as usize, y as usize);
        let height = self.height() as usize;

        if height <= 1 {
            0.0
        } else if y == 0 {
            self.get(1, x) - self.get(0, x)
        } else if y == height - 1 {
            self.get(y, x) - self.get(y - 1, x)
        } else {
            (self.get(y + 1, x) - self.get(y - 1, x)) * 0.5
        }
    }

    fn gradient_edge(&mut self) {
        let mut copy = self.clone();

        copy.par_iter_mut().enumerate().for_each(|(i, p)| {
            let width = self.get_dimensions().0;
            let x = i as u32 % width;
            let y = i as u32 / width;
            let dx = self.derivative_x(x, y);
            let dy = self.derivative_y(x, y);
            *p = dx.hypot(dy).clamp(0.0, 1.0);
        });

        *self = copy;
    }

    fn sobel(&mut self) {
        let mut s2 = self.clone();
        s2.convolve(3, vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0]);
        self.convolve(3, vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0]);

        self.par_iter_mut().enumerate().for_each(|(i, px)| {
            let dot = px.hypot(s2[i]);
            *px = dot.clamp(0.0, 1.0)
        });
    }

    fn laplacian(&mut self, amount: f64) {
        assert!(amount.is_finite() && amount >= 0.0);

        let mut laplacian = self.clone();
        laplacian.convolve(3, vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0]);

        self.data_mut()
            .par_iter_mut()
            .zip(laplacian.data().par_iter())
            .for_each(|(pixel, &lap)| {
                *pixel = (*pixel + amount * lap).clamp(0.0, 1.0);
            });
    }

    fn high_boost(&mut self, amount: f64) {
        assert!(amount.is_finite() && amount >= 0.0);

        let mut blurred = self.clone();
        blurred.ponderated_mean_filter();

        self.data_mut()
            .par_iter_mut()
            .zip(blurred.data().par_iter())
            .for_each(|(pixel, &blur)| {
                *pixel = (*pixel + amount * (*pixel - blur)).clamp(0.0, 1.0);
            });
    }

    fn discrete_fourier_transform(&mut self) -> Result<Self::Output, &'static str> {
        let (width, height) = self.get_dimensions();
        let mut output = vec![Complex::default(); (width * height) as usize];

        for v in 0..height {
            for u in 0..width {
                let mut sum = Complex::default();

                for y in 0..height {
                    for x in 0..width {
                        let pixel = self.get(y as usize, x as usize) as f64;
                        let angle = -TAU
                            * ((u as f64 * x as f64 / width as f64)
                                + (v as f64 * y as f64 / height as f64));

                        sum.re += pixel * angle.cos();
                        sum.im += pixel * angle.sin();
                    }
                }

                output[(v * width + u) as usize] = sum;
            }
        }

        Ok(output)
    }

    fn inverse_discrete_fourier_transform(
        &mut self,
        spectrum: &Self::Output,
    ) -> Result<(), &'static str> {
        let (width, height) = self.get_dimensions();

        if width == 0 || height == 0 || spectrum.len() != (width * height) as usize {
            return Err("Dimensões inválidas ou tamanho do espectro incompatível");
        }

        let normalization = (width * height) as f64;

        for y in 0..height {
            for x in 0..width {
                let mut sum = Complex::default();

                for v in 0..height {
                    for u in 0..width {
                        let coefficient = spectrum[(v * width + u) as usize];

                        let angle = TAU
                            * (u as f64 * x as f64 / width as f64
                                + v as f64 * y as f64 / height as f64);

                        let cos = angle.cos();
                        let sin = angle.sin();

                        // (a + bi)(cos + i sin)
                        sum.re += coefficient.re * cos - coefficient.im * sin;
                        sum.im += coefficient.re * sin + coefficient.im * cos;
                    }
                }

                let index = (y * width + x) as usize;
                self.buffer[index] = sum.re / normalization;
            }
        }

        Ok(())
    }
}

impl Transform for RGBPixelArray {
    type Output = Vec<[Complex; 3]>;
    type Output2 = [f64; 3];

    fn brightness(&mut self, c: f64) {
        self.apply(|p| {
            for channel in &mut p[..3] {
                *channel = (*channel * c).clamp(0.0, 1.0);
            }
        });
    }

    fn gamma(&mut self, c: f64, y: f64) {
        self.apply(|p| {
            for channel in &mut p[..3] {
                *channel = (channel.powf(y) * c).clamp(0.0, 1.0);
            }
        });
    }

    fn neg(&mut self) {
        self.apply(|p| {
            for channel in &mut p[..3] {
                *channel = 1.0 - *channel;
            }
        });
    }

    fn threshold(&mut self, threshold: f64) {
        self.apply(|p| {
            for channel in &mut p[..3] {
                *channel = if *channel > threshold { 1.0 } else { 0.0 };
            }
        });
    }

    fn hide(&mut self, msg: &str) {
        assert!(msg.is_ascii(), "Can only encode ASCII characters!");

        let mut bytes = msg.as_bytes().to_vec();
        bytes.push(0); // null terminator

        let pixels: Vec<&mut [f64]> = self.data_mut().collect();
        let samples = pixels.into_iter().flat_map(|pixel| pixel.iter_mut());

        for (sample, bit) in samples.zip(
            bytes
                .into_iter()
                .flat_map(|byte| (0..8).map(move |i| (byte >> i) & 1)),
        ) {
            let value = (sample.clamp(0.0, 1.0) * 255.0) as u8;
            *sample = ((value & 0b1111_1110) | bit) as f64 / 255.0;
        }
    }

    fn read(&self) -> String {
        let samples: Vec<f64> = self
            .data()
            .flat_map_iter(|pixel| pixel.iter().copied())
            .collect();

        let mut chars = Vec::new();

        #[allow(clippy::chunks_exact_to_as_chunks)]
        for samples_for_byte in samples.chunks_exact(8) {
            let mut byte = 0u8;

            for (i, sample) in samples_for_byte.iter().enumerate() {
                let bit = ((*sample).clamp(0.0, 1.0) * 255.0) as u8 & 1;
                byte |= bit << i;
            }

            if byte == 0 {
                break;
            }

            chars.push(byte);
        }

        String::from_utf8_lossy(&chars).into_owned()
    }

    fn contrast_stretching(&mut self, a: [f64; 2], b: [f64; 2]) {
        assert!(
            a.iter().chain(b.iter()).all(|value| value.is_finite()),
            "Breakpoints must be finite"
        );
        assert!(
            0.0 < a[0] && a[0] <= b[0] && b[0] < 1.0,
            "Input breakpoints must not be decreasing and inside (0, 1)"
        );
        assert!(
            0.0 <= a[1] && a[1] < b[1] && b[1] <= 1.0,
            "Output breakpoints must be increasing and inside [0, 1]"
        );

        self.apply(|pixel| {
            for channel in &mut pixel[..3] {
                *channel = if *channel < a[0] {
                    *channel * (a[1] / a[0])
                } else if *channel < b[0] {
                    a[1] + (*channel - a[0]) * ((b[1] - a[1]) / (b[0] - a[0]))
                } else {
                    b[1] + (*channel - b[0]) * ((1.0 - b[1]) / (1.0 - b[0]))
                };
            }
        });
    }

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.hist();

        let root = BitMapBackend::new("hist_plot.png", (640, 720)).into_drawing_area();
        root.fill(&WHITE)?;

        let areas = root.split_evenly((3, 1));

        let channels = [
            ("Red", &data[0], RED),
            ("Green", &data[1], GREEN),
            ("Blue", &data[2], BLUE),
        ];

        for (area, (name, histogram, color)) in areas.iter().zip(channels) {
            let max_count = histogram.iter().copied().max().unwrap_or(0);

            let mut chart = ChartBuilder::on(area)
                .caption(format!("{name} channel"), ("sans-serif", 20).into_font())
                .margin(10)
                .x_label_area_size(30)
                .y_label_area_size(45)
                .build_cartesian_2d(
                    (0usize..256).into_segmented(),
                    0usize..max_count.saturating_add(1),
                )?;

            chart
                .configure_mesh()
                .disable_x_mesh()
                .y_desc("Count")
                .x_desc("Bucket")
                .axis_desc_style(("sans-serif", 12))
                .draw()?;

            chart.draw_series(
                Histogram::vertical(&chart)
                    .margin(1)
                    .style(color.filled())
                    .data(
                        histogram
                            .iter()
                            .enumerate()
                            .map(|(bucket, count)| (bucket, *count)),
                    ),
            )?;
        }

        root.present()?;
        println!("Result has been saved");

        Ok(())
    }

    fn hist_eq(&mut self) {
        let hist = self.hist();

        let sizes: [usize; 3] = std::array::from_fn(|channel| hist[channel].iter().sum());

        if sizes.iter().all(|&size| size == 0) {
            return;
        }

        let cdfs: [[f64; 256]; 3] = std::array::from_fn(|channel| {
            let size = sizes[channel];

            if size == 0 {
                return [0.0; 256];
            }

            let mut cumulative = 0usize;

            std::array::from_fn(|bucket| {
                cumulative += hist[channel][bucket];
                cumulative as f64 / size as f64
            })
        });

        self.apply(|pixel| {
            for (channel, value) in pixel[..3].iter_mut().enumerate() {
                let bucket = (value.clamp(0.0, 1.0) * 255.0) as usize;
                *value = cdfs[channel][bucket];
            }
        });
    }

    fn convolve(&mut self, size_filter: isize, filter: Vec<f64>) {
        assert!(size_filter > 0 && size_filter % 2 == 1);
        assert_eq!(filter.len(), (size_filter * size_filter) as usize);

        let (width, height) = self.get_dimensions();
        let width = width as usize;
        let height = height as usize;
        let half = size_filter / 2;

        // Snapshot RGB pixels before writing the output.
        let source: Vec<[f64; 3]> = self
            .data()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();

        let mut output = vec![[0.0; 3]; width * height];

        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, output_row)| {
                output_row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(x, output_pixel)| {
                        for filter_y in 0..size_filter {
                            for filter_x in 0..size_filter {
                                let pos_x = x as isize + filter_x - half;
                                let pos_y = y as isize + filter_y - half;

                                if pos_x >= 0
                                    && pos_x < width as isize
                                    && pos_y >= 0
                                    && pos_y < height as isize
                                {
                                    let filter_index = (filter_y * size_filter + filter_x) as usize;
                                    let source_index = pos_y as usize * width + pos_x as usize;
                                    let weight = filter[filter_index];

                                    for channel in 0..3 {
                                        output_pixel[channel] +=
                                            source[source_index][channel] * weight;
                                    }
                                }
                            }
                        }

                        for channel in output_pixel {
                            *channel = channel.clamp(0.0, 1.0);
                        }
                    });
            });

        self.data_mut()
            .zip(output.into_par_iter())
            .for_each(|(pixel, output_pixel)| {
                pixel.copy_from_slice(&output_pixel);
            });
    }

    fn median_filter(&mut self, size_filter: isize) {
        assert!(size_filter > 0 && size_filter % 2 == 1);

        let (width, height) = self.get_dimensions();
        let width = width as usize;
        let height = height as usize;
        let half = size_filter / 2;

        // Snapshot pixels so the parallel pass reads the original image.
        let source: Vec<[f64; 3]> = self
            .data()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();

        let mut output = vec![[0.0; 3]; width * height];

        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, output_row)| {
                output_row
                    .iter_mut()
                    .enumerate()
                    .for_each(|(x, output_pixel)| {
                        let mut values: [Vec<f64>; 3] = std::array::from_fn(|_| {
                            Vec::with_capacity((size_filter * size_filter) as usize)
                        });

                        for filter_y in 0..size_filter {
                            for filter_x in 0..size_filter {
                                let pos_x = x as isize + filter_x - half;
                                let pos_y = y as isize + filter_y - half;

                                if pos_x >= 0
                                    && pos_x < width as isize
                                    && pos_y >= 0
                                    && pos_y < height as isize
                                {
                                    let source_index = pos_y as usize * width + pos_x as usize;

                                    for channel in 0..3 {
                                        values[channel].push(source[source_index][channel]);
                                    }
                                }
                            }
                        }

                        for channel in 0..3 {
                            let median_index = values[channel].len() / 2;
                            values[channel].select_nth_unstable_by(median_index, f64::total_cmp);
                            output_pixel[channel] = values[channel][median_index];
                        }
                    });
            });

        self.data_mut()
            .zip(output.into_par_iter())
            .for_each(|(pixel, output_pixel)| {
                pixel.copy_from_slice(&output_pixel);
            });
    }

    fn mean_filter(&mut self, size_filter: isize) {
        assert!(size_filter > 0 && size_filter % 2 == 1);

        let weight = 1.0 / (size_filter * size_filter) as f64;
        let filter = vec![weight; (size_filter * size_filter) as usize];

        self.convolve(size_filter, filter);
    }

    fn ponderated_mean_filter(&mut self) {
        let one = 1.0 / 16.0;
        let two = 2.0 / 16.0;
        let four = 4.0 / 16.0;
        let filter = vec![one, two, one, two, four, two, one, two, one];
        self.convolve(3, filter);
    }

    fn scale(&mut self, sx: f64, sy: f64, interpolation: Interpolation) {
        assert!(sx > 0.0 && sy > 0.0, "scale factors must be positive");

        let (old_width, old_height) = self.get_dimensions();
        let old_width = old_width as usize;
        let old_height = old_height as usize;
        assert!(old_width > 0 && old_height > 0, "image must not be empty");

        let new_width = ((old_width as f64 * sx).round() as usize).max(1);
        let new_height = ((old_height as f64 * sy).round() as usize).max(1);

        // Snapshot the source RGB pixels before replacing the image.
        let source: Vec<[f64; 3]> = self
            .data()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();

        let mut output = vec![[0.0; 3]; new_width * new_height];

        output.par_iter_mut().enumerate().for_each(|(pos, pixel)| {
            let row = pos / new_width;
            let col = pos % new_width;

            let x = ((col as f64 + 0.5) / sx - 0.5).clamp(0.0, (old_width - 1) as f64);
            let y = ((row as f64 + 0.5) / sy - 0.5).clamp(0.0, (old_height - 1) as f64);

            match interpolation {
                Interpolation::Nearest => {
                    let source_x = x.round() as usize;
                    let source_y = y.round() as usize;
                    *pixel = source[source_y * old_width + source_x];
                }
                Interpolation::Bilinear => {
                    let x0 = x.floor() as usize;
                    let y0 = y.floor() as usize;
                    let x1 = (x0 + 1).min(old_width - 1);
                    let y1 = (y0 + 1).min(old_height - 1);

                    let wx = x - x0 as f64;
                    let wy = y - y0 as f64;

                    let top_left = source[y0 * old_width + x0];
                    let top_right = source[y0 * old_width + x1];
                    let bottom_left = source[y1 * old_width + x0];
                    let bottom_right = source[y1 * old_width + x1];

                    for channel in 0..3 {
                        let top = (1.0 - wx) * top_left[channel] + wx * top_right[channel];
                        let bottom = (1.0 - wx) * bottom_left[channel] + wx * bottom_right[channel];

                        pixel[channel] = (1.0 - wy) * top + wy * bottom;
                    }
                }
            }
        });

        let output: Vec<f64> = output.into_iter().flatten().collect();

        self.set_dimensions(new_width as u32, new_height as u32);
        *self.data_mut_vec() = output;
    }

    fn rotation(&mut self, theta: f64, px: i32, py: i32, interpolation: Interpolation) {
        assert!(theta.is_finite(), "angle must be finite");

        let (width, height) = self.get_dimensions();
        let (width, height) = (width as usize, height as usize);

        if width == 0 || height == 0 {
            return;
        }

        let source: Vec<[f64; 3]> = self
            .data()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();

        let mut output = vec![[0.0; 3]; width * height];
        let (cos_theta, sin_theta) = (theta.cos(), theta.sin());
        let (pivot_x, pivot_y) = (px as f64, py as f64);

        output.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as f64;
            let y = (i / width) as f64;

            let dx = x - pivot_x;
            let dy = y - pivot_y;

            let source_x = pivot_x + cos_theta * dx + sin_theta * dy;
            let source_y = pivot_y - sin_theta * dx + cos_theta * dy;

            if source_x < 0.0
                || source_y < 0.0
                || source_x > (width - 1) as f64
                || source_y > (height - 1) as f64
            {
                return; // Output pixel is already black.
            }

            match interpolation {
                Interpolation::Nearest => {
                    let source_index =
                        source_y.round() as usize * width + source_x.round() as usize;
                    *pixel = source[source_index];
                }
                Interpolation::Bilinear => {
                    let x0 = source_x.floor() as usize;
                    let y0 = source_y.floor() as usize;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);

                    let wx = source_x - x0 as f64;
                    let wy = source_y - y0 as f64;

                    let top_left = source[y0 * width + x0];
                    let top_right = source[y0 * width + x1];
                    let bottom_left = source[y1 * width + x0];
                    let bottom_right = source[y1 * width + x1];

                    for channel in 0..3 {
                        let top = (1.0 - wx) * top_left[channel] + wx * top_right[channel];
                        let bottom = (1.0 - wx) * bottom_left[channel] + wx * bottom_right[channel];

                        pixel[channel] = (1.0 - wy) * top + wy * bottom;
                    }
                }
            }
        });

        *self.data_mut_vec() = output.into_iter().flatten().collect();
    }

    fn derivative_x(&self, x: u32, y: u32) -> Self::Output2 {
        let width = self.width() as usize;
        let (x, y) = (x as usize, y as usize);
        let mut out = [0.0; 3];

        if width <= 1 {
            return out;
        }

        let (a, b, scale) = if x == 0 {
            (self.get(y, 1), self.get(y, 0), 1.0)
        } else if x == width - 1 {
            (self.get(y, x), self.get(y, x - 1), 1.0)
        } else {
            (self.get(y, x + 1), self.get(y, x - 1), 0.5)
        };

        for c in 0..3 {
            out[c] = (a[c] - b[c]) * scale;
        }
        out
    }

    fn derivative_y(&self, x: u32, y: u32) -> Self::Output2 {
        let height = self.height() as usize;
        let (x, y) = (x as usize, y as usize);
        let mut out = [0.0; 3];

        if height <= 1 {
            return out;
        }

        let (a, b, scale) = if y == 0 {
            (self.get(1, x), self.get(0, x), 1.0)
        } else if y == height - 1 {
            (self.get(y, x), self.get(y - 1, x), 1.0)
        } else {
            (self.get(y + 1, x), self.get(y - 1, x), 0.5)
        };

        for c in 0..3 {
            out[c] = (a[c] - b[c]) * scale;
        }
        out
    }

    fn gradient_edge(&mut self) {
        let width = self.width() as usize;
        if width == 0 || self.height() == 0 {
            return;
        }

        let mut result = vec![0.0; self.buffer.len()];
        let this = &*self;

        // chunks de 3 = um pixel (r, g, b); i é o índice do PIXEL
        result.par_chunks_mut(3).enumerate().for_each(|(i, px)| {
            let (x, y) = ((i % width) as u32, (i / width) as u32);
            let dx = this.derivative_x(x, y);
            let dy = this.derivative_y(x, y);

            for c in 0..3 {
                px[c] = dx[c].hypot(dy[c]).clamp(0.0, 1.0);
            }
        });

        self.buffer = result;
    }

    fn sobel(&mut self) {
        let mut s2 = self.clone();
        s2.convolve(3, vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0]);
        self.convolve(3, vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0]);

        self.par_iter_mut().enumerate().for_each(|(i, px)| {
            let dot = px.hypot(s2[i]);
            *px = dot.clamp(0.0, 1.0)
        });
    }

    fn laplacian(&mut self, amount: f64) {
        assert!(amount.is_finite() && amount >= 0.0);

        let mut laplacian = self.clone();
        laplacian.convolve(3, vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0]);

        self.data_mut()
            .zip(laplacian.data())
            .for_each(|(pixel, lap)| {
                for channel in 0..3 {
                    pixel[channel] = (pixel[channel] + amount * lap[channel]).clamp(0.0, 1.0);
                }
            });
    }

    fn high_boost(&mut self, amount: f64) {
        assert!(amount.is_finite() && amount >= 0.0);

        let mut blurred = self.clone();
        blurred.ponderated_mean_filter();

        self.data_mut()
            .zip(blurred.data())
            .for_each(|(pixel, blur)| {
                for channel in 0..3 {
                    pixel[channel] = (pixel[channel] + amount * (pixel[channel] - blur[channel]))
                        .clamp(0.0, 1.0);
                }
            });
    }

    fn discrete_fourier_transform(&mut self) -> Result<Self::Output, &'static str> {
        let (width, height) = self.get_dimensions();

        if width == 0 || height == 0 {
            return Err("Dimensões inválidas");
        }

        let mut output = vec![[Complex::default(); 3]; (width * height) as usize];

        for v in 0..height {
            for u in 0..width {
                let mut sums = [Complex::default(); 3];

                for y in 0..height {
                    for x in 0..width {
                        let pixel = self.get(y as usize, x as usize);

                        let angle = -TAU
                            * ((u as f64 * x as f64 / width as f64)
                                + (v as f64 * y as f64 / height as f64));

                        let cos = angle.cos();
                        let sin = angle.sin();

                        for channel in 0..3 {
                            let value = pixel[channel];

                            sums[channel].re += value * cos;
                            sums[channel].im += value * sin;
                        }
                    }
                }

                let index = (v * width + u) as usize;
                output[index] = sums;
            }
        }

        Ok(output)
    }

    fn inverse_discrete_fourier_transform(
        &mut self,
        spectrum: &Self::Output,
    ) -> Result<(), &'static str> {
        let (width, height) = self.get_dimensions();

        if width == 0 || height == 0 || spectrum.len() != (width * height) as usize {
            return Err("Dimensões inválidas ou tamanho do espectro incompatível");
        }

        let normalization = (width * height) as f64;

        for y in 0..height {
            for x in 0..width {
                let mut sums = [Complex::default(); 3];

                for v in 0..height {
                    for u in 0..width {
                        let coefficients = spectrum[(v * width + u) as usize];

                        let angle = TAU
                            * (u as f64 * x as f64 / width as f64
                                + v as f64 * y as f64 / height as f64);

                        let cos = angle.cos();
                        let sin = angle.sin();

                        for channel in 0..3 {
                            let coefficient = coefficients[channel];

                            // (a + bi)(cos + i sin)
                            sums[channel].re += coefficient.re * cos - coefficient.im * sin;
                            sums[channel].im += coefficient.re * sin + coefficient.im * cos;
                        }
                    }
                }

                let pixel_index = (y * width + x) as usize;
                let buffer_index = 3 * pixel_index;

                for channel in 0..3 {
                    self.buffer[buffer_index + channel] = sums[channel].re / normalization;
                }
            }
        }

        Ok(())
    }
}
