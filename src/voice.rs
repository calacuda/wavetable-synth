#[cfg(feature = "embeded")]
use crate::alloc::borrow::ToOwned;
use crate::{
    common::{DataTable, ModMatrixDest},
    config::{N_ENV, N_LFO, N_OSC, SAMPLE_RATE},
    effects::{chorus::Chorus, /* reverb::Reverb, */ Effect, EffectsModule},
    lfo::LFO,
    midi_to_freq,
    synth_engines::{
        synth::osc::{OscTarget, Oscillator},
        synth_common::{biquad_filter::BQLowPass, env::ADSR, moog_filter::LowPass},
    },
    ModMatrix, ModulationDest, OscWaveTable, SampleGen,
};
use array_macro::array;
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Q_BUTTERWORTH_F32};
use core::ops::IndexMut;
use log::*;

// const U32: f32 = u32::MAX as f32 * 0.5;

// #[macro_export]
// macro_rules! array {
//     [$expr:expr; 0] => {
//         []
//     };
//     // [$expr:expr; $count:expr, false] =>
//     // };
//     [$expr:expr; $count:expr] => {{
//         // let value = $expr;
//         // $crate::array![_ => $crate::__core::clone::Clone::clone(&value); $count]
//         [ $crate::array![$expr; $count - 1, false], $expr ]
//     }};
//     // [$i:pat => $e:expr; $count:expr] => {
//     //     $crate::__array![$i => $e; $count]
//     // };
// }

#[derive(Clone, Debug)]
pub struct Voice {
    /// Oscilators holds the osc and if its playing
    pub oscs: [(Oscillator, bool); N_OSC],
    /// env filters
    pub envs: [ADSR; N_ENV],
    /// LFOs
    pub lfos: [LFO; N_LFO],
    /// filters
    pub filters: [LowPass; 2],
    // pub filters: [BQLowPass; 2],
    /// what notes this voice is playing
    pub playing: Option<u8>,
    /// effects, holds the effect and if its one or not
    pub effects: [(EffectsModule, bool); 1],
    /// holds the out put of the different modules and also other needed data (velocity, and note).
    data_table: DataTable,
    /// describes how loud the synth is
    pub level: f32,
    /// stores the modulation amount for level
    level_mod: f32,
    /// used to stop clipping.
    all_pass: DirectForm1<f32>,
}

impl Voice {
    #[cfg(not(feature = "embeded"))]
    pub fn new(wave_table: OscWaveTable) -> Self {
        let effects = [
            (EffectsModule::Chorus(Chorus::new()), false),
            // (EffectsModule::Reverb(Reverb::new()), false),
        ];
        // let lpf = LowPass::new();
        let mut oscs = array![(Oscillator::new(wave_table), false); N_OSC];
        oscs[0].1 = true;
        let targets = [
            OscTarget::Filter1,
            OscTarget::Filter2,
            OscTarget::Filter1_2,
            OscTarget::Effects,
            OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
        ];

        for i in 0..N_OSC {
            oscs.index_mut(i).0.target = targets[i];
            // info!("{:?}", targets[i]);
            // info!("{:?}", oscs.index_mut(i).0.target);
        }
        let f0 = 440.hz();
        let fs = SAMPLE_RATE.hz();

        // info!("{oscs:?}");
        let coeffs =
            Coefficients::<f32>::from_params(biquad::Type::AllPass, fs, f0, Q_BUTTERWORTH_F32)
                .unwrap();

        let filter = DirectForm1::<f32>::new(coeffs);

        Self {
            oscs,
            envs: array![ADSR::new(); N_ENV],
            lfos: array![LFO::new(); N_LFO],
            filters: [LowPass::new(), LowPass::new()],
            // filters: [BQLowPass::new(), BQLowPass::new()],
            playing: None,
            data_table: DataTable::default(),
            effects,
            level: 1.0,
            level_mod: 0.0,
            all_pass: filter,
        }
    }

