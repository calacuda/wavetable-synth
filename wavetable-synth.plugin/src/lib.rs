use biquad::*;
use nih_plug::prelude::*;
use std::sync::{Arc, RwLock};
use wavetable_synth::{
    common::ModMatrixDest,
    config::{POLYPHONY, SAMPLE_RATE},
    synth_engines::synth::{build_sine_table, osc::N_OVERTONES},
    voice::Voice,
    ModMatrix,
};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct WtSynth {
    params: Arc<WtSynthParams>,
    /// describes what modulates what.
    pub mod_matrix: ModMatrix,
    /// used for routung cc messages
    pub midi_table: [Option<ModMatrixDest>; 256],
    /// the sound producers
    pub voices: std::sync::Arc<[std::sync::RwLock<Voice>]>,
    /// all pass filter to avoid clipping
    allpass: biquad::DirectForm1<f32>,
}

#[derive(Params)]
struct WtSynthParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
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

        Self {
            params: Arc::new(WtSynthParams::default()),
            mod_matrix: [None; 256],
            midi_table: [None; 256],
            voices,
            allpass,
        }
    }
}

impl Default for WtSynthParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-1.0),
                    max: util::db_to_gain(1.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-1.0, 1.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(2.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(0))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

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
