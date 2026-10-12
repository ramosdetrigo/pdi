use crate::pixel_array::PixelArray;
use plotters::prelude::*;
use rayon::prelude::*;

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

    // TODO: esteganografia

    fn hist(&self) -> [usize; 256];

    fn hist_plot(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn hist_eq(&mut self);

    // TODO: convolução genérica
    fn gen_convolution(&mut self, size_filter: isize, filter: Vec<f64>);
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
        let dimensions = self.dimensions();
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
                                    sum += self.get(pos_y as usize, pos_x as usize)
                                        * filter[(filter_y * size_filter_half + filter_x) as usize];
                                }
                            }
                        }
                        *output_pix = sum;
                    });
            });
        self.data_mut().copy_from_slice(&output);
    }
}