    #[cfg(feature = "embeded")]
    pub fn new_2(wave_table: OscWaveTable) -> Self {
        let effects = [
            (EffectsModule::Chorus(Chorus::new()), false),
            // (EffectsModule::Reverb(Reverb::new()), false),
        ];
        // let lpf = LowPass::new();
        let mut oscs = [
            (Oscillator::new(wave_table.clone()), true),
            (Oscillator::new(wave_table.clone()), true),
        ];
        // oscs[0].1 = true;
        oscs[0].0.level = 0.8;
        oscs[1].0.offset = -12;
        oscs[1].0.level = 0.25;
        let targets = [
            // OscTarget::Effects,
            OscTarget::Filter1,
            OscTarget::Filter2,
            OscTarget::Filter1_2,
            OscTarget::Effects,
            OscTarget::DirectOut,
        ];

        for i in 0..N_OSC {
            oscs.index_mut(i).0.target = targets[i];
        }

        let mut lfo = LFO::new();
        lfo.set_frequency(2.0);
        let mut lfo_2 = LFO::new();
        lfo.set_frequency(4.0);

        let f0 = 440.hz();
        let fs = SAMPLE_RATE.hz();

        // info!("{oscs:?}");
        let coeffs =
            Coefficients::<f32>::from_params(biquad::Type::AllPass, fs, f0, Q_BUTTERWORTH_F32)
                .unwrap();

        let filter = DirectForm1::<f32>::new(coeffs);

        Self {
            oscs,
            envs: [
                ADSR::new(),
                ADSR::new(),
                // ADSR::new(),
                // ADSR::new(),
                // ADSR::new(),
            ],
            lfos: [lfo, lfo_2],
            filters: [LowPass::new(), LowPass::new()],
            // filters: [BQLowPass::new(), BQLowPass::new()],
            playing: None,
            data_table: DataTable::default(),
            effects,
            level: 1.0,
            level_mod: 0.0,
            all_pass: filter,
        }
    }

