#![allow(unused_imports)]
use pdi::pixel_array::PixelArray;
use pdi::rgb_pixel_array::RGBPixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = RGBPixelArray::from_path(args.pop().unwrap()).unwrap();

    img.sepia();
    img.into_rgb().save("transformed.png").unwrap();
}
