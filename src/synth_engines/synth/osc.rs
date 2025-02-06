use crate::{
    calculate_modulation,
    common::OscParam,
    config::{SAMPLE_RATE, WAVE_TABLE_SIZE},
    midi_to_freq, ModulationDest, SampleGen, WaveTable,
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

#[derive(Clone, Debug)]
pub struct Oscillator {
    osc: WavetableOscillator,
    frequency: f32,
    base_frequency: f32,
    level: f32,
    level_mod: f32,
    // pan: f32,
    detune: f32,
    detune_mod: f32,
    offset: i16,
    pub target: OscTarget,
    pub wave_table: WaveTable,
}

impl Oscillator {
    pub fn new(wave_table: WaveTable) -> Self {
        Self {
            osc: WavetableOscillator::new(),
            frequency: 0.0,
            base_frequency: 0.0,
            level: 0.75,
            level_mod: 0.0,
            detune: 0.0,
            detune_mod: 0.0,
            offset: 0,
            target: OscTarget::Filter1,
            wave_table,
        }
    }

    pub fn press(&mut self, midi_note: u8) {
        let note = midi_note as i16 + self.offset;

        self.frequency = midi_to_freq(note);
        self.base_frequency = self.frequency;

        self.osc.set_frequency(self.frequency);
    }

    pub fn release(&mut self) {}

    pub fn get_sample(&mut self) -> f32 {
        // if self.tune != 0.0 {
        // self.detune(tune)
        // }
        self.detune();

        self.osc.get_sample(&self.wave_table) * calculate_modulation(self.level, self.level_mod)
    }

    pub fn detune(&mut self) {
        // println!("bending");
        let amt = calculate_modulation(self.detune, self.detune_mod);

        if amt == 0.0 {
            return;
        };

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

impl ModulationDest for Oscillator {
    type ModTarget = OscParam;

    fn modulate(&mut self, what: Self::ModTarget, by: f32) {
        match what {
            Self::ModTarget::Level => self.level_mod = by,
            Self::ModTarget::Tune => self.detune_mod = by,
        }
    }

    fn reset(&mut self) {
        self.level_mod = 0.0;
        self.detune_mod = 0.0;
    }
}

impl SampleGen for Oscillator {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}

// #[derive(Clone, Debug)]
// pub struct NoteOscillator {
//     // notes: [Option<f32>; POLYPHONY],
//     osc_s: [Oscillator; POLYPHONY],
// }
//
// impl NoteOscillator {
//     pub fn new(wave_table: WaveTable) -> Self {
//         Self {
//             // notes: [None; POLYPHONY],
//             osc_s: [Oscillator::new(); POLYPHONY],
//         }
//     }
//
//     pub fn press(&mut self, midi_note: u8) {
//         for osc in self.osc_s.iter_mut() {
//             osc.press(midi_note as i16 + self.offset);
//         }
//     }
//
//     pub fn release(&mut self) {}
//
//     pub fn get_sample(&mut self) -> f32 {
//         let mut sample = 0.0;
//
//         for osc in self.osc_s.iter_mut() {
//             sample += osc.get_sample(&self.wave_table, self.detune);
//         }
//
//         sample * self.level
//     }
//
//     pub fn bend(&mut self, bend: f32) {
//         for osc in self.osc_s.iter_mut() {
//             osc.bend(bend);
//         }
//     }
//
//     pub fn unbend(&mut self) {
//         for osc in self.osc_s.iter_mut() {
//             osc.unbend();
//         }
//     }
// }

// impl SampleGen for NoteOscillator {
//     fn get_sample(&mut self) -> f32 {
//         self.get_sample()
//     }
// }
