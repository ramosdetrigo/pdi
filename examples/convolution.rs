use pdi::pixel_array::PixelArray;
use pdi::transforms::*;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut img = PixelArray::from_path(args.pop().unwrap()).unwrap();
    let filter1 = vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0];
    // let filter2 = vec![
    //     0.111, 0.111, 0.111, 0.111, 0.111, 0.111, 0.111, 0.111, 0.111,
    // ];
    img.gen_convolution(3, filter1);
    let og = img.into_luma8();
    og.save("og.png").unwrap();
}
