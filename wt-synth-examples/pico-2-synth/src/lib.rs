#![no_std]
#![allow(static_mut_refs)]
#![feature(f16)]
pub mod pll_settings;
pub mod vreg;

pub const HALF_U32: f32 = (u32::MAX / 2) as f32;
pub const HALF_U16: f32 = (u16::MAX / 2) as f32;
pub const FULL_U16: f32 = u16::MAX as f32;

pub fn get_u32_sample(sample: f32) -> u32 {
    // let sample = sample.unwrap_or(HALF_U32);
    // // let sample = (u32::MAX as f32 * sample) as u32;
    // //
    // // sample
    //
    // let normalized = (sample + 1.0) * HALF_U32;
    // let converted = normalized as u32;
    //
    // converted
    sample.to_bits()
}

pub fn get_u16_sample(sample: f32) -> u16 {
    // let sample = sample.unwrap_or(HALF_U32);
    // let sample = (u32::MAX as f32 * sample) as u32;
    //
    // sample

    let normalized = (sample * HALF_U16) + HALF_U16;
    // let normalized = (sample * FULL_U16).abs();
    // // let converted = normalized as u16;
    normalized as u16

    // converted
    // sample.to_bits()
    // (sample as f16).to_bits()
}
