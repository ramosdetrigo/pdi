use std::{
    error::Error,
    ops::{Deref, DerefMut},
    path::Path,
};

use plotters::prelude::*;

use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    prelude::*,
};

use image::{DynamicImage, ImageReader, RgbImage};

use crate::pixel_array::PixelArray;

#[derive(Clone, Debug, PartialEq)]
pub struct RGBPixelArray {
    width: u32,
    height: u32,
    buffer: Vec<f64>,
}

impl RGBPixelArray {
    pub fn from_path<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        Ok(ImageReader::open(path)?.decode()?.into())
    }

    pub fn into_rgb(self) -> RgbImage {
        RgbImage::from_vec(
            self.width,
            self.height,
            self.buffer
                .into_iter()
                .map(|c| (c.clamp(0.0, 1.0) * 255.0) as u8)
                .collect(),
        )
        .unwrap()
    }

    #[inline]
    pub fn apply<OP>(&mut self, op: OP)
    where
        OP: Fn(&mut [f64]) + Sync + Send,
    {
        self.data_mut().for_each(op);
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> [f64; 3] {
        let p = (row * self.width as usize + col) * 3;
        [self.buffer[p], self.buffer[p + 1], self.buffer[p + 2]]
    }

    #[inline]
    pub fn data<'a>(&'a self) -> rayon::slice::ChunksExact<'a, f64> {
        #[allow(clippy::chunks_exact_to_as_chunks)]
        self.buffer.par_chunks_exact(3)
    }

    #[inline]
    pub fn data_mut<'a>(&'a mut self) -> rayon::slice::ChunksExactMut<'a, f64> {
        #[allow(clippy::chunks_exact_to_as_chunks)]
        self.buffer.par_chunks_exact_mut(3)
    }

    #[inline]
    pub fn data_mut_vec(&mut self) -> &mut Vec<f64> {
        &mut self.buffer
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn grayscale_simple(&self) -> PixelArray {
        let buffer = self
            .data()
            .flat_map_iter(|p| {
                let gray = (p[0] + p[1] + p[2]) / 3.0;
                std::iter::repeat(gray).take(3)
            })
            .collect();

        PixelArray {
            width: self.width,
            height: self.height,
            buffer,
        }
    }

    pub fn grayscale_weighted(&self) -> PixelArray {
        let buffer = self
            .data()
            .flat_map_iter(|p| {
                let gray = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
                std::iter::repeat(gray).take(3)
            })
            .collect();

        PixelArray {
            width: self.width,
            height: self.height,
            buffer,
        }
    }

    pub fn sepia(&mut self) {
        self.data_mut().for_each(|p| {
            let (r, g, b) = (p[0], p[1], p[2]);

            p[0] = 0.393 * r + 0.769 * g + 0.189 * b;
            p[1] = 0.349 * r + 0.686 * g + 0.168 * b;
            p[2] = 0.272 * r + 0.534 * g + 0.131 * b;
        });
    }

    pub fn hist(&self) -> [[usize; 256]; 3] {
        let hist = self
            .data()
            .fold(
                || vec![[0usize; 256]; 3],
                |mut hist, pixel| {
                    for channel in 0..3 {
                        let value = pixel[channel].clamp(0.0, 1.0);
                        let bucket = (value * 255.0) as usize;
                        hist[channel][bucket] += 1;
                    }
                    hist
                },
            )
            .reduce(
                || vec![[0usize; 256]; 3],
                |mut a, b| {
                    for channel in 0..3 {
                        for i in 0..256 {
                            a[channel][i] += b[channel][i];
                        }
                    }
                    a
                },
            );

        hist.try_into().expect("histograma tem 3 canais")
    }

    pub fn rgb_to_hsi(&mut self) {
        self.apply(|pixel| {
            let r = pixel[0].clamp(0.0, 1.0);
            let g = pixel[1].clamp(0.0, 1.0);
            let b = pixel[2].clamp(0.0, 1.0);

            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let delta = max - min;

            // Matiz normalizada para [0, 1].
            let hue = if delta == 0.0 {
                0.0
            } else {
                let degrees = if max == r {
                    60.0 * ((g - b) / delta).rem_euclid(6.0)
                } else if max == g {
                    60.0 * ((b - r) / delta + 2.0)
                } else {
                    60.0 * ((r - g) / delta + 4.0)
                };

                degrees / 360.0
            };

            // Saturação: 0 quando a intensidade máxima é zero.
            let saturation = if max == 0.0 { 0.0 } else { delta / max };

            let intensity = (r + g + b) / 3.0;

            pixel[0] = hue;
            pixel[1] = saturation;
            pixel[2] = intensity;
        });
    }

    pub fn hist_plot_hsi(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.hist();

        let root = BitMapBackend::new("hist_plot_hsi.png", (640, 720)).into_drawing_area();
        root.fill(&WHITE)?;

        let areas = root.split_evenly((3, 1));

        let channels = [
            ("Hue (H)", &data[0], RED),
            ("Saturation (S)", &data[1], GREEN),
            ("Intensity (I)", &data[2], BLUE),
        ];

        for (area, (name, histogram, color)) in areas.iter().zip(channels) {
            let max_count = histogram.iter().copied().max().unwrap_or(0);

            let mut chart = ChartBuilder::on(area)
                .caption(name, ("sans-serif", 20).into_font())
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
                Histogram::vertical(&chart).style(color.filled()).data(
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

    pub fn hist_eq_hsi(&mut self) {
        let hist = self.hist();
        let intensity_hist = &hist[2];
        let size: usize = intensity_hist.iter().sum();

        if size == 0 {
            return;
        }

        let cdf: [f64; 256] = {
            let mut cumulative = 0usize;

            std::array::from_fn(|i| {
                cumulative += intensity_hist[i];
                cumulative as f64 / size as f64
            })
        };

        self.apply(|pixel| {
            let intensity = pixel[2].clamp(0.0, 1.0);
            let bucket = (intensity * 255.0) as usize;
            pixel[2] = cdf[bucket];
        });
    }
}

impl From<DynamicImage> for RGBPixelArray {
    fn from(value: DynamicImage) -> Self {
        let img = value.into_rgb8();
        Self {
            width: img.width(),
            height: img.height(),
            buffer: img.par_iter().map(|c| *c as f64 / 255.0).collect(),
        }
    }
}

impl Deref for RGBPixelArray {
    type Target = [f64];
    fn deref(&self) -> &Self::Target {
        self.buffer.deref()
    }
}

impl DerefMut for RGBPixelArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.deref_mut()
    }
}
