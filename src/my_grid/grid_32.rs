use std::{f32::consts::PI, ops::{Index, IndexMut}};

use rand::Rng;

use crate::fractal::Fractalize;

use super::Grid;

#[derive(Clone, Debug)]
pub struct Grid32
{
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<u32>,
}

impl<Idx> Index<Idx> for Grid32
where 
    Idx: Into<(usize, usize)>
{
    type Output = u32;

    fn index(&self, index: Idx) -> &Self::Output 
    {
        let (r, c) = index.into();
        
        &self.grid[self.flat_index(r, c)]
    }
}

impl<Idx> IndexMut<Idx> for Grid32 
where
    Idx: Into<(usize, usize)>
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output 
    {
        let (r, c) = index.into();

        &mut self.grid[r * self.cols + c]
    }
}

impl super::Grid for Grid32
{
    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }
}

impl Grid32
{
    pub fn new(rows: usize, cols: usize) -> Self
    {
        Grid32 { rows, cols, grid: vec![0; rows * cols] }
    }
}

pub type MyColorImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

impl From<Grid32> for MyColorImage
{
    fn from(value: Grid32) -> Self 
    {
        let buf: Vec<u8> = 
        value.grid
        .into_iter()
        .flat_map(
        |u| 
            [((u >> 24) & 0xFF) as u8, ((u >> 16) & 0xFF) as u8, ((u >> 8) & 0xFF) as u8, (0xFF) as u8]
        )
        .collect();

        let i = MyColorImage::from_vec(value.cols as u32, value.rows as u32, buf)
        .expect("into worked");

        

        i
    }
}

impl Fractalize for MyColorImage
{
    fn fractalize(&mut self, p: crate::fractal::FractalizeParameters) -> () 
    {
        let (mut x, mut y) = p.init_x_y();
        let max_points = p.max_points();
        
        let rot = p.rot();
        let rot_cos = rot.cos();
        let rot_sin = rot.sin();

        let theta_offset = p.theta_offset();

        let _method = *p.method();

        let distr = 
            rand::distributions::Uniform::new(0, usize::MAX);
        let rands: Vec<usize> = rand::thread_rng().sample_iter(&distr).take((max_points / 64) as usize).collect();


        let rows = self.height();
        let cols = self.width();

        let transform = 
        move |x: f32, y: f32, s: bool|
        {
            let (x, y) = 
            if s
            {
                (
                    x * rot_cos + y * rot_sin,
                    y * rot_cos - x * rot_sin
                )
            }
            else
            {
                let rad = x * 0.5 + 0.5;
                // let theta: f32 = y * PI + theta_offset;

                use crate::fractal::FractalMethod::*;
                let theta: f32 = match _method
                {
                    Default => y * PI + theta_offset,
                    MultiplyTheta => y * PI * theta_offset,
                };
                (
                    rad * theta.cos(),
                    rad * theta.sin()
                )
            };

            (x, y)
        };

        let xy_to_grid_loc =
        move |x: f32, y: f32| -> (u32, u32)
        {
            let r: f32 = (y * 0.5 + 0.5) * rows as f32;
            let c: f32 = (x * 0.5 + 0.5) * cols as f32;

            unsafe {
                (r.to_int_unchecked(), c.to_int_unchecked())
            }
        };

        let _do_both_transformations =
        ||
        {
            for rr in rands
            {
                for i in 0..64_usize
                {
                    let this_r = rr & (1 << i);

                    // first
                    let (xx, yy) = transform(x, y, this_r == 0);
                    let (r, c) = xy_to_grid_loc(xx, yy);
                    if let Some(p) = self.get_pixel_mut_checked(c, r)
                    {
                        p[0] += 1;
                        p[1] += 1;
                        p[2] += 1;
                    }

                    // second
                    let (xx, yy) = transform(x, y, this_r != 0);
                    let (r, c) = xy_to_grid_loc(xx, yy);
                    if let Some(p) = self.get_pixel_mut_checked(c, r)
                    {
                        p[0] += 1;
                        p[1] += 1;
                        p[2] += 1;
                    }

                    (x, y) = (xx, yy);
                }
            }
        };
        
        _do_both_transformations();
    }
}

impl crate::fractal::Fractalize for Grid32
{
    fn fractalize(&mut self, p: crate::fractal::FractalizeParameters) -> () 
    {
        let (mut x, mut y) = p.init_x_y();
        let max_points = p.max_points();
        
        let rot = p.rot();
        let rot_cos = rot.cos();
        let rot_sin = rot.sin();

        let theta_offset = p.theta_offset();

        let _method = *p.method();

        let distr = 
            rand::distributions::Uniform::new(0, usize::MAX);
        let rands: Vec<usize> = rand::thread_rng().sample_iter(&distr).take((max_points / 64) as usize).collect();


        let rows = self.rows;
        let cols = self.cols;

        let grey_one = (1_u32 << 8) | (1_u32 << 16) | (1_u32 << 24);

        let transform = 
        move |x, y, s: bool|
        {
            let (x, y) = 
            if s
            {
                (
                    x * rot_cos + y * rot_sin,
                    y * rot_cos - x * rot_sin
                )
            }
            else
            {
                let rad = x * 0.5 + 0.5;
                // let theta: f32 = y * PI + theta_offset;

                use crate::fractal::FractalMethod::*;
                let theta: f32 = match _method
                {
                    Default => y * PI + theta_offset,
                    MultiplyTheta => y * PI * theta_offset,
                };
                (
                    rad * theta.cos(),
                    rad * theta.sin()
                )
            };

            (x, y)
        };

        let xy_to_grid_loc =
        move |x: f32, y: f32| -> (usize, usize)
        {
            let r: f32 = (y * 0.5 + 0.5) * rows as f32;
            let c: f32 = (x * 0.5 + 0.5) * cols as f32;

            unsafe {
                (r.to_int_unchecked(), c.to_int_unchecked())
            }
        };

        let flat_index =
        move |r: usize, c: usize|
        {
            r * cols + c
        };

        let _do_both_transformations =
        ||
        {
            for rr in rands
            {
                for i in 0..64_usize
                {
                    let this_r = rr & (1 << i);

                    // first
                    let (xx, yy) = transform(x, y, this_r == 0);
                    let (r, c) = xy_to_grid_loc(xx, yy);
                    if let Some(pixel) = self.grid.get_mut(flat_index(r, c))
                    // let pixel = &mut self.grid[flat_index(r,c)];
                    {
                        // *pixel |= 0xFF;
                        *pixel = match pixel.checked_add(grey_one)
                        {
                            Some(v) => v,
                            None => *pixel,
                        };
                        // *pixel += T::one();
                    }

                    // second
                    let (xx, yy) = transform(x, y, this_r != 0);
                    let (r, c) = xy_to_grid_loc(xx, yy);
                    if let Some(pixel) = self.grid.get_mut(flat_index(r, c))
                    // let pixel = &mut self.grid[flat_index(r,c)];
                    {
                        // *pixel |= 0xFF;
                        *pixel = match pixel.checked_add(grey_one)
                        {
                            Some(v) => v,
                            None => *pixel,
                        };
                        // *pixel += T::one();
                    }

                    (x, y) = (xx, yy);
                }
            }
        };

        
        _do_both_transformations();
    }
}