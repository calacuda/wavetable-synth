#![feature(let_chains)]
use anyhow::Result;
use common::ModMatrixDest;
use common::ModMatrixItem;
use config::POLYPHONY;
use effects::EffectsModule;
use enum_dispatch::enum_dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use fxhash::FxHashMap;
use fxhash::FxHashSet;
use log::*;
use midi_control::MidiMessage;
use midir::MidiInput;
use midir::{Ignore, PortInfoError};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use voice::Voice;

pub type HashMap<Key, Val> = FxHashMap<Key, Val>;
pub type HashSet<T> = FxHashSet<T>;
pub type ModMatrix = [Option<ModMatrixItem>; 256];
pub type WaveTable = Arc<[f32]>;

pub mod common;
pub mod config;
pub mod effects;
pub mod lfo;
pub mod synth_engines;
pub mod voice;

pub trait MidiControlled {
    fn midi_input(&mut self, message: &MidiMessage);
}

#[enum_dispatch(EffectsModule, SynthModule)]
pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

pub trait ModulationDest {
    type ModTarget;

    fn modulate(&mut self, what: Self::ModTarget, by: f32);
    /// clears any aplied modulation.
    fn reset(&mut self);
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct App {
    /// join handle for the run_midi thread
    _handle: JoinHandle<()>,
    /// used to coordinate exits from run_midi function
    exit: Arc<AtomicBool>,
    /// describes what modulates what.
    mod_matrix: ModMatrix,
    /// used for routung cc messages
    midi_table: [Option<ModMatrixDest>; 256],
    /// the sound producers
    voices: [Voice; POLYPHONY],
}

#[allow(unused_variables)]
impl MidiControlled for App {
    fn midi_input(&mut self, message: &MidiMessage) {
        // TODO: if note, add midi note to the data table
        // TODO: if cc, route based on learned midi table
    }
}

impl SampleGen for App {
    fn get_sample(&mut self) -> f32 {
        let sample: f32 = self
            .voices
            .iter_mut()
            .filter_map(|voice| voice.get_sample(&self.mod_matrix))
            .sum();

        // TODO: add an AllPass filter

        (sample * 0.9).tanh()
    }
}

pub fn midi_to_freq(midi_note: i16) -> f32 {
    let exp = (f32::from(midi_note) + 36.376_316) / 12.0;

    2.0_f32.powf(exp)
}

pub fn calculate_modulation(base: f32, amt: f32) -> f32 {
    base + base * amt
}

pub fn run_midi(synth: Arc<Mutex<App>>, exit: Arc<AtomicBool>) -> Result<()> {
    let mut registered_ports = HashMap::default();

    while !exit.load(Ordering::Relaxed) {
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
                        let message = MidiMessage::from(message);

                        // do midi stuff
                        synth.lock().unwrap().midi_input(&message);
                    },
                    (),
                ),
            );
        }
    }

    Ok(())
}

pub fn logger_init() -> Result<()> {
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
