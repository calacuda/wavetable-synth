use midi_control::MidiNote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum GuiParam {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum Knob {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

// #[derive(
//     Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash, EnumIter, Serialize, Deserialize,
// )]
// pub enum SynthEngineType {
//     SubSynth,
//     B3Organ,
//     Wurlitzer,
//     // DrumSynth,
//     // SamplerSynth,
//     // WaveTableSynth,
// }
//
// impl Into<usize> for SynthEngineType {
//     fn into(self) -> usize {
//         match self {
//             Self::B3Organ => 0,
//             Self::SubSynth => 1,
//             Self::Wurlitzer => 2,
//         }
//     }
// }
//
// impl Display for SynthEngineType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::SubSynth => write!(f, "Subtract"),
//             Self::B3Organ => write!(f, "Organ"),
//             Self::Wurlitzer => write!(f, "Wurlitzer"),
//             // Self::SamplerSynth => write!(f, "Sampler"),
//             // Self::WaveTableSynth => write!(f, "WaveTbl"),
//         }
//     }
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DataTable {
    // osc_1: f32,
    // osc_2: f32,
    // osc_3: f32,
    pub osc: [f32; 3],
    // env_1: f32,
    // env_2: f32,
    // env_3: f32,
    // env_4: f32,
    pub env: [f32; 5],
    // pub lfo_1: f32,
    // pub lfo_2: f32,
    // pub lfo_3: f32,
    // pub lfo_4: f32,
    pub lfos: [f32; 4],
    pub filter_1: f32,
    pub filter_2: f32,
    pub chorus: f32,
    pub reverb: f32,
    pub direct_out: f32,
    // pub notes: [Option<(MidiNote, u8)>; POLYPHONY],
    pub note: Option<MidiNote>,
    pub velocity: Option<u8>,
    pub pitch_bend: f32,
    pub mod_wheel: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ModMatrixSrc {
    Velocity,
    Env1,
    Env2,
    Env3,
    Env4,
    Lfo1,
    Lfo2,
    Lfo3,
    Lfo4,
    Macro1,
    Macro2,
    Macro3,
    Macro4,
    ModWheel,
    PitchWheel,
}

// TODO: make ModMatrixDest an enum that holds other enums

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ModMatrixDest {
    ModMatrixEntryModAmt(usize),
    Osc1Level,
    Osc2Level,
    Osc3Level,
    Osc1Tune,
    Osc2Tune,
    Osc3Tune,
    Env1Atk,
    Env2Atk,
    Env3Atk,
    Env4Atk,
    Env1Decay,
    Env2Decay,
    Env3Decay,
    Env4Decay,
    Env1Sus,
    Env2Sus,
    Env3Sus,
    Env4Sus,
    Env1Rel,
    Env2Rel,
    Env3Rel,
    Env4Rel,
    Lfo1Speed,
    Lfo2Speed,
    Lfo3Speed,
    Lfo4Speed,
    LowPass1Cutoff,
    LowPass2Cutoff,
    LowPass1Res,
    LowPass2Res,
    LowPass1Mix,
    LowPass2Mix,
    LowPass1Keytrack,
    LowPass2Keytrack,
    LowPass1Drive,
    LowPass2Drive,
    SynthVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModMatrixItem {
    src: ModMatrixSrc,
    dest: ModMatrixDest,
    amt: f32,
    bipolar: bool,
}

// #[derive(Debug)]
// pub struct StepperSynth {
//     // synth: Arc<Mutex<Synth>>,
//     updated: Arc<Mutex<bool>>,
//     // screen: Screen,
//     _handle: JoinHandle<()>,
//     // _midi_thread: JoinHandle<()>,
//     exit: Arc<AtomicBool>,
//     // pub midi_sequencer: Arc<Mutex<SequencerIntake>>,
// }

// impl StepperSynth {
// pub fn new() -> Self {
//     // build synth in arc mutex
//     let app = Arc::new(Mutex::new(App::new()));
//     // let sequencer = Arc::new(Mutex::new(SequencerIntake::new(synth)));
//
//     let updated = Arc::new(Mutex::new(true));
//     let exit = Arc::new(AtomicBool::new(false));
//     // let effect_midi = Arc::new(AtomicBool::new(false));
//
//     let handle = {
//         // let seq = sequencer.clone();
//         // let synth = synth.clone();
//         let app = app.clone();
//         let updated = updated.clone();
//         let exit = exit.clone();
//         // let effect_midi = effect_midi.clone();
//
//         spawn(move || {
//             let params = OutputDeviceParameters {
//                 channels_count: 1,
//                 sample_rate: SAMPLE_RATE as usize,
//                 // channel_sample_count: 2048,
//                 channel_sample_count: 1024,
//             };
//             let device = run_output_device(params, {
//                 // let seq = seq.clone();
//
//                 move |data| {
//                     for samples in data.chunks_mut(params.channels_count) {
//                         let value = app.lock().expect("couldn't lock app").get_sample();
//
//                         for sample in samples {
//                             *sample = value;
//                         }
//                     }
//                 }
//             });
//
//             if let Err(e) = device {
//                 error!("strating audio playback caused error: {e}");
//             }
//
//             // let seq = sequencer.clone();
//
//             if let Err(e) = run_midi(app, updated, exit) {
//                 error!("{e}");
//             }
//         })
//     };
//
//     // let thread = {
//     //     let seq = sequencer.clone();
//     //
//     //     spawn(move || loop {
//     //         while !seq
//     //             .clone()
//     //             .lock()
//     //             .unwrap()
//     //             .state
//     //             .playing
//     //             .load(Ordering::Relaxed)
//     //         {
//     //             sleep(Duration::from_secs_f64(0.001));
//     //         }
//     //
//     //         play_sequence(seq.clone());
//     //     })
//     // };
//
//     if let Err(reason) = logger_init() {
//         eprintln!("failed to initiate logger because {reason}");
//     }
//
//     info!("Synth is ready to make sound");
//
//     Self {
//         // synth,
//         updated,
//         screen: Screen::Synth(SynthEngineType::B3Organ),
//         _handle: handle,
//         // _midi_thread: thread,
//         // midi_sequencer: sequencer,
//         exit,
//         // effect_midi,
//     }
// }

// pub fn get_engine_state(&self) -> SynthEngineState {
//     let mut seq = self.midi_sequencer.lock().unwrap();
//
//     SynthEngineState {
//         engine: seq.synth.engine_type,
//         effect: seq.synth.effect_type,
//         effect_on: seq.synth.effect_power,
//         knob_params: seq.synth.get_engine().get_params(),
//         gui_params: seq.synth.get_engine().get_gui_params(),
//     }
// }

// pub fn exit(&mut self) {
//     warn!("GoodBye");
//     self.exit.store(true, Ordering::Relaxed);
// }
//
// pub fn updated(&self) -> bool {
//     *self.updated.lock().unwrap()
// }
//
// pub fn toggle_effect_power(&mut self) {
//     {
//         let mut seq = self.midi_sequencer.lock().unwrap();
//         seq.synth.effect_power = !seq.synth.effect_power;
//     }
//     self.set_updated();
// }
//
// fn set_updated(&mut self) {
//     (*self.updated.lock().unwrap()) = true;
// }
//
// pub fn set_screen(&mut self, screen: Screen) {
//     self.screen = screen;
//     // info!("screen set");
//
//     match screen {
//         Screen::Effect(effect) => {
//             self.set_effect(effect);
//             // self.effect_midi.store(true, Ordering::Relaxed)
//             self.midi_sequencer.lock().unwrap().synth.target_effects = true;
//         }
//         Screen::Synth(engine) => {
//             self.set_engine(engine);
//             // self.effect_midi.store(false, Ordering::Relaxed)
//             self.midi_sequencer.lock().unwrap().synth.target_effects = false;
//         }
//         Screen::Stepper(seq) => {
//             self.midi_sequencer.lock().unwrap().set_rec_head_seq(seq);
//         } // Screen::Sequencer() => {}
//     }
//
//     // info!("screen engine/effect set");
//
//     self.set_updated();
// }
//
// pub fn get_screen(&self) -> Screen {
//     self.screen
// }
//
// pub fn get_state(&self) -> StepperSynthState {
//     // info!("get_state called");
//     (*self.updated.lock().unwrap()) = false;
//     // info!("after set");
//
//     let mut seq = self.midi_sequencer.lock().unwrap();
//
//     match self.screen {
//         // Screen::Synth(SynthEngineType::B3Organ) => StepperSynthState::Synth {
//         //     engine: SynthEngineType::B3Organ,
//         //     effect: seq.synth.effect_type,
//         //     effect_on: seq.synth.effect_power,
//         //     knob_params: seq.synth.get_engine().get_params(),
//         //     gui_params: seq.synth.get_engine().get_gui_params(),
//         // },
//         // Screen::Synth(SynthEngineType::SubSynth) => StepperSynthState::Synth {
//         //     engine: SynthEngineType::SubSynth,
//         //     effect: seq.synth.effect_type,
//         //     effect_on: seq.synth.effect_power,
//         //     knob_params: seq.synth.get_engine().get_params(),
//         //     gui_params: seq.synth.get_engine().get_gui_params(),
//         // },
//         Screen::Synth(engine_type) => StepperSynthState::Synth {
//             engine: engine_type,
//             effect: seq.synth.effect_type,
//             effect_on: seq.synth.effect_power,
//             knob_params: seq.synth.get_engine().get_params(),
//             gui_params: seq.synth.get_engine().get_gui_params(),
//         },
//         Screen::Effect(EffectType::Reverb) => StepperSynthState::Effect {
//             effect: EffectType::Reverb,
//             effect_on: seq.synth.effect_power,
//             params: seq.synth.get_effect().get_params(),
//         },
//         Screen::Effect(EffectType::Chorus) => StepperSynthState::Effect {
//             effect: EffectType::Chorus,
//             effect_on: seq.synth.effect_power,
//             params: seq.synth.get_effect().get_params(),
//         },
//         // Screen::Effect(EffectType::Delay) => StepperSynthState::Effect {
//         //     effect: EffectType::Delay,
//         //     effect_on: synth.effect_power,
//         //     params: synth.effect.get_params(),
//         // },
//         Screen::Stepper(sequence) => {
//             // if !seq.state.recording {
//             // seq.rec_head.set
//             // }
//             seq.set_sequence(sequence.abs() as usize);
//
//             StepperSynthState::MidiStepper {
//                 playing: seq.state.playing.load(Ordering::Relaxed),
//                 recording: seq.state.recording,
//                 name: seq.get_name(),
//                 tempo: seq.bpm,
//                 step: seq.get_step(false),
//                 cursor: seq.get_cursor(false),
//                 sequence: seq.get_sequence(),
//                 seq_n: seq.rec_head.get_sequence(),
//             }
//         }
//     }
// }
//
// pub fn set_engine(&mut self, engine: SynthEngineType) {
//     if self.midi_sequencer.lock().unwrap().synth.set_engine(engine) {
//         self.set_updated();
//     }
// }
//
// pub fn set_effect(&mut self, effect: EffectType) {
//     if self.midi_sequencer.lock().unwrap().synth.set_effect(effect) {
//         self.set_updated();
//     }
// }
//
// pub fn set_gui_param(&mut self, param: GuiParam, value: f32) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//
//     match param {
//         GuiParam::A => seq.synth.get_engine().gui_param_1(value),
//         GuiParam::B => seq.synth.get_engine().gui_param_2(value),
//         GuiParam::C => seq.synth.get_engine().gui_param_3(value),
//         GuiParam::D => seq.synth.get_engine().gui_param_4(value),
//         GuiParam::E => seq.synth.get_engine().gui_param_5(value),
//         GuiParam::F => seq.synth.get_engine().gui_param_6(value),
//         GuiParam::G => seq.synth.get_engine().gui_param_7(value),
//         GuiParam::H => seq.synth.get_engine().gui_param_8(value),
//     };
// }
//
// pub fn set_knob_param(&mut self, param: Knob, value: f32) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//     let synth = &mut seq.synth;
//
//     match (param, self.screen) {
//         (Knob::One, Screen::Synth(_)) => synth.get_engine().knob_1(value),
//         (Knob::Two, Screen::Synth(_)) => synth.get_engine().knob_2(value),
//         (Knob::Three, Screen::Synth(_)) => synth.get_engine().knob_3(value),
//         (Knob::Four, Screen::Synth(_)) => synth.get_engine().knob_4(value),
//         (Knob::Five, Screen::Synth(_)) => synth.get_engine().knob_5(value),
//         (Knob::Six, Screen::Synth(_)) => synth.get_engine().knob_6(value),
//         (Knob::Seven, Screen::Synth(_)) => synth.get_engine().knob_7(value),
//         (Knob::Eight, Screen::Synth(_)) => synth.get_engine().knob_8(value),
//         (Knob::One, Screen::Effect(_)) => synth.get_effect().knob_1(value),
//         (Knob::Two, Screen::Effect(_)) => synth.get_effect().knob_2(value),
//         (Knob::Three, Screen::Effect(_)) => synth.get_effect().knob_3(value),
//         (Knob::Four, Screen::Effect(_)) => synth.get_effect().knob_4(value),
//         (Knob::Five, Screen::Effect(_)) => synth.get_effect().knob_5(value),
//         (Knob::Six, Screen::Effect(_)) => synth.get_effect().knob_6(value),
//         (Knob::Seven, Screen::Effect(_)) => synth.get_effect().knob_7(value),
//         (Knob::Eight, Screen::Effect(_)) => synth.get_effect().knob_8(value),
//         (_, Screen::Stepper(_)) => false,
//     };
// }
//
// // /// increments the record head to the next step
// // pub fn next_step(&mut self) {}
//
// // /// sets the record head to a step
// // pub fn set_rec_head_step(&mut self, step: usize) {}
//
// pub fn start_recording(&mut self) {
//     self.set_updated();
//
//     // self.midi_sequencer
//     //     .lock()
//     //     .unwrap()
//     //     .state
//     //     .playing
//     //     .store(false, Ordering::Relaxed);
//     self.midi_sequencer.lock().unwrap().state.recording = true;
// }
//
// pub fn stop_seq(&mut self) {
//     self.set_updated();
//
//     self.midi_sequencer
//         .lock()
//         .unwrap()
//         .state
//         .playing
//         .store(false, Ordering::Relaxed);
//     self.midi_sequencer.lock().unwrap().state.recording = false;
// }
//
// pub fn start_playing(&mut self) {
//     self.set_updated();
//
//     self.midi_sequencer
//         .lock()
//         .unwrap()
//         .state
//         .playing
//         .store(true, Ordering::Relaxed);
//     self.midi_sequencer.lock().unwrap().state.recording = false;
// }
//
// pub fn prev_sequence(&mut self) {
//     self.midi_sequencer.lock().unwrap().prev_sequence();
//     self.set_updated();
//
//     // match self.screen.clone() {
//     //     Screen::Stepper(s) => {
//     //         Screen::Stepper(self.synth.lock().unwrap().midi_sequencer.rec_head.);
//     //     }
//     // }
// }
//
// pub fn next_sequence(&mut self) {
//     self.midi_sequencer.lock().unwrap().next_sequence();
//     self.set_updated()
// }
//
// pub fn next_step(&mut self) {
//     self.midi_sequencer.lock().unwrap().next_step();
//     self.set_updated()
// }
//
// pub fn prev_step(&mut self) {
//     self.midi_sequencer.lock().unwrap().prev_step();
//     self.set_updated()
// }
//
// pub fn tempo_up(&mut self) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//
//     seq.bpm = (seq.bpm + 1) % u16::MAX;
//
//     if seq.bpm == 0 {
//         seq.bpm = 1;
//     }
// }
//
// pub fn tempo_down(&mut self) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//
//     if seq.bpm > 0 {
//         seq.bpm = (seq.bpm - 1) % u16::MAX;
//     }
//
//     if seq.bpm == 0 {
//         seq.bpm = 1;
//     }
// }
//
// pub fn add_step(&mut self) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//     seq.add_step();
// }
//
// pub fn del_step(&mut self) {
//     self.set_updated();
//     let mut seq = self.midi_sequencer.lock().unwrap();
//     seq.del_step();
// }
// }
