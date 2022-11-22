#![cfg_attr(target_arch = "spirv", no_std)]
#![deny(warnings)]

use shared::Params;
use spirv_std::macros::spirv;

use spirv_std::glam::*;
use spirv_std::num_traits::Float;

#[spirv(vertex)]
pub fn main_vs(position: Vec2, #[spirv(position, invariant)] out: &mut Vec4) {
    *out = vec4(position.x, position.y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(position)] position: Vec4,
    #[spirv(push_constant)] params: &Params,
    output: &mut Vec4,
) {
    let x = params.middle_x
        + ((position.x as f64 - params.width as f64 / 2.) / params.width as f64 * 3.) * params.zoom;

    let y = params.middle_y
        + ((position.y as f64 - params.height as f64 / 2.) / params.height as f64 * 2.)
            * params.zoom;

    let pixel = pixel(dvec2(x, y), params.iterations);

    let r = pixel as f32 / params.iterations as f32;
    let g = (pixel * pixel) as f32 / (params.iterations * params.iterations) as f32;
    let b = Float::sqrt(pixel as f32) / Float::sqrt(params.iterations as f32);

    *output = vec4(r, g, b, 1.0);
}

fn square(n: DVec2) -> DVec2 {
    dvec2(n.x * n.x - n.y * n.y, 2. * n.x * n.y)
}

fn pixel(coord: DVec2, iterations: u32) -> u32 {
    let mut z = dvec2(0., 0.);

    for i in 0..iterations {
        z = square(z) + coord;
        if z.length() > 2. {
            return i;
        }
    }

    0
}
