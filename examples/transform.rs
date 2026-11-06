use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = PixelArray::from_path(args.pop().unwrap()).unwrap();

    img.rotation(45.0_f64.to_radians(), 450, 800, Interpolation::Bilinear);
    img.into_luma8().save("transformed.png").unwrap();
}
