use std::time::Instant;

use serde::{Deserialize, Serialize};
use RustFractal::{fractal::{Fractalize, FractalizeParameters}, my_grid::{atomic_grid::AtomicGrid, MyGreyImage, MyGrid, MyGridPar}};

fn time_and_save(dim: usize, num_points: usize) -> f64
{
    let start = Instant::now();

    let mut img = 
        MyGrid::<u8>::new(dim, dim);

    img.fractalize(FractalizeParameters::default().with_max_points(num_points));

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
    let start = Instant::now();

    println!("Hello, world!");

    // test();
    let p = FractalizeParameters::default().with_max_points(1_000_000_000);


    // let mut img = MyGrid::<u8>::new(4096, 4096);
    // let mut img = crate::my_grid::atomic_grid::AtomicGrid::new();
    let mut img = AtomicGrid::new(4096, 4096);
    println!("time to create grid: {} seconds", start.elapsed().as_secs_f64());
    let start = Instant::now();
    img.fractalize(p);
    println!("Time to fractalize: {} seconds", start.elapsed().as_secs_f64());
    let start = Instant::now();
    let img: MyGreyImage<u8> = img.into();
    println!("time to into MyGreyImage: {} seconds", start.elapsed().as_secs_f64());
    let start = Instant::now();
    let _ = img.save("improved_rand.png");
    println!("time to save png: {} seconds", start.elapsed().as_secs_f64());

    // let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    // let slice = v.as_mut_slice();
    // slice.chunks_mut(5).enumerate().for_each(|a| { println!("i: {}, {:?}", a.0, a.1) });
    

    // Must contain references (pointers) so the array 
    // has elements of known size at compile time
    // let mut a : Vec<&dyn std::fmt::Debug> = vec![];
    // a.push(&5_i32);
    // a.push(&1.0_f32);
    // a.push(&"hello there!");
    // println!("{:?}", a);

    // let mut img = MutexGridPar::<u8>::new(4096, 4096);
    // let mut img = MyGrid::<u8>::new(1024, 1024);
    // // let mut img = MyGridPar::<u8>::new(1024, 1024);
    // // let mut img: sprs::CsMat<u8> = sprs::CsMatBase::zero((1024, 1024));
    // // let vec: sprs::CsVec<u8> = sprs::CsVecBase::zero();
    // img.fractalize(1_000_000);
    
    // // let img: MyGrid<_> = img.into();
    // let img : MyGreyImage<_> = img.into();
    // let _ = img.save("test_par.png");

    // let mut img = MutexGrid::<u8>::new(4096, 4096);
    // img.fractalize(1_000_000_000);
    // img.apply_all_in_parallel(4, 
    //     |p|
    //     (*p as f64).sqrt() as u8
    // );

    // let img: MyGreyImage<_> = img.into();
    // let _ = img.save("sqrtimg.png");

    // let a = Arc::new(RefCell::new(vec![0; 2]));
    // let mut aa = a.as_ref().borrow_mut();
    // let mut aaa = a.as_ref().borrow_mut();

    // aa[0] = 1;
    // aaa[1] = 2;
    
    // println!("{:?}", a);
    // println!("Elapsed time: {}", start.elapsed().as_secs_f64());
}

fn test() {
    let mut t = Table::<f64>::default();
    
    const NAME: &str = "TrivialSerial";
    const DIM: usize = 4096;
    const START: usize = 1_000_000;
    const END: usize = 1_000_000_000;
    const NUM_PTS_TESTS: usize = 10;
    const NUM_REPS: usize = 5;
    
    // t.name = "SerialMethod8192x8192".to_string();
    t.name = format!("{NAME}_{DIM}x{DIM}_{START}_to_{END}_in_{NUM_PTS_TESTS}_steps_{NUM_REPS}_reps_each");
    
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
            let time = time_and_save(DIM, cur_pts);
            r.add_elem(time);
        }
    
        t.rows.push(r);

        cur_pts = ((cur_pts as f64) * mult).ceil() as usize;
    }

    println!("{:#?}", t);
    println!("{:?}", postcard::to_stdvec(&t).unwrap());
}

#[cfg(test)]
mod test
{
    use RustFractal::{fractal::{Fractalize, FractalizeParameters}, my_grid::{MyGreyImage, MyGrid}};

    #[test]
    fn test_basic() -> Result<(), image::ImageError>
    {
        let mut img = MyGrid::<u8>::new(512, 512);
        img.fractalize(FractalizeParameters::default().with_max_points(1_000_000));
        let img: MyGreyImage<_> = img.into();
        img.save("test/test_basic.png")
    }

    #[test]
    fn mutex_grid_random_static() -> Result<(), image::ImageError>
    {
        let mut img = 
            MyGrid::<u8>::new(256, 256);
        
        img.r#static();

        let img: MyGreyImage<_> = img.into();
        img.save("test/static_noise.png")
    }

    #[test]
    fn sprs_grid_fractalize() -> Result<(), image::ImageError>
    {
        let mut s: sprs::CsMat<u8> = sprs::CsMatBase::zero((512, 512));
        s.fractalize(FractalizeParameters::default().with_max_points(1_000_000));

        let s: MyGrid<u8> = s.into();
        let s: MyGreyImage<u8> = s.into();
        s.save("test/sprs_grid_fractalize.png")
    }
}