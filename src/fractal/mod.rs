use std::{f32::consts::PI, sync::{Mutex, MutexGuard}};
use derive_setters::*;
use derive_getters::*;
use image::Pixel;

use cubecl::CubeType;

pub trait Index2D<Idx, Idy>
where
    Idx: ?Sized,
    Idy: ?Sized,
{
    type Output: ?Sized;

    fn index_2d(&self, x: Idx, y: Idy) -> Result<&Self::Output, Index2DError>;
}

pub trait IndexMut2D<Idx, Idy>: Index2D<Idx, Idy>
where
    Idx: ?Sized,
    Idy: ?Sized,
{
    fn index_mut_2d(&mut self, x: Idx, y: Idy) -> Result<&mut Self::Output, Index2DError>;
}


#[derive(Debug)]
pub enum Index2DError
where
    Self: Sized
{
    IndexOutOfBounds(String),
}

pub trait Fractalize
{
    fn fractalize(&mut self, p: FractalizeParameters) -> ();
}

// pub trait AsyncFractalize: Fractalize
// {
//     fn async_fractalize(&mut self, p: FractalizeParameters) -> impl std::future::Future<Output = ()> + Send + Sync
//     {
//         // todo!()
//         let a: Box<std::future::IntoFuture>;
//         self.fractalize(p)
//     }
// }

#[derive(Setters, Getters, Clone, Copy, CubeType, Debug, PartialEq)]
#[setters(prefix = "with_")]
#[getter(prefix = "get_")]
pub struct FractalizeParameters
{
    // #[setters(skip)]
    pub init_x_y: (f32, f32),
    pub rot: f32,
    pub theta_offset: f32,
    pub method: FractalMethod,
    pub max_points: u32,
}

#[derive(Default, Clone, Copy, CubeType, Debug, PartialEq, Eq)]
pub enum FractalMethod
{
    #[default]
    Default,
    MultiplyTheta,
}

impl Default for FractalizeParameters
{
    fn default() -> Self {
        Self 
        { 
            init_x_y: (0.0, 0.5), 
            rot: 1.724643921305295,
            theta_offset: 3.0466792337230033,
            method: FractalMethod::default(),
            max_points: 1_000_000
        }
    }
}

pub struct Image
{
    x: usize,
    y: usize,
    img: Vec<usize>
}

impl std::fmt::Display for Image
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dimensions of image: x: {}, y: {}\n", self.x, self.y)?;

        for y in 0..self.y
        {
            for x in 0..self.x
            {
                write!(f, "{} ", self.img[y * self.x + x])?;
            }
            write!(f, "\n")?;
        }

        write!(f, "")
    }
}

impl<P> Fractalize for image::ImageBuffer<image::Luma<P>, Vec<P> >
where
    P: image::Primitive + num_traits::CheckedAdd,
{
    fn fractalize(&mut self, p: FractalizeParameters) -> () 
    {   
        let (mut x, mut y) = p.init_x_y();
        let rot = p.rot();
        let theta_offset = p.theta_offset();
        let _ = p.method();
        let max_points = p.max_points();

        for _ in 0..max_points
        {
            let this_rand = rand::random::<u64>();

            if this_rand & 1 == 1
            {
                (x, y) = (
                    x * rot.cos() + y * rot.sin(),
                    y * rot.cos() - x * rot.sin()
                )
            }
            else
            {
                let rad = x * 0.5 + 0.5;
                let theta = y * PI + theta_offset;

                x = rad * theta.cos();
                y = rad * theta.sin();
            }

            // add point to array
            // assumes square right now
            let xx = (x / 2.0 + 0.5) * self.width() as f32;
            let yy = (y / 2.0 + 0.5) * self.height() as f32;

            if let Some(pixel) = self.get_pixel_mut_checked(xx as u32, yy as u32)
            {
                pixel.apply(
                    |p: P| -> P 
                    {
                        match p.checked_add(&P::one())
                        {
                            Some(pp) => pp,
                            None => p,
                        }
                    }
                )
            }
        }
    }
}

impl<P> Fractalize for Mutex<image::ImageBuffer<image::Luma<P>, Vec<P> > >
where
    P: image::Primitive + num_traits::CheckedAdd 
{
    fn fractalize(&mut self, params: FractalizeParameters) -> () 
    {
        let FractalizeParameters 
        { 
            init_x_y: (mut x, mut y), 
            rot, 
            theta_offset, 
            method: _, 
            max_points 
        } = params;
        
        for _ in 0..max_points
        {
            let this_rand = rand::random::<u64>();

            if this_rand & 1 == 1
            {
                (x, y) = (
                    x * rot.cos() + y * rot.sin(),
                    y * rot.cos() - x * rot.sin()
                )
            }
            else
            {
                let rad = x * 0.5 + 0.5;
                let theta = y * PI + theta_offset;

                x = rad * theta.cos();
                y = rad * theta.sin();
            }

            // add point to array
            // assumes square right now

            let mut img: MutexGuard<image::ImageBuffer<_, _> > = self.lock().unwrap();

            let xx = (x / 2.0 + 0.5) * img.width() as f32;
            let yy = (y / 2.0 + 0.5) * img.height() as f32;

            if let Some(pixel) = img.get_pixel_mut_checked(xx as u32, yy as u32)
            {
                pixel.apply(
                    |p: P| -> P 
                    {
                        match p.checked_add(&P::one())
                        {
                            Some(p) => p,
                            None => p
                        }
                    }
                )
            }
        }
    }
}

// impl<IMG> Fractalize for Image<IMG>
// where
//     IMG: IndexMut2D<usize, usize> + IndexMut<usize>,
// {
impl Image
{
    pub fn new(x: usize, y: usize) -> Image
    {
        let mut v = Vec::new();
        v.resize(x * y, 0);
        Image { x, y, img: v}
    }

    pub fn fractalize(&mut self) -> ()
    {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.5;

        let rot: f32 = 1.724643921305295;
        let theta_offset: f32 = 3.0466792337230033;
        let num_pts = 10_000_000_usize;

        // let mut rng = rand::thread_rng();
        
        for _ in 0..num_pts
        {
            let this_rand = rand::random::<u64>();

            if this_rand & 1 == 1
            {
                (x, y) = (
                    x * rot.cos() + y * rot.sin(),
                    y * rot.cos() - x * rot.sin()
                )
            }
            else
            {
                let rad: f32 = x * 0.5 + 0.5;
                let theta = y * PI + theta_offset;

                x = rad * theta.cos();
                y = rad * theta.sin();
            }

            // add point to array
            // assumes square right now
            let xx = (x / 2.0 + 0.5) * self.x as f32;
            let yy = (y / 2.0 + 0.5) * self.x as f32;

            // println!("row: {}\ncol: {}\nindex: {}", yy as usize, xx as usize, (yy as usize) * self.x + (xx as usize));

            // let r = self.img.get_mut((yy as usize) * self.x + (xx as usize)).unwrap();
            if let Some(r) = self.img.get_mut((yy as usize) * self.x + (xx as usize))
            {
                *r += 1;
            }
            // *r += 1;
        }

        self.img.iter_mut().for_each(|p| *p = (*p as f64).ln() as usize);
    }
}