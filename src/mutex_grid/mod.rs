use std::io::Write;

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

    pub fn apply_in_parallel<F>(&mut self, threads: u16, mut f: F)
    where
        F: FnMut(&mut P) -> P + Send + Sync + Copy,
        P: Send + Sync
    {
        let chunk_size = ((self.grid.len() as f64) / (threads as f64)).ceil() as usize;
        let slice = self.grid.as_mut_slice();

        let r = crossbeam::scope(|scope| {
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
}

impl<T> crate::fractal::Fractalize for MutexGrid<T>
where
    T: image::Primitive + num_traits::CheckedAdd,
{
    fn fractalize(&mut self) -> () {
        let distr = 
            rand::distributions::Uniform::new(0, self.width);
        let mut rng = rand::thread_rng();
        for _ in 0..10_000_000
        {
            let x = distr.sample(&mut rng);
            let y = distr.sample(&mut rng);

            // println!("{}x{}, x: {}, y: {}", self.width, self.height, x, y);

            let a = self.grid.get_mut((y * self.width + x) as usize).unwrap();
            // *a = *a + T::one();

            *a = match T::checked_add(a, &T::one())
            {
                Some(v) => v,
                None => *a
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
        println!("length of grid buffer: {}", self.grid.len());
        let _ = std::io::stdout().flush();

        MyGreyImage::from_raw(self.width, self.height, self.grid).unwrap()
    }
}



mod test
{
    #[ignore = "Don't want this to run every time"]
    #[test]
    fn main()
    {
        use crate::fractal::Fractalize;
        let mut img = super::MutexGrid::<u8>::new(1024, 1024);
        img.fractalize();

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