use crate::pixel_array::PixelArray;
use plotters::prelude::*;
use rayon::prelude::*;

pub enum Interpolation {
    Nearest,
    Bilinear,
}

pub trait Transform {
    fn apply<OP>(&mut self, op: OP)
    where
        OP: Fn(&mut f64) + Sync + Send;

    fn threshold(&mut self, threshold: f64);

    fn neg(&mut self);

    fn brightness(&mut self, b: f64);

    fn gamma(&mut self, c: f64, y: f64);

    fn hide(&mut self, msg: &str);

    fn read(&self) -> String;

    fn contrast_stretching(&mut self, a: [f64; 2], b: [f64; 2]);

    fn hist(&self) -> [usize; 256];

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn hist_eq(&mut self);

    fn gen_convolution(&mut self, size_filter: isize, filter: Vec<f64>);

    fn median_filter(&mut self, size_filter: isize);

    fn mean_filter(&mut self, size_filter: isize);

    fn ponderated_mean_filter(&mut self);

    fn scale(&mut self, sx: f64, sy: f64, interpolation: Interpolation);

    fn rotation(&mut self, theta: f64, px: i32, py: i32, interpolation: Interpolation);

    fn gradient_edge(&mut self);

    fn sobel(&mut self);
}

impl Transform for PixelArray {
    fn apply<OP>(&mut self, op: OP)
    where
        OP: Fn(&mut f64) + Sync + Send,
    {
        self.data_mut().par_iter_mut().for_each(op);
    }

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
        self.apply(|p| {
            if *p < a[0] {
                *p *= a[1] / a[0]
            } else if *p < b[0] {
                *p = *p * (b[1] - a[1] / b[0] - a[0]) + a[1]
            } else {
                *p = *p * (1.0 - b[1] / 1.0 - b[0]) + b[1]
            }
        });
    }

    fn hist(&self) -> [usize; 256] {
        self.data()
            .par_iter()
            .fold(
                || [0usize; 256],
                |mut hist, &p| {
                    hist[(p.clamp(0.0, 1.0) * 255.0) as usize] += 1;
                    hist
                },
            )
            .reduce(|| [0; 256], |a, b| std::array::from_fn(|i| a[i] + b[i]))
    }

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.hist();

        let max_count = data.iter().copied().max().unwrap_or(0);

        let root = BitMapBackend::new("hist_plot.png", (640, 480)).into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Histograma :)", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d((0usize..data.len()).into_segmented(), 0usize..max_count + 1)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15))
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RED.mix(0.5).filled())
                .data(
                    data.iter()
                        .enumerate()
                        .map(|(bucket, count)| (bucket, *count)),
                ),
        )?;

        root.present()?;

        println!("Result has been saved");

        Ok(())
    }

    fn hist_eq(&mut self) {
        let hist = self.hist();
        let size: f64 = self.data().par_iter().sum();
        let prob: [f64; 256] = std::array::from_fn(|i| hist[i] as f64 / size);
        let prob_acc: [f64; 256] = std::array::from_fn(|i| prob[0..=i].par_iter().sum());
        self.apply(|p| *p *= prob_acc[(*p * 255.0) as usize]);
    }

    fn gen_convolution(&mut self, size_filter: isize, filter: Vec<f64>) {
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
        let filter = vec![1.0 / 9.0; 9];
        self.gen_convolution(size_filter, filter);
    }

    fn ponderated_mean_filter(&mut self) {
        let one = 1.0 / 16.0;
        let two = 2.0 / 16.0;
        let four = 4.0 / 16.0;
        let filter = vec![one, two, one, two, four, two, one, two, one];
        self.gen_convolution(3, filter);
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
        todo!()
    }

    fn gradient_edge(&mut self) {
        todo!()
    }

    fn sobel(&mut self) {
        let mut s2 = self.clone();
        s2.gen_convolution(3, vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0]);
        self.gen_convolution(3, vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0]);

        self.par_iter_mut().enumerate().for_each(|(i, px)| {
            let dot = (*px * *px + s2[i] * s2[i]).sqrt(); // direction -- sqrt
            *px = dot.clamp(0.0, 1.0)
        });
    }
}
