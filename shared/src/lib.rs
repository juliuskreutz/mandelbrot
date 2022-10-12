#![no_std]

pub use bytemuck;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub iterations: u32,
    _pad0: u32,
    pub zoom: f64,
    pub middle_x: f64,
    pub middle_y: f64,
}

impl Params {
    pub fn new(width: u32, height: u32, iterations: u32) -> Self {
        Self {
            width,
            height,
            iterations,
            _pad0: 0,
            zoom: 1.0,
            middle_x: -0.5,
            middle_y: 0.0,
        }
    }
}
