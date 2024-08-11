use std::{cell::RefCell, sync::{atomic::AtomicU16, Arc, Mutex}, time::Instant};

use image::{GenericImage, ImageBuffer, Luma};
use serde::{Deserialize, Serialize};
use RustFractal::{fractal::Fractalize, mutex_grid::{MutexGrid, MyGreyImage}};

type GreyImageHigh = image::ImageBuffer<image::Luma<u16>, Vec<u16> >;



struct w(AtomicU16);

fn test_a()
{
    let mut img: image::ImageBuffer<image::Luma<u8>, Vec<u8> > = 
        image::GrayImage::new(2048, 2048);

    img.fractalize(10_000_000);

    let _ = img.save("test_a.png");
}

fn mutex_grid_static_b()
{
    let mut img = 
        MutexGrid::<u8>::new(2048, 2048);
    
    img.r#static();

    // img.apply_in_parallel(4, |p| *p * *p);
    // img.apply_in_parallel(4, |p| (*p).pow(8));


    let img: MyGreyImage<_> = img.into();
    let _ = img.save("mutex_grid_rand.png");
}

fn mutex_grid_fractal_c(dim: u32, num_points: usize) -> f64
{
    let start = Instant::now();

    let mut img = 
        MutexGrid::<u8>::new(dim, dim);

    img.fractalize(num_points);

    let img: MyGreyImage<_> = img.into();
    let _ = img.save("mutex_grid_fractal.png");
    // let _ = img.save("mutex_grid_fractal.bmp");

    let dur = start.elapsed().as_secs_f64();
    dur
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Row<T>
where
    T: std::fmt::Debug
{
    name: String,
    elems: Vec<T>,
}

impl<T> Row<T>
where
    T: std::fmt::Debug
{
    fn init(name: String) -> Self
    {
        Self { name, elems: vec![] }
    }

    fn add_elem(&mut self, elem: T) -> ()
    {
        self.elems.push(elem);
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct Table<T>
where
    T: std::fmt::Debug
{
    name: String,
    col_names: Vec<String>,
    rows: Vec<Row<T>>,
}

// impl<T> std::fmt::Display for Table<T>
// where
//     T: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}\n", self.name)?;
//         for col_name in &self.col_names
//         {
//             write!(f, "{}\t", col_name)?;
//         } writeln!(f)?;

//         write!(f, "")
//     }
// }

fn main() {
    println!("Hello, world!");

    // test();

    // let mut img = MutexGrid::<u8>::new(4096, 4096);
    // img.fractalize(1_000_000_000);
    // img.apply_all_in_parallel(4, 
    //     |p|
    //     (*p as f64).sqrt() as u8
    // );

    // let img: MyGreyImage<_> = img.into();
    // let _ = img.save("sqrtimg.png");

    let a = Arc::new(RefCell::new(vec![0; 2]));
    let mut aa = a.as_ref().borrow_mut();
    let mut aaa = a.as_ref().borrow_mut();

    aa[0] = 1;
    aaa[1] = 2;
    
    println!("{:?}", a);
}

fn test() {
    let mut t = Table::<f64>::default();
    t.name = "SerialMethod8192x8192".to_string();
    
    const DIM: u32 = 8192;
    const START: usize = 1_000_000;
    const END: usize = 1_000_000_000;
    const NUM_PTS_TESTS: usize = 10;
    const NUM_REPS: usize = 5;
    
    let mut cur_pts = START;

    // Find log-gap between start and end and then divide by the number of tests
    let mult = ((END as f64).log10() - (START as f64).log10()) / (NUM_PTS_TESTS as f64);
    
    println!("log diff: {mult}");

    // The correct number to multiply by each time is 10^mult.
    let mult = 10_f64.powf(mult);

    println!("mult: {mult}");

    for _ in 0..=NUM_PTS_TESTS
    {
        println!("points: {cur_pts}");

        let mut r = Row::<f64>::init(format!("{cur_pts}"));

        for _ in 0..NUM_REPS
        {
            let time = mutex_grid_fractal_c(DIM, cur_pts);
            r.add_elem(time);
        }
    
        t.rows.push(r);

        cur_pts = ((cur_pts as f64) * mult).ceil() as usize;
    }

    println!("{:#?}", t);
    println!("{:?}", postcard::to_stdvec(&t).unwrap());
}

