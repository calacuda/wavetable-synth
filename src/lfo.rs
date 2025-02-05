use crate::{
    config::{SAMPLE_RATE, WAVE_TABLE_SIZE},
    SampleGen, WaveTable,
};

/// the WaveTable oscilator that is used for generating LFO samples
#[derive(Clone, Debug)]
pub struct LfoWaveTableOsc {
    sample_rate: f32,
    index: f32,
    index_increment: f32,
    wave_table_len: f32,
    freq: f32,
}

impl LfoWaveTableOsc {
    pub fn new() -> Self {
        Self {
            sample_rate: SAMPLE_RATE as f32,
            index: 0.0,
            index_increment: 0.0,
            wave_table_len: WAVE_TABLE_SIZE as f32,
            freq: 2.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.freq = frequency;
        self.calc_index_inc();
    }

    fn calc_index_inc(&mut self) {
        self.index_increment = self.freq * self.wave_table_len / self.sample_rate;
    }

    pub fn set_wave_table_size(&mut self, size: usize) {
        self.wave_table_len = size as f32;
        self.calc_index_inc();
        self.index = 0.0;
    }

    pub fn get_sample(&mut self, wave_table: &[f32]) -> f32 {
        let sample = self.lerp(wave_table);

        self.index += self.index_increment;
        self.index %= self.wave_table_len;

        sample * 0.95
    }

    pub fn press(&mut self) {
        self.index = 0.0;
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

/// the actual LFO struct
#[derive(Clone, Debug)]
pub struct LFO {
    /// can be modulated by envelopes, lfos, velocity, etc
    freq: f32,
    wave_table: WaveTable,
    pub osc: LfoWaveTableOsc,
    playing: bool,
}

impl LFO {
    pub fn new() -> Self {
        let wave_table = mk_default_lfo_wt();

        Self {
            freq: 2.0,
            wave_table,
            osc: LfoWaveTableOsc::new(),
            playing: false,
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        if self.playing {
            self.osc.get_sample(&self.wave_table)
        } else {
            0.0
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.freq = frequency;
        self.osc.set_frequency(frequency);
    }

    pub fn set_wave_table(&mut self, wave_table: WaveTable) {
        self.wave_table = wave_table;
        self.osc.set_wave_table_size(self.wave_table.len());
    }

    pub fn press(&mut self) {
        self.osc.press();
        self.osc.index = 0.0;
        self.playing = true;
    }

    pub fn release(&mut self) {
        self.playing = false;
    }
}

impl SampleGen for LFO {
    fn get_sample(&mut self) -> f32 {
        self.osc.get_sample(&self.wave_table)
    }
}

fn mk_default_lfo_wt() -> WaveTable {
    let size = WAVE_TABLE_SIZE * 4;
    let half_way = size as f32 / 2.0;
    let slope = 2.0 / size as f32;
    let f = |i: usize| slope * i as f32;

    (0..size)
        .map(|i| {
            if i as f32 <= half_way {
                f(i)
            } else {
                1.0 - f(i)
            }
        })
        .collect()
}
