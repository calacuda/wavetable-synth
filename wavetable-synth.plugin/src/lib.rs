use biquad::*;
use nih_plug::{log::info, prelude::*};
use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};
use wavetable_synth::{
    common::ModMatrixDest,
    config::{N_ENV, N_LFO, N_OSC, POLYPHONY, SAMPLE_RATE},
    synth_engines::{
        synth::{
            build_sine_table,
            osc::{OscTarget, N_OVERTONES},
        },
        synth_common::env::{ATTACK, DECAY, RELEASE},
    },
    voice::Voice,
    ModMatrix,
};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct WtSynth {
    params: Arc<WtSynthParams>,
    memo_params: Arc<WtSynthParams>,
    /// describes what modulates what.
    pub mod_matrix: ModMatrix,
    /// used for routung cc messages
    pub midi_table: [Option<ModMatrixDest>; 256],
    /// the sound producers
    pub voices: std::sync::Arc<[std::sync::RwLock<Voice>]>,
    /// all pass filter to avoid clipping
    allpass: biquad::DirectForm1<f32>,
}

#[derive(Params, Debug)]
struct OscParams {
    // Osc stuff
    #[id = "Osc Enabled"]
    pub osc_enable: BoolParam,
    #[id = "Osc Level"]
    pub osc_level: FloatParam,
    #[id = "Osc Detune"]
    pub osc_detune: FloatParam,
    #[id = "Osc Note Offset"]
    pub osc_offset: IntParam,
    #[id = "Osc Target"]
    pub osc_target: EnumParam<OscTarget>,
}

