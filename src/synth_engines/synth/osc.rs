use crate::{
    common::POLYPHONY,
    midi_to_freq,
    synth_engines::synth_common::{WaveTable, WAVE_TABLE_SIZE},
    SAMPLE_RATE,
};

pub const N_OVERTONES: usize = 32;

#[derive(Clone, Copy, Debug)]
pub enum OscTarget {
    Filter1,
    Filter2,
    Filter1_2,
    Effects,
    DirectOut,
}

#[derive(Clone, Copy, Debug)]
pub struct WavetableOscillator {
    sample_rate: f32,
    index: f32,
    index_increment: f32,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        // let overtones: Vec<f64> = (1..=N_OVERTONES).map(|i| i as f64).collect();
        // let wave_table = build_sine_table(&overtones);

        Self {
            sample_rate: SAMPLE_RATE as f32,
            index: 0.0,
            index_increment: 0.0,
            // wave_table,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * WAVE_TABLE_SIZE as f32 / self.sample_rate;
        self.index = 0.0;
    }

    pub fn get_sample(&mut self, wave_table: &[f32]) -> f32 {
        let mut sample = 0.0;

        sample += self.lerp(wave_table);

        self.index += self.index_increment;
        self.index %= WAVE_TABLE_SIZE as f32;

        sample * 0.9
    }

    fn lerp(&self, wave_table: &[f32]) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % WAVE_TABLE_SIZE;

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * wave_table[truncated_index]
            + next_index_weight * wave_table[next_index]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NoteOscillator {
    osc: WavetableOscillator,
    frequency: f32,
    base_frequency: f32,
}

impl NoteOscillator {
    pub fn new() -> Self {
        Self {
            osc: WavetableOscillator::new(),
            // wave_table,
            frequency: 0.0,
            base_frequency: 0.0,
        }
    }

    pub fn press(&mut self, midi_note: i16) {
        // self.env_filter.press();
        self.frequency = midi_to_freq(midi_note);
        self.base_frequency = self.frequency;

        self.osc.set_frequency(self.frequency);
        // self.low_pass.set_note(self.frequency);
        // self.playing = Some(midi_note);
    }

    pub fn release(&mut self) {
        // self.env_filter.release();
    }

    pub fn get_sample(&mut self, wave_table: &WaveTable, tune: f32) -> f32 {
        if tune != 0.0 {
            self.detune(tune)
        }

        self.osc.get_sample(wave_table)
    }

