use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Q_BUTTERWORTH_F32};

use crate::{calculate_modulation, common::LowPassParam, config::SAMPLE_RATE, ModulationDest};

#[derive(Clone, Copy, Debug)]
pub struct BQLowPass {
    filter: DirectForm2Transposed<f32>,
    pub cutoff: f32,
    pub resonance: f32,
    pub mix: f32,
    pub note: f32,
    // pub range: (f32, f32),
    pub key_track: bool,
    pub cutoff_mod: f32,
    pub res_mod: f32,
    pub mix_mod: f32,
}

impl BQLowPass {
    pub fn new() -> Self {
        // Cutoff and sampling frequencies
        let f0 = 440.hz();
        let fs = SAMPLE_RATE.hz();

        // Create coefficients for the biquads
        let coeffs =
            Coefficients::<f32>::from_params(biquad::Type::LowPass, fs, f0, Q_BUTTERWORTH_F32)
                .unwrap();

        let filter = DirectForm2Transposed::<f32>::new(coeffs);
        // filter.compute_coeffs(5_000.0, 0.75);

        Self {
            filter,
            cutoff: 0.5,
            resonance: 0.5,
            note: 0.0,
            // range: (0.0, 0.0),
            key_track: true,
            mix: 0.25,
            cutoff_mod: 0.0,
            res_mod: 0.0,
            mix_mod: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn set_resonace(&mut self, res: f32) {
        self.resonance = res;
    }

    pub fn get_sample(&mut self, sample: f32) -> f32 {
        // let nudge = 2.0_f32.powf(19.0 * env * self.cutoff / 12.0);
        // // let delta = (self.note * 2.0) - (self.note / 2.0);
        // // let nudge = delta * env * self.cutoff;
        // let cutoff = if env > 0.1 {
        //     self.note * nudge
        // } else if nudge < 0.1 {
        //     self.note / nudge
        // } else {
        //     self.note
        // };

        let delta = self.note * 12.0;
        let nudge = delta * calculate_modulation(self.cutoff, self.cutoff_mod);
        let cutoff = (self.note) + nudge;

        // warn!("res {}", calculate_modulation(self.resonance, self.res_mod));
        // let mix = calculate_modulation(self.mix, self.mix_mod);

        self.filter.update_coefficients(
            Coefficients::<f32>::from_params(
                biquad::Type::LowPass,
                SAMPLE_RATE.hz(),
                cutoff.hz(),
                Q_BUTTERWORTH_F32,
            )
            .unwrap(),
        );

        // self.filter.process(
        //     sample,
        //     cutoff,
        //     calculate_modulation(self.resonance, self.res_mod),
        // ) * (1.0 - mix)
        //     + sample * mix

        self.filter.run(sample)
    }

    pub fn set_note(&mut self, note: f32) {
        self.note = note;
    }
}

impl ModulationDest for BQLowPass {
    type ModTarget = LowPassParam;

    fn modulate(&mut self, what: Self::ModTarget, by: f32) {
        match what {
            Self::ModTarget::Cutoff => self.cutoff_mod = by,
            Self::ModTarget::Res => self.res_mod = by,
            Self::ModTarget::Mix => self.mix_mod = by,
        }
    }

    fn reset(&mut self) {
        self.cutoff_mod = 0.0;
        self.res_mod = 0.0;
        self.mix_mod = 0.0;
    }
}
