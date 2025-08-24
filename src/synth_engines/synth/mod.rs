use crate::{
    config::{OSC_WAVE_TABLE_SIZE, SAMPLE_RATE},
    sin, tanh, OscWaveTable,
};
use biquad::{
    Biquad, Coefficients, DirectForm1, DirectForm2Transposed, ToHertz, Q_BUTTERWORTH_F32,
};

pub const N_OVERTONES_SAW: usize = 4;

pub mod osc;

pub fn build_sine_table(overtones: &[f64]) -> OscWaveTable {
    let f0 = 440.hz();
    let fs = SAMPLE_RATE.hz();

    // info!("{oscs:?}");
    let coeffs =
        Coefficients::<f32>::from_params(biquad::Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();

    let mut filter = DirectForm2Transposed::<f32>::new(coeffs);

    let mut wave_table = [0.0; OSC_WAVE_TABLE_SIZE];

    let n_overtones = overtones.len();

    let bias = 1.0 / (n_overtones as f32 * 0.5);

    for i in 0..OSC_WAVE_TABLE_SIZE {
        for over_tone in overtones {
            wave_table[i] +=
                sin(2.0 * core::f64::consts::PI * i as f64 * over_tone / OSC_WAVE_TABLE_SIZE as f64)
                    as f32
        }

        // wave_table[i] *= bias;
        wave_table[i] = filter.run(wave_table[i] * bias);
    }

    wave_table.into_iter().collect()
}
