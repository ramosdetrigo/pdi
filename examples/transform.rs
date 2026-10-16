use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = PixelArray::from_path(args.pop().unwrap()).unwrap();
    img.hide("Hello, World!");

    let decoded_msg = img.read();
    println!("{decoded_msg}");

    let og = img.clone().into_luma8();
    og.save("og.png").unwrap();
    img.into_luma8().save("hidden.png").unwrap();
}
