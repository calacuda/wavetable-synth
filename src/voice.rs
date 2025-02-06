use crate::{
    calculate_modulation,
    common::{DataTable, LowPass as LP, ModMatrixDest},
    config::{N_ENV, N_LFO, N_OSC},
    effects::{chorus::Chorus, reverb::Reverb, Effect, EffectsModule},
    lfo::LFO,
    midi_to_freq,
    synth_engines::{
        synth::osc::{OscTarget, Oscillator},
        synth_common::{env::ADSR, moog_filter::LowPass},
    },
    ModMatrix, ModulationDest, SampleGen, WaveTable,
};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Voice {
    /// Oscilators holds the osc and if its playing
    oscs: [(Oscillator, bool); N_OSC],
    /// env filters
    envs: [ADSR; N_ENV],
    /// LFOs
    lfos: [LFO; N_LFO],
    /// filters
    filters: [LowPass; 2],
    /// what notes this voice is playing
    playing: Option<u8>,
    /// effects, holds the effect and if its one or not
    effects: [(EffectsModule, bool); 2],
    /// holds the out put of the different modules and also other needed data (velocity, and note).
    data_table: DataTable,
    level: f32,
    level_mod: f32,
}

impl Voice {
    pub fn new(wave_table: WaveTable) -> Self {
        let effects = [
            (EffectsModule::Chorus(Chorus::new()), false),
            (EffectsModule::Reverb(Reverb::new()), false),
        ];

        Self {
            oscs: [
                (Oscillator::new(wave_table.deref().into()), true),
                (Oscillator::new(wave_table.deref().into()), false),
                (Oscillator::new(wave_table.deref().into()), false),
            ],
            envs: [
                ADSR::new(),
                ADSR::new(),
                ADSR::new(),
                ADSR::new(),
                ADSR::new(),
            ],
            lfos: [LFO::new(), LFO::new(), LFO::new(), LFO::new()],
            filters: [LowPass::new(), LowPass::new()],
            playing: None,
            data_table: DataTable::default(),
            effects,
            level: 1.0,
            level_mod: 0.0,
        }
    }

    pub fn press(&mut self, midi_note: u8) {
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
    }

    pub fn release(&mut self) {
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.release()
            }
        });
        self.envs.iter_mut().for_each(|env| env.release());
        self.lfos.iter_mut().for_each(|lfo| lfo.release());
        // self.filters.iter_mut().for_each(|filter| {
        //     if filter.key_track {
        //         filter.set_note(midi_to_freq(midi_note));
        //     }
        // });
        // self.lfos.iter_mut().for_each(|lfo| lfo.start());
    }

    /// ressets the mod matrix along with the effects, lfos, oscilators, etc
    pub fn reset(&mut self) {
        // self.lfos.iter_mut().for_each(|lfo| lfo.index);
        self.oscs.iter_mut().for_each(|(osc, on)| {
            if *on {
                osc.reset()
            }
        });
        self.lfos.iter_mut().for_each(|lfo| lfo.reset());
        self.envs.iter_mut().for_each(|env| env.reset());
        self.filters.iter_mut().for_each(|lp| lp.reset());
        self.level_mod = 0.0;
        // TODO: figure out how to reset a modulation of mod amount
    }

    /// send data from data_table where ever it needs to go, based on the mod_natrix
    pub fn route_mod_matrix(&mut self, mod_matrix: &ModMatrix) {
        for mod_entry in mod_matrix {
            if let Some(entry) = mod_entry {
                // get mod amount
                let mut amt = self.data_table.get_entry(&entry.src) * entry.amt;

                if entry.bipolar {
                    amt -= entry.amt / 2.0;
                }

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
                    ModMatrixDest::LowPass { low_pass, param } => match low_pass {
                        LP::LP1 => self.filters[0].modulate(param, amt),
                        LP::LP2 => self.filters[1].modulate(param, amt),
                    },
                    ModMatrixDest::SynthVolume => self.modulate_level(amt),
                };
            }
        }
        // for (osc, on) in self.oscs.iter_mut() {
        //     if on {
        //         osc.mod
        //     }
        // }
    }

    fn modulate_level(&mut self, amt: f32) {
        self.level_mod = amt;
    }

    pub fn get_sample(&mut self, mod_matrix: &ModMatrix) -> Option<f32> {
        if self.playing.is_none() {
            return None;
        }

        self.route_mod_matrix(mod_matrix);

        // calculate envs
        for (i, env) in self.envs.iter_mut().enumerate() {
            let sample = env.get_samnple();
            self.data_table.env[i] += sample;
        }

        // calculate lfos
        for (i, lfo) in self.lfos.iter_mut().enumerate() {
            let sample = lfo.get_sample();
            self.data_table.lfos[i] += sample;
        }

        if !self.envs[0].pressed() && self.data_table.env[0] <= 0.0 {
            self.playing = None;
            self.reset();
            return None;
        }

        let mut output = 0.0;

        let mut osc_sample = 0.0;

        for (i, (osc, on)) in self.oscs.iter_mut().enumerate() {
            if !*on {
                continue;
            }

            let sample = osc.get_sample();

            self.data_table.osc[i] = sample;

            osc_sample += match osc.target {
                OscTarget::Filter1 => self.filters[0].get_sample(sample),
                OscTarget::Filter2 => self.filters[1].get_sample(sample),
                OscTarget::Filter1_2 => {
                    self.filters[0].get_sample(sample) + self.filters[1].get_sample(sample)
                }
                OscTarget::Effects => sample,
                OscTarget::DirectOut => {
                    output += sample;
                    // if let Some(s) = output.as_mut() {
                    //     *s += sample
                    // } else {
                    //     output = Some(sample);
                    // }

                    continue;
                }
            }
        }

        let mut effects_sample = None;

        for (effect, on) in self.effects.iter_mut() {
            // if let Some((effect, on)) = effect {
            if *on {
                let s = if let Some(sample) = effects_sample {
                    sample
                } else {
                    osc_sample
                };

                effect.take_input(s);

                effects_sample = Some(effect.get_sample());
            }
        }

        if let Some(sample) = effects_sample {
            output += sample;
        }

        // TODO: add an allpass filter.
        Some(output * self.data_table.env[0] * calculate_modulation(self.level, self.level_mod))
    }
}