impl OscParams {
    pub fn new(i: usize, target: OscTarget) -> Self {
        Self {
            osc_enable: BoolParam::new(format!("Osc {i} Enabled"), i == 1),
            osc_level: FloatParam::new(
                format!("Osc {i} Level"),
                1.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.0,
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(0.0, 1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(1.0)),
            osc_detune: FloatParam::new(
                format!("Osc {i} Detune"),
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            osc_offset: IntParam::new(
                format!("Osc {i} Note Offset"),
                0,
                IntRange::Linear { min: -96, max: 96 },
            ),
            osc_target: EnumParam::new(format!("Osc {i} Target"), target),
        }
    }
}

#[derive(Params)]
struct EnvParams {
    // Env stuff
    #[id = "Attack"]
    pub attack: FloatParam,
    #[id = "Decay"]
    pub decay: FloatParam,
    #[id = "Sustain"]
    pub sustain: FloatParam,
    #[id = "Release"]
    pub release: FloatParam,
}

impl EnvParams {
    fn new(i: usize) -> Self {
        Self {
            attack: FloatParam::new(
                format!("Attack {i}"),
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            decay: FloatParam::new(
                format!("Decay {i}"),
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            sustain: FloatParam::new(
                format!("Sustain {i}"),
                0.5,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.0,
                    factor: FloatRange::gain_skew_factor(0.0, 1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(1.0)),
            release: FloatParam::new(
                format!("Release {i}"),
                0.02,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

#[derive(Params, Debug)]
struct FilterParams {
    // filter stuff
    // #[id = "Filter Enabled"]
    // pub enabled: BoolParam,
    #[id = "Key Track Enabled"]
    pub key_track: BoolParam,
    #[id = "Cutoff"]
    pub cutoff: FloatParam,
    #[id = "Resonance"]
    pub resonance: FloatParam,
    #[id = "Dry Mix"]
    pub mix: FloatParam,
}

impl FilterParams {
    fn new(i: usize) -> Self {
        Self {
            // enabled: BoolParam::new(format!("Filter {i} Enabled"), true),
            key_track: BoolParam::new(format!("Filter {i} Key Tracking"), true),
            cutoff: FloatParam::new(
                format!("Filter {i} Cutoff"),
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            resonance: FloatParam::new(
                format!("Filter {i} Resonance"),
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            mix: FloatParam::new(
                format!("Filter {i} Dry Mix"),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

#[derive(Params)]
struct LfoParams {
    #[id = "Speed"]
    pub freq: FloatParam,
    // TODO: Add LFO WaveTable here
}

impl LfoParams {
    fn new(i: usize) -> Self {
        Self {
            freq: FloatParam::new(
                format!("LFO {i} frequency"),
                2.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 20.0,
                },
            ),
        }
    }
}

#[derive(Params)]
struct WtSynthParams {
    /// parameters for eatch Oscilator
    #[nested(array, group = "OSC")]
    /// parameters for Envelope Generators
    pub osc: Vec<OscParams>,
    #[nested(array, group = "ENV")]
    pub env: Vec<EnvParams>,
    /// parameters for Filter 1 and 2
    #[nested(array, group = "Filter")]
    pub filter: [FilterParams; 2],
    // params for lfos
    #[nested(array, group = "LFO")]
    pub lfo: Vec<LfoParams>,
}

impl Default for WtSynthParams {
    fn default() -> Self {
        let osc = [
            OscTarget::Filter1,
            OscTarget::Filter2,
            OscTarget::Filter1_2,
            OscTarget::Effects,
            OscTarget::DirectOut,
        ][0..N_OSC]
            .iter()
            .enumerate()
            .map(|(i, target)| OscParams::new(i + 1, *target))
            .collect();
        let env = (0..N_ENV).map(|i| EnvParams::new(i + 1)).collect();
        let filter = [FilterParams::new(1), FilterParams::new(2)];
        let lfo = (0..N_LFO).map(|i| LfoParams::new(i + 1)).collect();

        Self {
            osc,
            env,
            filter,
            lfo,
        }
    }
}

impl Default for WtSynth {
    fn default() -> Self {
        let mut overtones = [1.0; N_OVERTONES];

        (1..N_OVERTONES).for_each(|i| overtones[i] = (i + 1) as f64);

        let wave_table = build_sine_table(&overtones);

        let voices: std::sync::Arc<[std::sync::RwLock<Voice>]> = (0..POLYPHONY)
            .map(|_| RwLock::new(Voice::new(wave_table.clone())))
            .collect();

        // Cutoff and sampling frequencies
        let f0 = ((20_000 + 20) / 2).hz();
        let fs = SAMPLE_RATE.hz();
        let coeffs =
            Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
        let allpass = DirectForm1::<f32>::new(coeffs);

        // voices[0].write().unwrap().press(48, 100);
        let params = || WtSynthParams::default();

        Self {
            params: Arc::new(params()),
            memo_params: Arc::new(params()),
            mod_matrix: [None; 256],
            midi_table: [None; 256],
            voices,
            allpass,
        }
    }
}

impl Plugin for WtSynth {
    const NAME: &'static str = "Wt Synth";
    const VENDOR: &'static str = "Calacuda";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    // const EMAIL: &'static str = "your@email.com";
    const EMAIL: &'static str = "pls-dont-email-me";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(0),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // fn initialize(
    //     &mut self,
    //     _audio_io_layout: &AudioIOLayout,
    //     _buffer_config: &BufferConfig,
    //     _context: &mut impl InitContext<Self>,
    // ) -> bool {
    //     // Resize buffers and perform other potentially expensive initialization operations here.
    //     // The `reset()` function is always called right after this function. You can remove this
    //     // function if you do not need it.
    //     true
    // }
    //
    // fn reset(&mut self) {
    //     // Reset buffers and envelopes here. This can be called from the audio thread and may not
    //     // allocate. You can remove this function if you do not need it.
    // }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing: _,
                    voice_id: _,
                    channel: _,
                    note,
                    velocity,
                } => {
                    for voice in self.voices.iter() {
                        if let Ok(mut voice) = voice.write() {
                            if voice.playing.is_none() {
                                voice.press(note, (velocity * 127.) as u8);
                                break;
                            }
                        }
                    }
                }
                NoteEvent::NoteOff {
                    timing: _,
                    voice_id: _,
                    channel: _,
                    note,
                    velocity: _,
                } => {
                    for voice in self.voices.iter() {
                        if let Ok(mut voice) = voice.write() {
                            if voice.playing.is_some_and(|n| n == note) {
                                voice.release();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // set voice parameters
        self.set_voice_params();

        // reset memo_params
        self.memo_params = self.params.clone();

        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            // let gain = self.params.gain.smoothed.next();
            let value: f32 = self
                .voices
                .iter()
                .map(|voice| voice.write().unwrap().get_sample(&self.mod_matrix))
                .sum();

            // AllPass filter
            let value = self.allpass.run(value * 0.75) * 0.5;

            for sample in channel_samples {
                *sample = value;
            }
        }

        ProcessStatus::Normal
    }
}

impl WtSynth {
    fn set_voice_params(&mut self) {
        // Oscilator
        self.params
            .osc
            .iter()
            // .zip(self.memo_params.osc.iter())
            .enumerate()
            .for_each(|(i, osc_params)| {
                // oscilator enabled
                {
                    let param = osc_params.osc_enable.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.oscs[i].1 {
                                voice.oscs[i].1 = param;
                            }
                        }
                    })
                }

                // oscilator level
                {
                    let param = osc_params.osc_level.smoothed.next();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.oscs[i].0.level {
                                voice.oscs[i].0.level = param;
                            }
                        }
                    })
                }

                // oscilator detune
                {
                    let param = osc_params.osc_detune.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.oscs[i].0.detune {
                                voice.oscs[i].0.detune = param;
                            }
                        }
                    })
                }

                // oscilator offset
                {
                    let param = osc_params.osc_offset.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.oscs[i].0.offset as i32 {
                                voice.oscs[i].0.offset = param as i16;
                            }
                        }
                    })
                }

                // oscilator target
                {
                    let param = osc_params.osc_target.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.oscs[i].0.target {
                                voice.oscs[i].0.target = param;
                            }
                        }
                    })
                }
            });

        // Envelope filter
        self.params
            .env
            .iter()
            .zip(self.memo_params.env.iter())
            .enumerate()
            .for_each(|(i, (env_params, memo_params))| {
                // Envelope Attack
                {
                    let param = env_params.attack.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.envs[i].base_params[ATTACK] {
                                // info!("set attack to {}", param);
                                voice.envs[i].set_atk(param);
                            }
                        }
                    })
                }

