#![allow(unused_imports)]
use image::open;
use pdi::pixel_array::PixelArray;
use pdi::rgb_pixel_array::RGBPixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let img = open(args.pop().unwrap()).unwrap().into_rgb8();
    let mut img2 = RGBPixelArray::from_path(args.pop().unwrap()).unwrap();

    img2.chroma_key(0.5, &img);
    img2.into_rgb().save("transformed.png").unwrap();
}
