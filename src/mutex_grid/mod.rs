use rand::prelude::*;

struct MutexGrid<P>
{
    width: u32,
    height: u32,
    grid: std::sync::Arc<[P]>
}

impl<P> MutexGrid<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    fn new(width: u32, height: u32) -> Self
    {
        MutexGrid
        {
            width,
            height,
            grid: vec![P::default(); (width * height) as usize].into()
        }
    }
}

impl<T> crate::fractal::Fractalize for MutexGrid<T>
where
    T: image::Primitive,
{
    fn fractalize(&mut self) -> () {
        let distr = 
            rand::distributions::Uniform::new_inclusive(0, self.width);
        let mut rng = rand::thread_rng();
        for _ in 0..10_000_000
        {
            let x = distr.sample(&mut rng);
            let y = distr.sample(&mut rng);

            
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

type MyGreyImage<P> = image::ImageBuffer<image::Luma<P>, std::sync::Arc<[P]>>;

/// Conversion to a grey image
impl<P> Into<MyGreyImage<P>> for MutexGrid<P>
where
    P: image::Primitive
{ 
    fn into(self) -> MyGreyImage<P> {
        // from_raw fails if the buffer is not large enough.
        // But we know the buffer will have the right size so it will not fail 
        MyGreyImage::from_raw(self.width as u32, self.height as u32, self.grid).unwrap()
    }
}



mod test
{
    // use crate::mutex_grid::{MutexGrid, MyGreyImage};


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
        // let m = MutexGrid::<u16>::new(48, 48);

        // let v: &dyn Send = &m;
        // let v: &dyn Sync = &m;

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