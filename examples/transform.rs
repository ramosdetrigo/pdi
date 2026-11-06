use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = PixelArray::from_path(args.pop().unwrap()).unwrap();

    img.sobel();
    img.into_luma8().save("transformed.png").unwrap();
}
