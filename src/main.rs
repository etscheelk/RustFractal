use std::{cell::RefCell, sync::{atomic::AtomicU16, Arc, Mutex}};

use image::{GenericImage, ImageBuffer, Luma};
use RustFractal::{fractal::Fractalize, mutex_grid::{MutexGrid, MyGreyImage}};

type GreyImageHigh = image::ImageBuffer<image::Luma<u16>, Vec<u16> >;

struct w(AtomicU16);

fn test_a()
{
    let mut img: image::ImageBuffer<image::Luma<u8>, Vec<u8> > = 
        image::GrayImage::new(2048, 2048);

    img.fractalize();

    let _ = img.save("test_a.png");
}

fn mutex_grid_test_b()
{
    let mut img = 
        MutexGrid::<u8>::new(512, 512);
    
    img.fractalize();

    img.apply_in_parallel(4, |p| *p * *p);
    img.apply_in_parallel(4, |p| (*p).pow(8));


    let img: MyGreyImage<_> = img.into();
    let _ = img.save("mutex_grid_rand.png");
}

fn main() {
    println!("Hello, world!");

    mutex_grid_test_b();
}