                // Envelope Decay
                {
                    let param = env_params.decay.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.envs[i].base_params[DECAY] {
                                // info!("set decay to {}", param);
                                voice.envs[i].set_decay(param);
                            }
                        }
                    })
                }

                // Envelope Sustain
                {
                    let param = env_params.sustain.smoothed.next();

                    if param != memo_params.sustain.smoothed.next() {
                        self.voices.iter().for_each(|voice| {
                            if let Ok(mut voice) = voice.write() {
                                // info!("setting sustain to: {param}");
                                voice.envs[i].set_sus(param);
                            }
                        })
                    }
                }

                // Envelope release
                {
                    let param = env_params.release.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.envs[i].base_params[RELEASE] {
                                // info!("set release to {}", param);
                                voice.envs[i].set_release(param);
                            }
                        }
                    })
                }
            });

        // Filters
        self.params
            .filter
            .iter()
            .enumerate()
            .for_each(|(i, filter_params)| {
                // key tracking
                {
                    let param = filter_params.key_track.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.filters[i].key_track {
                                voice.filters[i].key_track = param;
                            }
                        }
                    })
                }

                // filter cutoff
                {
                    let param = filter_params.cutoff.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.filters[i].cutoff {
                                voice.filters[i].cutoff = param;
                            }
                        }
                    })
                }

                // filter resonance
                {
                    let param = filter_params.resonance.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.filters[i].resonance {
                                voice.filters[i].resonance = param;
                            }
                        }
                    })
                }

                // filter dry mix
                {
                    let param = filter_params.mix.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.filters[i].mix {
                                voice.filters[i].mix = param;
                            }
                        }
                    })
                }
            });

        self.params
            .lfo
            .iter()
            .enumerate()
            .for_each(|(i, lfo_params)| {
                // frequency
                {
                    let param = lfo_params.freq.value();

                    self.voices.iter().for_each(|voice| {
                        if let Ok(mut voice) = voice.write() {
                            if param != voice.lfos[i].freq {
                                voice.lfos[i].set_frequency(param);
                            }
                        }
                    })
                }
            });
    }
}

impl ClapPlugin for WtSynth {
    const CLAP_ID: &'static str = "online.eoghan-west.wt-synth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("a wavetable synth");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Synthesizer, ClapFeature::Mono];
}

impl Vst3Plugin for WtSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"WtSynthPlugin\0\0\0";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Synth, Vst3SubCategory::Mono];
}

nih_export_clap!(WtSynth);
nih_export_vst3!(WtSynth);
