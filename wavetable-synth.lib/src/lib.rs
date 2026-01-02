#![cfg_attr(not(feature = "std"), no_std)]
// #![feature(stmt_expr_attributes)]
#[cfg(feature = "embeded")]
extern crate alloc;
#[cfg(feature = "embeded")]
use heapless::Vec;
// use alloc::rc::Rc;
use anyhow::Result;
use common::ModMatrixDest;
use common::ModMatrixItem;
// use config::LFO_WAVE_TABLE_SIZE;
// use config::OSC_WAVE_TABLE_SIZE;
use config::POLYPHONY;
use effects::EffectsModule;
use enum_dispatch::enum_dispatch;
use log::*;
// #[cfg(feature = "desktop")]
// use std::rc::Rc;
use synth_engines::synth::build_sine_table;
use synth_engines::synth::osc::N_OVERTONES;
use voice::Voice;

#[cfg(feature = "desktop")]
pub type HashMap<Key, Val> = fxhash::FxHashMap<Key, Val>;
// pub type HashSet<T> = FxHashSet<T>;
#[cfg(feature = "desktop")]
pub const MOD_MATRIX_SIZE: usize = 256;
#[cfg(feature = "desktop")]
pub type ModMatrix = [Option<ModMatrixItem>; MOD_MATRIX_SIZE];
#[cfg(feature = "embeded")]
pub const MOD_MATRIX_SIZE: usize = 8;
#[cfg(feature = "embeded")]
pub type ModMatrix = Vec<Option<ModMatrixItem>, MOD_MATRIX_SIZE>;
// pub type WaveTable = Arc<[f32]>;
// pub type OscWaveTable = [f32; OSC_WAVE_TABLE_SIZE];
// pub type OscWaveTable = Arc<[f32]>;
// pub type OscWaveTable = Rc<[f32]>;
// pub type OscWaveTable = Vec<f32>;
// pub type LfoWaveTable = [f32; LFO_WAVE_TABLE_SIZE];
// pub type LfoWaveTable = Rc<[f32]>;
// pub type LfoWaveTable = Vec<f32>;

#[cfg(feature = "embeded")]
pub type OscWaveTable = Vec<f32, { config::OSC_WAVE_TABLE_SIZE }>;

#[cfg(feature = "embeded")]
pub type LfoWaveTable = Vec<f32, { config::LFO_WAVE_TABLE_SIZE }>;

#[cfg(feature = "desktop")]
pub type OscWaveTable = Vec<f32>;

#[cfg(feature = "desktop")]
pub type LfoWaveTable = Vec<f32>;

pub mod common;
pub mod config;
pub mod effects;
pub mod lfo;
pub mod synth_engines;
pub mod voice;

#[cfg(feature = "desktop")]
pub trait MidiControlled {
    fn midi_input(&mut self, message: &midi_control::MidiMessage);
}

#[enum_dispatch(EffectsModule)]
pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

pub trait ModulationDest {
    type ModTarget;

    fn modulate(&mut self, what: Self::ModTarget, by: f32);
    /// clears any applied modulation.
    fn reset(&mut self);
}

#[cfg(feature = "desktop")]
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct App {
    /// used to coordinate exits from run_midi function
    exit: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// describes what modulates what.
    pub mod_matrix: ModMatrix,
    /// used for routung cc messages
    pub midi_table: [Option<ModMatrixDest>; 256],
    /// the sound producers
    pub voices: std::sync::Arc<[std::sync::RwLock<Voice>]>,
    /// all pass filter to avoid clipping
    allpass: biquad::DirectForm1<f32>,
}

#[cfg(feature = "desktop")]
impl Default for App {
    fn default() -> Self {
        use crate::config::SAMPLE_RATE;
        use biquad::*;
        use std::sync::{Arc, RwLock};

        let mut overtones = [1.0; N_OVERTONES];

        (1..N_OVERTONES).for_each(|i| overtones[i] = (i + 1) as f64);

        let wave_table = build_sine_table(&overtones);

        let voices = (0..POLYPHONY)
            .map(|_| RwLock::new(Voice::new(wave_table.clone())))
            .collect();

        // Cutoff and sampling frequencies
        let f0 = ((20_000 + 20) / 2).hz();
        let fs = SAMPLE_RATE.hz();
        let coeffs =
            Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
        let allpass = DirectForm1::<f32>::new(coeffs);

        Self {
            exit: Arc::new(false.into()),
            mod_matrix: [None; 256],
            midi_table: [None; 256],
            voices,
            allpass,
        }
    }
}

