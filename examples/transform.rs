use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let img = PixelArray::from_path(args.pop().unwrap()).unwrap();
    let hidden = img.hide("Hello, World!");

    let decoded_msg = hidden.read();
    println!("{decoded_msg}");

    let og = img.into_luma8();
    og.save("og.png").unwrap();
    hidden.into_luma8().save("hidden.png").unwrap();
}
