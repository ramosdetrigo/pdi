use std::{
    error::Error,
    ops::{Deref, DerefMut},
    path::Path,
    slice::{ChunksExact, ChunksExactMut},
};

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
