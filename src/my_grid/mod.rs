pub mod sprs_grid;

use std::{f64::consts::PI, ops::{Deref, DerefMut}, thread};

use image::flat;
use rand::prelude::*;

pub struct MyGrid<P>
{
    rows: usize,
    cols: usize,
    grid: Vec<P>
}

impl<P> MyGrid<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    pub fn new(rows: usize, cols: usize) -> Self
    {
        MyGrid
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
        for _ in 0..1_000_000
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

impl<T> crate::fractal::Fractalize for MyGrid<T>
where
    T: image::Primitive + num_traits::CheckedAdd + Send,
{
    fn fractalize(&mut self, num_points: usize) -> () 
    {
        let distr = 
            rand::distributions::Uniform::new(0, usize::MAX);
        let rands: Vec<usize> = rand::thread_rng().sample_iter(&distr).take(num_points / 64).collect();

        let mut x: f64 = 0.5;
        let mut y: f64 = 0.5;
        
        let rot: f64 = 1.724643921305295;
        let theta_offset: f64 = 3.0466792337230033;

        let rows = self.rows;
        let cols = self.cols;

        let transform = 
        move |x: f64, y: f64, s: bool| -> (f64, f64)
        {
            let (x, y) = 
            if s
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

            (x, y)
        };

        let xy_to_grid_loc =
        move |x, y| -> (usize, usize)
        {
            let r = (y / 2.0 + 0.5) * rows as f64;
            let c = (x / 2.0 + 0.5) * cols as f64;

            return (r as usize, c as usize);
        };

        let flat_index =
        move |r: usize, c: usize|
        {
            r * cols + c
        };

        let _mpsc_ver_1 = 
        ||
        {
            let (sx, rx) = std::sync::mpsc::channel::<usize>();

            for i in 0..4
            {
                let angle = 2.0 * PI / 4_f64 * i as f64;
                // let (mut x, mut y) = 0.5 * (2.0 * PI / 4 as f64)
                // let (mut x, mut y) = (0.75 * angle.cos(), 0.1 * angle.sin());
                let (mut x, mut y) = (0.5, 0.5);
            
    
                let sxi = sx.clone();
                let handle = std::thread::spawn(
                move ||
                {
                    let mut rng = thread_rng();
                    for _ in 0..(num_points/4)
                    {
                        let b = (rng.sample(distr) & 1) == 0;
                        (x, y) = transform(x, y, b);
                        let (r, c) = xy_to_grid_loc(x, y);
                        let _ = sxi.send(flat_index(r, c));
                        // println!("Sending... {i}");
                        // println!("{r}");
                    }
                    drop(sxi);
                });
            }
            drop(sx);
    
            // let mut i = 0;
            while let Ok(ind) = rx.recv()
            {
                // println!("\tReceiving...");
                // println!("{i}");
                // i += 1;
                if let Some(pixel) = self.grid.get_mut(ind)
                {
                    *pixel = match pixel.checked_add(&T::one())
                    {
                        Some(v) => v,
                        None => *pixel
                    }
                }
            }
        };

        // attempts to use scoped threads to distribute rands
        // not a good idea. Blocks on scope so receiver waits
        let _mpsc_ver_2 =
        ||
        {
            let (sx, rx) = std::sync::mpsc::channel::<usize>();

            let num_threads = 4;
    
            let slice = rands.as_slice();
            let chunk_size_exact = rands.len() / num_threads;
            std::thread::scope(
            |scope|
            {
                slice
                .chunks_exact(chunk_size_exact)
                .enumerate()
                .for_each(
                |(i, sub_slice)|
                {
                    let sxi = sx.clone();
                    scope.spawn(
                    move ||
                    {
                        let angle = 2.0 * PI / (num_threads as f64) * (i as f64);
                        let (mut x, mut y) = (0.5 * angle.cos(), 0.5 * angle.sin());
                        for this_rand in sub_slice
                        {
                            for i in 0..64
                            {
                                let this_bool: bool = (this_rand & (1 << i)) != 0;
                                
                                (x, y) = transform(x, y, this_bool);
                                let (r, c) = xy_to_grid_loc(x, y);
                                let _ = sxi.send(flat_index(r, c));
                            }
                        }
                    });
                })
            });
            drop(sx);

            while let Ok(ind) = rx.recv()
            {
                if let Some(pixel) = self.grid.get_mut(ind)
                {
                    *pixel = match pixel.checked_add(&T::one())
                    {
                        Some(v) => v,
                        None => *pixel
                    }
                }
            }
        };

        // every chunk looks at all rands and checks if the output x-y val is in the valid range
        // Technically some redundant effort but Paul did this for cache optimization reasons.
        let mut _par_chunks = 
        ||
        {
            // assumes right now the number of rows is divisible by four
            let chunk_exact_size = self.rows * self.cols / 4;
            let slice = self.grid.as_mut_slice();
            std::thread::scope(
            |scope|
            {
                // chunk by some number of whole rows
                slice
                .chunks_exact_mut(chunk_exact_size)
                .enumerate()
                .for_each(
                |(en, sub_slice)|
                {
                    // every chunk looks at all rands and checks if the output x-y val is in the valid range
                    // Technically some redundant effort but Paul did this for cache optimization reasons.
                    let rrr = &rands;
                    scope.spawn(
                    move ||
                    {
                        let valid_indices = (en * chunk_exact_size)..((en+1) * chunk_exact_size);
                        for r in rrr
                        {
                            for i in 0..64_usize
                            {
                                let b: bool = (r & (1 << i)) != 0;
                                (x, y) = transform(x, y, b);
                                let (r, c) = xy_to_grid_loc(x, y);
    
                                let index = flat_index(r, c);
    
                                if !valid_indices.contains(&index) { continue }
    
                                let translated_index = index - valid_indices.start;
    
                                sub_slice[translated_index] = sub_slice[translated_index] + T::one();
                            }
                        }
                    });
                })
            });
        };

        // no parallelization
        let _default =
        ||
        {
            for r in rands
            {
                for i in 0..64_usize
                {
                    let this_r = r & (1 << i);
            
                    (x, y) = transform(x, y, this_r != 0);
        
                    let (r, c) = xy_to_grid_loc(x, y);
        
                    if let Some(pixel) = self.grid.get_mut(flat_index(r, c))
                    {
                        *pixel = match pixel.checked_add(&T::one())
                        {
                            Some(v) => v,
                            None => *pixel
                        }
                    }
                }
            }
        };

        _default();
    }
}

pub struct MyGridPar<P>(MyGrid<P>);

impl<P> Deref for MyGridPar<P>
{
    type Target = MyGrid<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> DerefMut for MyGridPar<P>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P> MyGridPar<P>
where
    P: image::Primitive + Default
{
    /// Create an all-black single-color image
    /// of dimensions width x height
    pub fn new(rows: usize, cols: usize) -> Self
    {
        MyGridPar(MyGrid::new(rows, cols))
    }
}

impl<P> From<MyGridPar<P>> for MyGrid<P>
{
    fn from(value: MyGridPar<P>) -> Self {
        value.0
    }
}

impl<P> Into<MyGreyImage<P>> for MyGridPar<P>
where
    P: image::Primitive
{
    fn into(self) -> MyGreyImage<P> {
        self.0.into()
    }
}

impl crate::fractal::Fractalize for MyGridPar<u8>
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

        for _ in 0..num_threads
        {
            let handle = thread::spawn(
                move ||
                {
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

                    for _ in 0..(num_points / num_threads)
                    {
                        // if ii % 100_000 == 0 { println!("{ii} in thread {i}"); }
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

pub type MyGreyImage<P> = image::ImageBuffer<image::Luma<P>, Vec<P>>;

/// Conversion to a grey image
impl<P> Into<MyGreyImage<P>> for MyGrid<P>
where
    P: image::Primitive
{ 
    fn into(self) -> MyGreyImage<P> {
        // from_raw fails if the buffer is not large enough.
        // But we know the buffer will have the right size so it will not fail 
        MyGreyImage::from_raw(self.rows as u32, self.cols as u32, self.grid).unwrap()
    }
}


#[cfg(test)]
mod test
{
    #[ignore = "Don't want this to run every time"]
    #[test]
    fn main()
    {
        use crate::fractal::Fractalize;
        let mut img = super::MyGrid::<u8>::new(1024, 1024);
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
        let m = super::MyGrid::<u16>::new(48, 48);

        let _v: &dyn Send = &m;
        let _v: &dyn Sync = &m;

        let img = image::GrayImage::new(128, 128);

        let _v: &dyn Send = &img;
        let _v: &dyn Sync = &img;
    }

    #[test]
    fn image_buffer_from_arc_buf()
    {
        use crate::my_grid::{MyGrid, MyGreyImage};

        let img = 
            image::ImageBuffer::<image::Luma<u16>, std::sync::Arc<_> >::from_raw(
                128, 
                128, 
                vec![0_u16; 128*128].into()
            );
        assert_ne!(img, None);

        let img: MyGrid<u16> = MyGrid::<u16>::new(128, 128);
        let _img: MyGreyImage<u16> = img.into();
    }
}