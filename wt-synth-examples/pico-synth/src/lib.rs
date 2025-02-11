#![no_std]

pub mod pll_settings;
pub mod vreg;

pub const HALF_U32: f32 = (u32::MAX / 2) as f32;

pub fn get_u32_sample(sample: Option<f32>) -> u32 {
    let sample = sample.unwrap_or(0.0);
    // let sample = (u32::MAX as f32 * sample) as u32;
    //
    // sample

    let normalized = (sample + 1.0) * HALF_U32;
    let converted = normalized as u32;

    converted
}
