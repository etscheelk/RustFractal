use std::{cell::RefCell, sync::{atomic::AtomicU8, Arc, Mutex}};

use rand::Rng;

use crate::fractal::{Fractalize, FractalizeParameters};

use super::MyGreyImage;

const MUTEX_CELL_LENGTH: u32 = 1024;

/// A grid of mutex guards of size 256. Either a strip 256 long or 
pub struct AtomicGrid
{
    grid: Vec<Arc<MutexCell>>,
    rows: u32,
    cols: u32,
    num_mutex_cells: u32,
}

struct MutexCell
{
    sub_grid: Mutex<[u8; MUTEX_CELL_LENGTH as usize]>,
    index: u32
}

impl Default for MutexCell
{
    fn default() -> Self {
        Self 
        { 
            sub_grid: Mutex::new([0; MUTEX_CELL_LENGTH as usize]), 
            index: Default::default(), 
        }
    }
}

impl MutexCell
{
    fn new(index: u32) -> Self
    {
        Self::default().with_index(index)
    }

    fn with_index(self, index: u32) -> Self
    {
        Self { index, ..self }
    }
}

impl AtomicGrid
{
    /// Creates a grid that must fit MutexCell blocks of length 256.
    pub fn new(rows: u32, cols: u32) -> Self
    {
        assert_eq!((rows * cols) % MUTEX_CELL_LENGTH, 0);

        let num_mutex_cells = (rows * cols) / MUTEX_CELL_LENGTH;

        let grid = 
            (0..num_mutex_cells)
            .map(|c| MutexCell::new(c).into())
            .collect();

        Self
        {
            grid,
            rows,
            cols,
            num_mutex_cells,
        }
    }
}

impl Fractalize for AtomicGrid
{
    fn fractalize(&mut self, p: FractalizeParameters) -> () {
        let (mut x, mut y) = p.init_x_y();
        let max_points = p.max_points();
        let rot = p.rot();
        let theta_offset = p.theta_offset();
        let method = p.method();
        
        let distr = rand::distributions::Uniform::new(0, usize::MAX);
        let rands: Vec<usize> = 
            rand::thread_rng()
            .sample_iter(distr)
            .take(max_points / 64)
            .collect();

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
                let theta = y * std::f64::consts::PI + theta_offset;
                (
                    rad * theta.cos(),
                    rad * theta.sin()
                )
            };

            (x, y)
        };

        let xy_to_grid_loc =
        move |x: f64, y: f64| -> (u32, u32)
        {
            let r = (y / 2.0 + 0.5) * rows as f64;
            let c = (x / 2.0 + 0.5) * cols as f64;

            return (r as u32, c as u32);
        };

        let flat_index =
        move |r: u32, c: u32|
        {
            (r * cols + c) as usize
        };

        // linear
        // for rand in rands
        // {
        //     for i in 0..64_u8
        //     {
        //         let this_rand = rand & (1 << i);
        //         (x, y) = transform(x, y, this_rand == 0);
        //         let (r, c) = xy_to_grid_loc(x, y);
        //         let index = flat_index(r, c);
        //         let mc_index = index / MUTEX_CELL_LENGTH as usize;
        //         let internal_index = index % MUTEX_CELL_LENGTH as usize;
                
        //         self.grid[mc_index].sub_grid.lock().unwrap()[internal_index] += 1;
        //     }
        // }

        // parallel chunks
        const NUM_THREADS: usize = 12;
        let chunk_size = self.num_mutex_cells as usize / NUM_THREADS;
        let slice = self.grid.as_slice();        
        std::thread::scope(
        |scope|
        {
            slice
            .chunks_exact(chunk_size)
            .enumerate()
            .for_each(
            |(i, sub_slice)|
            {
                let rands_slice = rands.as_slice();
                scope.spawn(
                move ||
                {
                    // let valid_range = (i * chunk_size * MUTEX_CELL_LENGTH as usize)..=((i+1) * chunk_size * MUTEX_CELL_LENGTH as usize);
                    let valid_range = (i * chunk_size)..((i+1) * chunk_size);
                    println!("range for thread {i}: {:?}", valid_range);
                    for rand in rands_slice
                    {
                        for i in 0..64_u8
                        {
                            let this_rand: bool = (rand & (1 << i)) == 0;
                            (x, y) = transform(x, y, this_rand);
                            let (r, c) = xy_to_grid_loc(x, y);
                            let index = flat_index(r, c);
                            let mc_index = index / MUTEX_CELL_LENGTH as usize;
                            if !valid_range.contains(&mc_index) { continue }
                            let internal_index = index % MUTEX_CELL_LENGTH as usize;

                            // sub_slice[mc_index - valid_range.start].sub_grid.lock().unwrap()[internal_index] += 1;
                            if let Some(p) = sub_slice.get(mc_index - valid_range.start)
                            {
                                p.sub_grid.lock().unwrap()[internal_index] += 1;
                            }
                        }
                    }
                });
            });
        });

    }
}

impl From<AtomicGrid> for MyGreyImage<u8>
{
    fn from(value: AtomicGrid) -> Self {
        // let vec: Vec<u8> =
        //     value.grid
        //     .into_vec()
        //     .into_iter()
        //     .map(
        //     |a| 
        //         a.into_inner()
        //     ).collect();

        // MyGreyImage::from_raw(value.cols, value.rows, vec).unwrap()

        let vec: Vec<u8> = 
            value.grid
            .into_iter()
            .flat_map(
            |arcmc|
            {
                *arcmc.sub_grid.lock().unwrap()
            })
            .collect();

        MyGreyImage::from_raw(value.cols, value.rows, vec).unwrap()
    }
}


#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn atomic_grid_impl_send()
    {
        let ag = AtomicGrid::new(100, 100);

        let mut _is_send: &dyn Send = &ag.grid;
        let mut _is_sync: &dyn Sync = &ag.grid;
    }

    // #[test]
    // fn ensure_grid_empty()
    // {
    //     let ag = AtomicGrid::new(100, 100);
    //     for elem in ag.grid
    //     {
    //         assert_eq!(elem.into_inner(), 0);   
    //     }
    // }
}