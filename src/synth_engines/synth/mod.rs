use super::synth_common::{WaveTable, WAVE_TABLE_SIZE};
use std::fmt::Debug;

pub const N_OVERTONES_SAW: usize = 16;

pub mod osc;
// mod saw_tooth;
// pub mod synth;

// trait SynthOscilatorBackend: Debug + SampleGen {
//     fn set_frequency(&mut self, frequency: f32);
//     // fn set_type(&mut self, osc_type: OscType);
//     fn sync_reset(&mut self);
// }
//
// #[derive(Debug, Clone)]
// pub enum SynthBackend {
//     Sin(WavetableOscillator),
//     Saw(SawToothOsc),
// }
//
// impl From<OscType> for SynthBackend {
//     fn from(value: OscType) -> Self {
//         match value {
//             OscType::Sin => Self::Sin(WavetableOscillator::new()),
//             OscType::Saw => Self::Saw(SawToothOsc::new()),
//         }
//     }
// }
//
// impl SampleGen for SynthBackend {
//     fn get_sample(&mut self) -> f32 {
//         match self {
//             Self::Sin(osc) => osc.get_sample(),
//             Self::Saw(osc) => osc.get_sample(),
//         }
//     }
// }
//
// impl SynthOscilatorBackend for SynthBackend {
//     fn set_frequency(&mut self, frequency: f32) {
//         match self {
//             Self::Sin(osc) => osc.set_frequency(frequency),
//             Self::Saw(osc) => osc.set_frequency(frequency),
//         }
//     }
//
//     fn sync_reset(&mut self) {
//         match self {
//             Self::Sin(osc) => osc.sync_reset(),
//             Self::Saw(osc) => osc.sync_reset(),
//         }
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OscType {
    Sin,
    // Tri,
    // Sqr,
    Saw,
}

impl From<usize> for OscType {
    fn from(value: usize) -> Self {
        match value {
            _ if value == Self::Sin as usize => Self::Sin,
            // _ if value == Self::Tri as usize => Self::Tri,
            // _ if value == Self::Sqr as usize => Self::Sqr,
            _ if value == Self::Saw as usize => Self::Saw,
            _ => Self::Saw,
        }
    }
}

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
