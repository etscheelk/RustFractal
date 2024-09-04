use std::{f64::consts::PI, io::Write, path::Path};

use rand::prelude::*;

pub struct MutexGrid<P>
{
    width: u32,
    height: u32,
    grid: Vec<P>
}

impl<P> MutexGrid<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    pub fn new(width: u32, height: u32) -> Self
    {
        MutexGrid
        {
            width,
            height,
            grid: vec![P::default(); (width * height) as usize]
        }
    }
    
    pub fn apply_all_in_parallel<F>(&mut self, threads: u16, mut f: F)
    where
        F: FnMut(&mut P) -> P + Send + Sync + Copy,
        P: Send + Sync
    {
        let chunk_size = ((self.grid.len() as f64) / (threads as f64)).ceil() as usize;
        let slice = self.grid.as_mut_slice();

        crossbeam::scope(|scope| {
            slice
            .chunks_mut(chunk_size)
            .for_each(
                |sub_slice: &mut [P]| -> ()
                {
                    scope.spawn(
                        move |_|
                        {
                            sub_slice
                            .iter_mut()
                            .for_each(|at_elem| *at_elem = f(at_elem))
                        }
                    );
                }
            )
        })
        .expect("Some thread panicked");
    }

    pub fn r#static(&mut self) -> () 
    where
        P: num_traits::CheckedAdd
    {
        let distr = 
            rand::distributions::Uniform::new(0, self.width);
        let mut rng = rand::thread_rng();
        for _ in 0..1_000_000_000
        {
            let x = distr.sample(&mut rng);
            let y = distr.sample(&mut rng);

            let a = self.grid.get_mut((y * self.width + x) as usize).unwrap();

            *a = match P::checked_add(a, &P::one())
            {
                Some(v) => v,
                None => *a
            }
        }
    }
}

impl<T> crate::fractal::Fractalize for MutexGrid<T>
where
    T: image::Primitive + num_traits::CheckedAdd,
{
    fn fractalize(&mut self, num_points: usize) -> () {
        let distr = 
            rand::distributions::Uniform::new(0, self.width);
        let mut rng = rand::thread_rng();

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.5;
        
        let rot: f64 = 1.724643921305295;
        let theta_offset: f64 = 3.0466792337230033;

        for _ in 0..num_points
        {
            let this_rand = distr.sample(&mut rng);

            // (x, y) = match this_rand & 1
            // {
            //     1 => (
            //         x * rot.cos() + y * rot.sin(),
            //         y * rot.cos() - x * rot.sin()
            //     ),
            //     _ => {
            //         let rad = x * 0.5 + 0.5;
            //         let theta = y * PI + theta_offset;
            //         (
            //             rad * theta.cos(),
            //             rad * theta.sin()
            //         )
            //     }
            // };

            // Check whether these are the same. 
            // I believe this case would be identical
            (x, y) = 
            if this_rand & 1 == 1
            {
                (
                    x * rot.cos() + y * rot.sin(),
                    y * rot.cos() - x * rot.sin()
                )
            }
            else
            {
                let rad = x * 0.5 + 0.5;
                let theta = y * PI + theta_offset;
                (
                    rad * theta.cos(),
                    rad * theta.sin()
                )
            };

            // add point to array
            // assumes square right now
            let xx = (x / 2.0 + 0.5) * self.width as f64;
            let yy = (y / 2.0 + 0.5) * self.height as f64;

            if let Some(pixel) = self.grid.get_mut(yy as usize * self.width as usize + xx as usize)
            {
                *pixel = match pixel.checked_add(&T::one())
                {
                    Some(v) => v,
                    None => *pixel
                }
            }
        }
    }
}

struct MutexGridRefMut<'a, T: image::Primitive>
{
    // width: usize,
    // height: usize, // assume 48x48 for now
    grid: std::sync::Mutex<&'a mut [T]>
}

// impl<P> Into<image::ImageBuffer<P, Vec<P> > > for MutexGrid<P>
// where
//     P: image::Pixel<Subpixel: image::Primitive>
// {
//     fn into(self) -> image::ImageBuffer<P, Vec<P> > {
//         let a = (&*self.grid).to_vec();
//         image::ImageBuffer::from_raw(self.width as u32, self.height as u32, a).unwrap()
//     }
// }

pub type MyGreyImage<P> = image::ImageBuffer<image::Luma<P>, Vec<P>>;

/// Conversion to a grey image
impl<P> Into<MyGreyImage<P>> for MutexGrid<P>
where
    P: image::Primitive
{ 
    fn into(self) -> MyGreyImage<P> {
        // from_raw fails if the buffer is not large enough.
        // But we know the buffer will have the right size so it will not fail 
        MyGreyImage::from_raw(self.width, self.height, self.grid).unwrap()
    }
}

// impl<P> Into<MyGreyImage<P>> for &MutexGrid<P>
// where
//     P: image::Primitive
// {
//     fn into(self) -> MyGreyImage<P> {
//         MyGreyImage::from_raw(self.width, self.height, self.grid.clone()).unwrap()
//     }
// }



mod test
{
    #[ignore = "Don't want this to run every time"]
    #[test]
    fn main()
    {
        use crate::fractal::Fractalize;
        let mut img = super::MutexGrid::<u8>::new(1024, 1024);
        img.fractalize(10_000_000);

        let img: super::MyGreyImage<_> = img.into();
        let _ = img.save("image_a.png");
    }

    #[test]
    fn slice_chunks_even()
    {
        let b = Vec::from_iter(1..=20);
        let mut it = (&*b).chunks(5);
        
        assert_eq!(Some(&[1, 2, 3, 4, 5][..]),      it.next());
        assert_eq!(Some(&[6, 7, 8, 9, 10][..]),     it.next());
        assert_eq!(Some(&[11, 12, 13, 14, 15][..]), it.next());
        assert_eq!(Some(&[16, 17, 18, 19, 20][..]), it.next());
        
        assert_eq!(None, it.next());
    }

    #[test]
    fn image_send_sync()
    {
        let m = super::MutexGrid::<u16>::new(48, 48);

        let v: &dyn Send = &m;
        let v: &dyn Sync = &m;

        let img = image::GrayImage::new(128, 128);

        let v: &dyn Send = &img;
        let v: &dyn Sync = &img;
    }

    #[test]
    fn image_buffer_from_arc_buf()
    {
        use crate::mutex_grid::{MutexGrid, MyGreyImage};

        let img = 
            image::ImageBuffer::<image::Luma<u16>, std::sync::Arc<_> >::from_raw(
                128, 
                128, 
                vec![0_u16; 128*128].into()
            );
        assert_ne!(img, None);

        let img: MutexGrid<u16> = MutexGrid::<u16>::new(128, 128);
        let img: MyGreyImage<u16> = img.into();
    }
}