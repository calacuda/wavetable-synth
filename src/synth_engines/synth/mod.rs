use crate::{config::WAVE_TABLE_SIZE, WaveTable};

pub const N_OVERTONES_SAW: usize = 16;

pub mod osc;

pub fn build_sine_table(overtones: &[f64]) -> WaveTable {
    let mut wave_table = [0.0; WAVE_TABLE_SIZE];

    let n_overtones = overtones.len();

    let bias = 1.0 / (n_overtones as f32 * 0.5);

    for i in 0..WAVE_TABLE_SIZE {
        for ot in overtones {
            wave_table[i] +=
                (2.0 * core::f64::consts::PI * i as f64 * ot / WAVE_TABLE_SIZE as f64).sin() as f32
        }

        wave_table[i] *= bias;
    }

    wave_table.into()
}
