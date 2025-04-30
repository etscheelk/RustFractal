use cubecl::prelude::*;

pub mod sqrt
{
    use super::*;

    #[cube(launch_unchecked)]
    fn apply_on_array<F: Float>(input: &mut Array<Line<F>>)
    {
        if ABSOLUTE_POS < input.len()
        {
            input[ABSOLUTE_POS] = div_by_sqrt_2(input[ABSOLUTE_POS]);       
        }
    }


    #[cube]
    fn div_by_sqrt_2<F: Float>(x: Line<F>) -> Line<F>
    {
        let sqrt_2 = F::new(comptime!{2.0_f32.sqrt()});

        x / Line::new(sqrt_2)
    }

    pub fn launch<R: Runtime>(device: &R::Device, input: &[f32]) -> Vec<f32>
    {
        let client = R::client(device);

        let vectorization = 4;
        let io_handle = client.create(f32::as_bytes(input));

        unsafe
        {
            let cube_cnt_x = input.len() as f32  / vectorization as f32 / 256_f32;
            let cube_cnt_x = cube_cnt_x.ceil() as u32;

            apply_on_array::launch_unchecked::<f32, R>(
                &client,
                CubeCount::Static(cube_cnt_x, 1, 1),
                CubeDim::new(256, 1, 1),
                ArrayArg::from_raw_parts::<f32>(&io_handle, input.len(), vectorization as u8),
            )
        }

        let bytes = client.read_one(io_handle.binding());
        let output = f32::from_bytes(&bytes).to_vec();

        output
    }
}