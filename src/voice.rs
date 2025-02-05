use crate::{
    common::DataTable,
    effects::{chorus::Chorus, reverb::Reverb, Effect, EffectsModule},
    lfo::LFO,
    midi_to_freq,
    synth_engines::{
        synth::osc::{OscTarget, Oscillator},
        synth_common::{env::ADSR, moog_filter::LowPass, WaveTable},
    },
    ModMatrix, SampleGen,
};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Voice {
    /// Oscilators
    oscs: [(Oscillator, bool); 3],
    /// env filters
    envs: [ADSR; 5],
    /// LFOs
    lfos: [LFO; 4],
    /// filters
    filters: [LowPass; 2],
    /// what notes this voice is playing
    playing: Option<u8>,
    /// effects, holds the effect and if its one or not
    effects: [(EffectsModule, bool); 2],
    /// holds the out put of the different modules and also other needed data (velocity, and note).
    data_table: DataTable,
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
        // self.lfos.iter_mut().for_each(|lfo| lfo.start());
        self.playing = Some(midi_note);
    }

    pub fn release(&mut self) {
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.release()
            }
        });
        self.envs.iter_mut().for_each(|env| env.release());
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
    }

    /// send data from data_table where ever it needs to go, based on the mod_natrix
    pub fn route_mod_matrix(&mut self, mod_matrix: &ModMatrix) {}

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
        Some(output * self.data_table.env[0])
    }
}
