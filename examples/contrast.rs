use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = PixelArray::from_path(args.pop().unwrap()).unwrap();
    img.contrast_stretching([0.3, 0.1], [0.8, 0.9]);
    let og = img.into_luma8();
    og.save("og.png").unwrap();
}