    pub fn press(&mut self, midi_note: u8, velocity: u8) {
        // info!("velocity => {velocity}");
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.press(midi_note)
            }
        });
        self.envs.iter_mut().for_each(|env| env.press());
        self.filters.iter_mut().for_each(|filter| {
            if filter.key_track {
                filter.set_note(midi_to_freq(midi_note as i16));
            }
        });
        self.lfos.iter_mut().for_each(|lfo| lfo.press());
        self.playing = Some(midi_note);
        self.data_table.velocity = Some(velocity);
        self.data_table.note = Some(midi_note);
    }

    pub fn release(&mut self) {
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.release()
            }
        });
        self.lfos.iter_mut().for_each(|lfo| lfo.release());
        self.envs.iter_mut().for_each(|env| env.release());
        // self.filters.iter_mut().for_each(|filter| {
        //     if filter.key_track {
        //         filter.set_note(midi_to_freq(midi_note));
        //     }
        // });
        self.lfos.iter_mut().for_each(|lfo| lfo.release());
        self.playing = None;
    }

    /// resets the mod matrix along with the effects, lfos, oscilators, etc
    pub fn reset(&mut self) {
        // self.lfos.iter_mut().for_each(|lfo| lfo.index);
        self.oscs.iter_mut().for_each(|(osc, on)| {
            if !*on {
                osc.reset()
            }
        });
        self.lfos.iter_mut().for_each(|lfo| lfo.reset());
        self.envs.iter_mut().for_each(|env| env.reset());
        self.filters.iter_mut().for_each(|lp| lp.reset());
        // TODO: figure out how to reset a modulation of mod amount

        // reset self
        self.level_mod = 0.0;
    }

    /// send data from data_table where ever it needs to go, based on the mod_natrix
    pub fn route_mod_matrix(&mut self, mod_matrix: &ModMatrix) {
        // info!("{:?}", self.data_table.velocity);

        // for mod_entry in mod_matrix {
        mod_matrix.iter().for_each(|mod_entry| {
            if let Some(entry) = mod_entry {
                // get mod amount
                let mut amt = self.data_table.get_entry(&entry.src) * entry.amt;

                if entry.bipolar {
                    amt -= entry.amt / 2.0;
                }

                // info!("src {:?}, amt {}, dest {:?}", entry.src, amt, entry.dest);

                match entry.dest {
                    ModMatrixDest::ModMatrixEntryModAmt(mod_amt_amt) => {
                        todo!("figure out a way to modulate mod amounts");
                    }
                    ModMatrixDest::Osc { osc, param } => {
                        let (osc, on) = &mut self.oscs[osc];

                        if *on {
                            osc.modulate(param, amt);
                        }
                    }
                    ModMatrixDest::Env { env, param } => self.envs[env].modulate(param, amt),
                    ModMatrixDest::Lfo { lfo, param } => self.lfos[lfo].modulate(param, amt),
                    ModMatrixDest::LowPass { low_pass, param } => {
                        let i = match low_pass {
                            crate::common::LowPass::LP1 => 0,
                            crate::common::LowPass::LP2 => 1,
                        };

                        self.filters[i].modulate(param, amt)
                    }
                    // ModMatrixDest::LowPass { low_pass, param } => match low_pass {
                    //     LP::LP1 => self.filters[0].modulate(param, amt),
                    //     LP::LP2 => self.filters[1].modulate(param, amt),
                    // },
                    ModMatrixDest::SynthVolume => self.modulate_level(amt),
                };
            }
        });
        // for (osc, on) in self.oscs.iter_mut() {
        //     if on {
        //         osc.mod
        //     }
        // }
    }

    fn modulate_level(&mut self, amt: f32) {
        self.level_mod = amt;
    }

    pub fn get_sample(&mut self, mod_matrix: &ModMatrix) -> f32 {
        if self.playing.is_none() && !self.envs[0].pressed() && self.data_table.env[0] <= 0.0 {
            return 0.0;
        }

        self.route_mod_matrix(mod_matrix);

        // calculate envs
        for (i, env) in self.envs.iter_mut().enumerate() {
            let sample = env.get_samnple();
            self.data_table.env[i] = sample;
            break;
        }

        // calculate lfos
        for (i, lfo) in self.lfos.iter_mut().enumerate() {
            let sample = lfo.get_sample();
            self.data_table.lfos[i] = sample;
        }

        if !self.envs[0].pressed() && self.data_table.env[0] <= 0.0 {
            self.playing = None;
            self.reset();
            return 0.0;
        }

        let mut output = 0.0;

        let mut osc_sample = 0.0;

        for (osc, on) in self.oscs.iter_mut() {
            // output += osc.get_sample();
            // output += self.filters[0].get_sample(osc.get_sample());
            // break;
            if on.to_owned() {
                // continue;

                let sample = osc.get_sample();
                // continue;

                // self.data_table.osc[i] = sample;

                // osc_sample += // self.filters[0].get_sample(sample);
                osc_sample += match osc.target {
                    OscTarget::Filter1 => self.filters[0].get_sample(sample),
                    OscTarget::Filter2 => self.filters[1].get_sample(sample),
                    OscTarget::Filter1_2 => {
                        self.filters[0].get_sample(sample) + self.filters[1].get_sample(sample)
                    }
                    OscTarget::Effects => sample,
                    OscTarget::DirectOut => {
                        // _ => {
                        output += sample;

                        continue;
                    }
                };
            }
        }

        let mut effects_sample = osc_sample;

        for (effect, on) in self.effects.iter_mut() {
            if on.to_owned() {
                effect.take_input(effects_sample);

                effects_sample += effect.get_sample();
            }
        }

        output += effects_sample;

        // return Some(output);

        // an allpass filter.
        let sample = output * self.data_table.env[0];
        // warn!("{}", get_u32_sample(Some(sample)));

        self.all_pass.run(sample)

        // sample
    }
}
