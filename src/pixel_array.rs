use std::{
    error::Error,
    ops::{Deref, DerefMut},
    path::Path,
};

use image::{DynamicImage, GrayImage, ImageReader};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone, Debug, PartialEq)]
pub struct PixelArray {
    width: u32,
    height: u32,
    buffer: Vec<f64>,
}

impl PixelArray {
    pub fn from_path<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        Ok(ImageReader::open(path)?.decode()?.into())
    }

    pub fn into_luma8(self) -> GrayImage {
        GrayImage::from_vec(
            self.width,
            self.height,
            self.buffer
                .into_iter()
                .map(|p| (p.clamp(0.0, 1.0) * 255.0) as u8)
                .collect(),
        )
        .unwrap()
    }

    pub fn data(&self) -> &[f64] {
        &self.buffer
    }

    pub fn data_mut(&mut self) -> &mut [f64] {
        &mut self.buffer
    }
}

impl From<DynamicImage> for PixelArray {
    fn from(value: DynamicImage) -> Self {
        let img = value.into_luma8();
        Self {
            width: img.width(),
            height: img.height(),
            buffer: img.par_iter().map(|p| *p as f64 / 255.0).collect(),
        }
    }
}

impl Deref for PixelArray {
    type Target = [f64];
    fn deref(&self) -> &Self::Target {
        self.buffer.deref()
    }
}

impl DerefMut for PixelArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.deref_mut()
    }
}
