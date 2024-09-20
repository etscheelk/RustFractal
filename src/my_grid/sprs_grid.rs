use std::f64::consts::PI;

use rand::prelude::*;

use crate::fractal::FractalizeParameters;

use super::MyGrid;

// FAR TOO SLOW
impl crate::fractal::Fractalize for sprs::CsMat<u8>
{
    fn fractalize(&mut self, p: FractalizeParameters) -> () 
    {
        let distr = 
            rand::distributions::Uniform::new(0, self.rows());
        let mut rng = rand::thread_rng();
        let (mut x, mut y) = p.init_x_y();
        let max_points = p.max_points();
        let rot = p.rot();
        let theta_offset = p.theta_offset();
        let method = p.method();

        for _ in 0..max_points
        {
            let this_rand = distr.sample(&mut rng);

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
            let xx = (x / 2.0 + 0.5) * self.rows() as f64;
            let yy = (y / 2.0 + 0.5) * self.cols() as f64;

            match self.get_mut(xx as usize, yy as usize)
            {
                Some(value) => {
                    if let Some(v) = value.checked_add(1)
                    {
                        *value = v;
                    }
                },
                None => {
                    self.insert(xx as usize, yy as usize, 1);
                },
            }
        }
    }
}

impl From<sprs::CsMat<u8>> for MyGrid<u8>
{
    fn from(value: sprs::CsMat<u8>) -> Self {
        // read sparse matrix data into self.grid
        let indptr = value.indptr();
        let indices = value.indices();
        let data = value.data();

        let mut grid: Vec<u8> = vec![0; value.rows() * value.cols()];
        for row in 0..(value.rows())
        {
            // as described in the documentation
            // https://docs.rs/sprs/latest/sprs/struct.CsMatBase.html#indices-and-data
            let (ind_a, ind_b) = (indptr.index(row), indptr.index(row+1));
            for (col, val) in indices[ind_a..ind_b].iter().zip(data[ind_a..ind_b].iter())
            {
                grid[row * value.cols() as usize + col] = *val;
            }
        }

        MyGrid { rows: value.rows(), cols: value.cols(), grid }
    }
}