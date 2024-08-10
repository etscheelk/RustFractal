use std::{cell::RefCell, sync::{atomic::AtomicU16, Arc, Mutex}};

use image::{GenericImage, ImageBuffer, Luma};
use RustFractal::fractal::Fractalize;

type GreyImageHigh = image::ImageBuffer<image::Luma<u16>, Vec<u16> >;

struct w(AtomicU16);



fn main() {
    println!("Hello, world!");

    // let img: image::ImageBuffer<image::Luma<u8>, Vec<u8> > = image::GrayImage::new(512, 512);
    // let mut img: image::ImageBuffer<image::Luma<u16>, Vec<u16> > = 
    
    let mut img: image::ImageBuffer<image::Luma<u8>, Vec<u8> > = 
        image::GrayImage::new(2048, 2048);


    // let a: Arc<RefCell<image::ImageBuffer<image::Luma<u8>, Vec<u8> > > > = Arc::new(RefCell::new(img));
    let mut a = Mutex::new(img.clone());

    // let mut img = 
    //     GreyImageHigh::new(128, 128);
    
        // image::ImageBuffer::new(128, 128);
    // img.put_pixel(1, 1, image::Luma([13]));

    // img.fractalize();

    let sub = img.sub_image(0, 0, img.width(), img.height() / 7);
    
    // let aa = AtomicU16::new(5);

    // aa.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // let mut img2: ImageBuffer<Luma<AtomicU16>, Vec<AtomicU16> > = ImageBuffer::new(512, 512);

    a.fractalize();


    // let _out = img.save("./test.png");

    // let b = &*a.clone();
    let _b = a.lock().unwrap().save("./test.png");
}