#[cfg(feature = "desktop")]
#[allow(unused_variables)]
impl MidiControlled for App {
    fn midi_input(&mut self, message: &midi_control::MidiMessage) {
        use midi_control::{KeyEvent, MidiMessage};

        // TODO: if note, add midi note to the data table
        // TODO: if cc, route based on learned midi table
        match *message {
            MidiMessage::NoteOn(_channel, KeyEvent { key, value }) => {
                for voice in self.voices.iter() {
                    if let Ok(mut voice) = voice.write() {
                        if voice.playing.is_none() {
                            voice.press(key, value);
                            break;
                        }
                    }
                }
            }
            MidiMessage::NoteOff(_channel, KeyEvent { key, value }) => {
                for voice in self.voices.iter() {
                    if let Ok(mut voice) = voice.write() {
                        if voice.playing.is_some_and(|note| note == key) {
                            voice.release();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(feature = "desktop")]
impl SampleGen for App {
    fn get_sample(&mut self) -> f32 {
        use biquad::Biquad;

        let sample: f32 = self
            .voices
            .iter()
            .map(|voice| voice.write().unwrap().get_sample(&self.mod_matrix))
            .sum();

        // AllPass filter

        let sample = self.allpass.run(sample * 0.75) * 0.5;
        // let sample = sample.tanh();

        sample
    }
}

#[cfg(feature = "desktop")]
impl App {
    pub fn play(&mut self, note: midi_control::MidiNote, velocity: u8) {
        for voice in self.voices.iter() {
            if let Ok(mut voice) = voice.write() {
                if voice.playing.is_none() {
                    voice.press(note, velocity);
                    break;
                }
            }
        }
    }

    pub fn stop(&mut self, note: midi_control::MidiNote) {
        for voice in self.voices.iter() {
            if let Ok(mut voice) = voice.write() {
                if voice.playing.is_some_and(|n| n == note) {
                    voice.release();
                }
            }
        }
    }
}

pub fn midi_to_freq(midi_note: i16) -> f32 {
    let exp = (f32::from(midi_note) + 36.376_316) / 12.0;

    pow(2.0, exp)
}

pub fn calculate_modulation(base: f32, amt: f32) -> f32 {
    base + base * amt
}

#[cfg(feature = "desktop")]
pub fn run_midi(
    synth: std::sync::Arc<std::sync::RwLock<App>>, /* , exit: Arc<AtomicBool> */
) -> Result<()> {
    use midir::{Ignore, MidiInput, PortInfoError};

    let mut registered_ports = HashMap::default();
    let exit = {
        let app = synth.read().unwrap();
        app.exit.clone()
    };

    while !exit.load(std::sync::atomic::Ordering::Relaxed) {
        let mut midi_in = MidiInput::new("midir reading input")?;
        midi_in.ignore(Ignore::None);

        // Get an input port (read from console if multiple are available)
        let in_ports = midi_in.ports();
        let port_names: Vec<std::result::Result<String, PortInfoError>> = in_ports
            .iter()
            .map(|port| midi_in.port_name(port))
            .collect();
        registered_ports.retain(|k: &String, _| port_names.contains(&Ok(k.clone())));

        for in_port in in_ports.iter() {
            let Ok(port_name) = midi_in.port_name(in_port) else {
                continue;
            };

            if registered_ports.contains_key(&port_name) {
                continue;
            }

            info!("port {port_name}");
            let mut midi_in = MidiInput::new("midir reading input")?;
            midi_in.ignore(Ignore::None);
            let synth = synth.clone();

            registered_ports.insert(
                port_name,
                midi_in.connect(
                    in_port,
                    "midir-read-input",
                    move |_stamp, message, _| {
                        let message = midi_control::MidiMessage::from(message);

                        // do midi stuff
                        // synth.lock().unwrap().midi_input(&message);
                        if let Ok(mut synth) = synth.write() {
                            synth.midi_input(&message);
                        }
                    },
                    (),
                ),
            );
        }
    }

    Ok(())
}

#[cfg(feature = "desktop")]
pub fn logger_init() -> Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Magenta)
        .error(Color::Red);

    #[cfg(debug_assertions)]
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        // .chain(fern::log_file("wavetable-synth.log")?)
        // .filter(|metadata| metadata..starts_with("wavetable"))
        .chain(std::io::stderr())
        .apply()?;

    #[cfg(not(debug_assertions))]
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        // .chain(fern::log_file("wavetable-synth.log")?)
        // .filter(|metadata| metadata.target().starts_with("wavetable"))
        .chain(std::io::stderr())
        .apply()?;

    info!("logger started");

    Ok(())
}

#[cfg(feature = "std")]
#[inline]
fn pow(base: f32, exp: f32) -> f32 {
    base.powf(exp)
}

#[cfg(feature = "embeded")]
#[inline]
fn pow(base: f32, exp: f32) -> f32 {
    use libm::powf;

    powf(base, exp)
}

#[cfg(feature = "std")]
#[inline]
fn tanh(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;

    let a = x + (0.16489087 * x3) + (0.00985468 * x5);

    a / (1.0 + (a * a)).sqrt()
}

#[cfg(feature = "embeded")]
#[inline]
fn tanh(x: f32) -> f32 {
    use libm::sqrtf;

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;

    let a = x + (0.16489087 * x3) + (0.00985468 * x5);

    a / sqrtf(1.0 + (a * a))
}

#[cfg(feature = "std")]
#[inline]
fn exp(x: f32) -> f32 {
    x.exp()
}

#[cfg(feature = "embeded")]
#[inline]
fn exp(x: f32) -> f32 {
    use libm::expf;

    expf(x)
}

#[cfg(feature = "std")]
#[inline]
fn sin(x: f64) -> f64 {
    x.sin()
}

#[cfg(feature = "embeded")]
#[inline]
fn sin(x: f64) -> f64 {
    use libm::sin;

    sin(x)
}