    pub fn detune(&mut self, amt: f32) {
        // println!("bending");
        let nudge = 2.0_f32.powf(amt / 12.0);
        let new_freq = if amt < 0.0 {
            self.frequency / nudge
        } else if amt > 0.0 {
            self.frequency * nudge
        } else {
            self.frequency
        };
        // + self.frequency;
        self.osc.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn bend(&mut self, bend: f32) {
        // println!("bending");
        let nudge = 2.0_f32.powf((bend * 3.0) / 12.0);
        let new_freq = if bend < 0.0 {
            self.base_frequency / nudge
        } else if bend > 0.0 {
            self.base_frequency * nudge
        } else {
            self.base_frequency
        };
        // + self.frequency;
        self.osc.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn unbend(&mut self) {
        // println!("unbend => {}", self.base_frequency);
        self.osc.set_frequency(self.base_frequency);
        self.frequency = self.base_frequency;
    }
}

#[derive(Clone, Debug)]
pub struct Oscillator {
    notes: [Option<f32>; POLYPHONY],
    osc_s: [NoteOscillator; POLYPHONY],
    level: f32,
    // pan: f32,
    detune: f32,
    offset: i16,
    pub target: OscTarget,
    pub wave_table: WaveTable,
}

impl Oscillator {
    pub fn new(wave_table: WaveTable) -> Self {
        Self {
            notes: [None; POLYPHONY],
            osc_s: [NoteOscillator::new(); POLYPHONY],
            level: 0.75, // wave_table,
            detune: 0.0,
            offset: 0,
            target: OscTarget::Filter1,
            wave_table,
        }
    }

    pub fn press(&mut self, midi_note: u8) {
        // self.frequency = Self::get_freq(midi_note);

        for (note, osc) in self.notes.iter().zip(self.osc_s.iter_mut()) {
            if note.is_none() {
                osc.press(midi_note as i16 + self.offset);
            }
        }
    }

    pub fn release(&mut self) {
        // self.env_filter.release();
    }

    pub fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        for (note, osc) in self.notes.iter().zip(self.osc_s.iter_mut()) {
            if note.is_some() {
                sample += osc.get_sample(&self.wave_table, self.detune);
            }
        }

        sample * self.level
    }

    pub fn bend(&mut self, bend: f32) {
        for (note, osc) in self.notes.iter().zip(self.osc_s.iter_mut()) {
            if note.is_some() {
                osc.bend(bend);
            }
        }
    }

    pub fn unbend(&mut self) {
        for (note, osc) in self.notes.iter().zip(self.osc_s.iter_mut()) {
            if note.is_some() {
                osc.unbend();
            }
        }
    }
}

// impl SampleGen for WavetableOscillator {
//     fn get_sample(&mut self) -> f32 {
//         self.get_sample()
//     }
// }

// impl SynthOscilatorBackend for WavetableOscillator {
//     fn set_frequency(&mut self, frequency: f32) {
//         self.set_frequency(frequency)
//     }
//
//     fn sync_reset(&mut self) {
//         if self.index > WAVE_TABLE_SIZE as f32 * (5.0 / 12.0)
//         // && self.wave_table[self.index as usize] != 0.0
//         {
//             // warn!("reset wave_table");
//             self.index = 0.0;
//             self.direction = !self.direction;
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct SynthOscillator {
//     osc: SynthBackend,
//     pub env_filter: ADSR,
//     /// what midi note is being played by this osc
//     pub playing: Option<u8>,
//     frequency: f32,
//     base_frequency: f32,
//     // note_space: f32,
//     pub low_pass: LowPass,
//     // pub wave_table: WaveTable,
// }
//
// impl SynthOscillator {
//     pub fn new() -> Self {
//         Self {
//             // osc: SynthBackend::Sin(WavetableOscillator::new()),
//             osc: SynthBackend::Saw(SawToothOsc::new()),
//             env_filter: ADSR::new(),
//             playing: None,
//             frequency: 0.0,
//             base_frequency: 0.0,
//             // note_space: 2.0_f32.powf(1.0 / 12.0),
//             low_pass: LowPass::new(),
//         }
//     }
//
//     pub fn sync_reset(&mut self) {
//         self.osc.sync_reset()
//     }
//
//     pub fn set_osc_type(&mut self, osc_type: OscType) {
//         self.osc = osc_type.into();
//     }
//
//     pub fn is_pressed(&self) -> bool {
//         self.env_filter.pressed()
//     }
//
//     pub fn press(&mut self, midi_note: u8) {
//         self.env_filter.press();
//         self.frequency = Self::get_freq(midi_note);
//         self.base_frequency = self.frequency;
//
//         self.osc.set_frequency(self.frequency);
//         self.low_pass.set_note(self.frequency);
//         self.playing = Some(midi_note);
//     }
//
//     fn get_freq(midi_note: u8) -> f32 {
//         let exp = (f32::from(midi_note) + 36.376_316) / 12.0;
//         // 2_f32.powf(exp)
//
//         2.0_f32.powf(exp)
//     }
//
//     pub fn release(&mut self) {
//         self.env_filter.release();
//         // self.playing = None;
//     }
//
//     pub fn get_sample(&mut self) -> f32 {
//         let env = self.env_filter.get_samnple();
//         let sample = self.osc.get_sample() * env;
//
//         if env <= 0.0 {
//             self.playing = None;
//         }
//         // println!("osc sample => {sample}");
//
//         self.low_pass.get_sample(sample, env)
//     }
//
//     pub fn bend(&mut self, bend: f32) {
//         // println!("bending");
//         let nudge = 2.0_f32.powf((bend * 3.0).abs() / 12.0);
//         let new_freq = if bend < 0.0 {
//             self.base_frequency / nudge
//         } else if bend > 0.0 {
//             self.base_frequency * nudge
//         } else {
//             self.base_frequency
//         };
//         // + self.frequency;
//         self.osc.set_frequency(new_freq);
//         // println!("frequency => {}", self.frequency);
//         // println!("new_freq => {}", new_freq);
//         self.frequency = new_freq;
//     }
//
//     pub fn unbend(&mut self) {
//         // println!("unbend => {}", self.base_frequency);
//         self.osc.set_frequency(self.base_frequency);
//         self.frequency = self.base_frequency;
//     }
// }
