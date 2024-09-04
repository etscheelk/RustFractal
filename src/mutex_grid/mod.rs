use std::{f64::consts::PI, io::Write, ops::{Add, Deref, DerefMut}, path::Path, thread};

use rand::prelude::*;

pub struct MutexGrid<P>
{
    rows: u32,
    cols: u32,
    grid: Vec<P>
}

impl<P> MutexGrid<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    pub fn new(rows: u32, cols: u32) -> Self
    {
        MutexGrid
        {
            rows,
            cols,
            grid: vec![P::default(); (rows * cols) as usize]
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
            rand::distributions::Uniform::new(0, self.rows);
        let mut rng = rand::thread_rng();
        for _ in 0..1_000_000_000
        {
            let x = distr.sample(&mut rng);
            let y = distr.sample(&mut rng);

            let a = self.grid.get_mut((y * self.rows + x) as usize).unwrap();

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
    fn fractalize(&mut self, num_points: usize) -> () 
    {
        let distr = 
            rand::distributions::Uniform::new(0, self.rows);
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
            let xx = (x / 2.0 + 0.5) * self.rows as f64;
            let yy = (y / 2.0 + 0.5) * self.cols as f64;

            if let Some(pixel) = self.grid.get_mut(yy as usize * self.rows as usize + xx as usize)
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

pub struct MutexGridPar<P>(MutexGrid<P>);

impl<P> Deref for MutexGridPar<P>
{
    type Target = MutexGrid<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> DerefMut for MutexGridPar<P>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P> MutexGridPar<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    pub fn new(rows: u32, cols: u32) -> Self
    {
        MutexGridPar(MutexGrid::new(rows, cols))
    }
}


impl crate::fractal::Fractalize for MutexGridPar<u8>
// where
//     P: image::Primitive + num_traits::CheckedAdd + Add + Send,
{
    fn fractalize(&mut self, num_points: usize) -> () 
    {
        // Strategy: Create a sparse matrix and use that in each thread
        // Upon thread join, add the matrices, convert 

        let num_threads = 4;
        let matrix_size = (self.rows as usize, self.cols as usize);

        let mut handles = vec![];

        for i in 0..num_threads
        {
            let handle = thread::spawn(
                move ||
                {
                    println!("Hello fromm thread #{i}");
                    ////////////////////////////////////////////////
                    let mut local_matrix: sprs::CsMat<u8> = 
                        sprs::CsMatBase::zero(matrix_size);
                    
                    // assumes square
                    let distr = rand::distributions::Uniform::new(0, matrix_size.0);
                    let mut rng = rand::thread_rng();

                    let mut x: f64 = 0.0;
                    let mut y: f64 = 0.5;

                    let rot: f64 = 1.724643921305295;
                    let theta_offset: f64 = 3.0466792337230033;

                    for ii in 0..(num_points / num_threads)
                    {
                        if ii % 100_000 == 0 { println!("{ii} in thread {i}"); }
                        let this_rand = distr.sample(&mut rng);

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

                        let xx = (x / 2.0 + 0.5) * matrix_size.0 as f64;
                        let yy = (y / 2.0 + 0.5) * matrix_size.1 as f64;

                        // TODO: Checked add?
                        match local_matrix.get_mut(xx as usize, yy as usize)
                        {
                            Some(value) => {
                                if let Some(v) = value.checked_add(1)
                                {
                                    *value = v;
                                }
                            },
                            None => {
                                local_matrix.insert(xx as usize, yy as usize, 1);
                            },
                        }
                    }

                    local_matrix
                    ////////////////////////////////////////////////
                }
            );
            handles.push(handle);
        }

        let mut final_matrix: sprs::CsMat<u8> = 
            sprs::CsMatBase::zero(matrix_size);
        
        for handle in handles
        {
            let local_matrix = handle.join().unwrap();

            // final_matrix = final_matrix + local_matrix;
            final_matrix = &final_matrix + &local_matrix;
        }

        // read sparse matrix data into self.grid
        let indptr = final_matrix.indptr();
        let indices = final_matrix.indices();
        let data = final_matrix.data();

        // let row = 1;
        // let z = &indices[indptr.index(row)..indptr.index(row+1)];

        for row in 0..(matrix_size.0)
        {
            let (ind_a, ind_b) = (indptr.index(row), indptr.index(row+1));
            for (col, val) in indices[ind_a..ind_b].iter().zip(data[ind_a..ind_b].iter())
            {
                self.grid[row * matrix_size.1 as usize + col] = *val;
            }
        }
    }
}

impl<P> Into<MyGreyImage<P>> for MutexGridPar<P>
where
    P: image::Primitive
{ 
    fn into(self) -> MyGreyImage<P> {
        // from_raw fails if the buffer is not large enough.
        // But we know the buffer will have the right size so it will not fail 
        MyGreyImage::from_raw(self.rows, self.cols, self.grid.clone()).unwrap()
    }
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
        MyGreyImage::from_raw(self.rows, self.cols, self.grid).unwrap()
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