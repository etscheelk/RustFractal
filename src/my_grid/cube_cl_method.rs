use cubecl::prelude::*;
use rand::Rng;

use crate::fractal::FractalizeParameters;

// #[derive(Debug, Clone, CubeType)]
// struct CubeStruct
// {
//     pixels: Vec<u8>,
//     dims: (u32, u32),
// }

// #[cube(launch_unchecked)]
// fn cube_struct_fractalize<
//     AtomicU: Int, 
//     RandU: Int
// >

type AtomicType = u32;
type RandType = u32;

#[cube(launch_unchecked)]
fn cube_struct_fractalize<F: Float>
(
    pixels: &mut Array<Atomic<AtomicType>>, 
    rand_nums: Array<RandType>,
)
{
    let mut x = F::new(0.0_f32);
    let mut y = F::new(0.5_f32);
    let rot = F::new(1.72464392_f32);
    let rot_cos = F::cos(rot);
    let rot_sin = F::sin(rot);

    
    // let rot_cos = comptime! { F::new(F::cos(rot)) };
    // let rot_sin = comptime! { F::sin(rot) };
    // let theta_offset = 3.0466792337230033_f32;


    // let mut rot = 1.7246f32.sqrt();
    
    
    // let mut y: F = 0.5;
    
    
    
    // let rot: F = 1.724643921305295.into();
    // let theta_offset: f32 = 3.0466792337230033.into();

    for index in 0..rand_nums.len()
    {
        let this_rand = rand_nums[index];

        for i in 0..64_u32
        {
            let r = this_rand & (1 << i);
            
            if r == 0
            {
                x = x * rot_cos + y * rot_sin;
                y = y * rot_cos - x * rot_sin;
            }
            else
            {
                let rad = x * F::new(0.5_f32) + F::new(0.5_f32);
                let theta = y * F::new(3.14159_f32) + F::new(3.0466792337230033_f32);
                // let rad = F::new(x * 0.5 + 0.5);
                // let theta = F::new(y * 3.14159 + theta_offset);

                // x = rad * F::cos(theta);

                x = rad * F::cos(theta);
                y = rad * F::sin(theta);

                // x = rad * theta.cos();
                // y = rad * theta.sin();
            }

            // convert to row and column grid location from floats
            // let (r, c) =
            // {
            //     let r = x * F::new(0.5_f32) + F::new(0.5_f32);
            //     let c = x * F::new(0.5_f32) + F::new(0.5_f32);
            let a = pixels[0];
            a.add(1);
                
            // };
        }
    }
}

#[cube]
fn xy_to_grid_loc(x: f32, y: f32) -> (f32, f32)
{
    // FIXME: hardcoded for 1024x1024
    let r = (y * 0.5 + 0.5) * 1024.0 as f32;
    let c = (x * 0.5 + 0.5) * 1024.0 as f32;

    // unsafe {
    //     (r.to_int_unchecked(), c.to_int_unchecked())
    // }
    (r, c)
}

fn flat_index(r: u32, c: u32) -> u32
{
    r * 1024 + c
}

fn transform(x: f32, y: f32, s: bool, rot_cos: f32, rot_sin: f32, theta_offset: f32) -> (f32, f32)
{
    let (x, y) = 
    if s
    {
        (
            x * rot_cos + y * rot_sin,
            y * rot_cos - x * rot_sin,
        )
    }
    else
    {
        let rad = x * 0.5 + 0.5;
        let theta = y * 3.14159 + theta_offset;

        (
            rad * theta.cos(),
            rad * theta.sin(),
        )
    };

    (x, y)
}

pub fn launch<R: Runtime>(device: &R::Device)
{
    let client = R::client(device);

    let num_pixels = 1024 * 1024;

    let points = 50_000_000;

    let distr = rand::distributions::Uniform::new(RandType::MIN, RandType::MAX);
    let rands = 
        rand::thread_rng()
        .sample_iter(&distr)
        .take(points / RandType::BITS as usize)
        .collect::<Vec<RandType>>();
    
    let rand_nums_handle = client.create(RandType::as_bytes(&rands));
    let pixels_handle = client.empty(num_pixels * core::mem::size_of::<u32>());

    let vectorization = 1;

    unsafe
    {
        cube_struct_fractalize::launch_unchecked::<f32, R>(
            &client, 
            CubeCount::Static(1, 1, 1), 
            // CubeDim::new(rands.len() as u32 / vectorization, 1, 1), 
            CubeDim::new_1d(128),
            ArrayArg::<R>::from_raw_parts::<u32>(&pixels_handle, num_pixels, vectorization as u8), 
            ArrayArg::<R>::from_raw_parts::<RandType>(&rand_nums_handle, rands.len(), vectorization as u8),
        );
    }
}