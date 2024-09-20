use std::sync::atomic::AtomicU8;

use rand::Rng;

use crate::fractal::{Fractalize, FractalizeParameters};

use super::MyGreyImage;

pub struct AtomicGrid
{
    grid: Box<[AtomicU8]>,
    rows: u32,
    cols: u32,
}

impl AtomicGrid
{
    pub fn new(rows: u32, cols: u32) -> Self
    {
        let grid = Self::create_blank_grid(rows, cols);
        
        Self
        {
            grid,
            rows,
            cols
        }
    }

    pub fn clear(&mut self)
    {
        self.grid = Self::create_blank_grid(self.rows, self.cols);
    }

    fn create_blank_grid(rows: u32, cols: u32) -> Box<[AtomicU8]>
    {
        (0..(rows*cols))
        .map(
        |_| 
        {
            AtomicU8::new(0)
        })
        .collect()
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

        for rand in rands
        {
            for i in 0..64_u8
            {
                let this_rand = rand & (1 << i);

                (x, y) = transform(x, y, this_rand == 0);

                let (r, c) = xy_to_grid_loc(x, y);

                if let Some(p) = self.grid.get_mut(flat_index(r, c))
                {
                    p.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                }
            }   
        }
    }
}

impl From<AtomicGrid> for MyGreyImage<u8>
{
    fn from(value: AtomicGrid) -> Self {
        let vec: Vec<u8> =
            value.grid
            .into_vec()
            .into_iter()
            .map(
            |a| 
                a.into_inner()
            ).collect();

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

        let _is_send: &dyn Send = &ag;
    }

    #[test]
    fn ensure_grid_empty()
    {
        let ag = AtomicGrid::new(100, 100);
        for elem in ag.grid
        {
            assert_eq!(elem.into_inner(), 0);   
        }
    }
}