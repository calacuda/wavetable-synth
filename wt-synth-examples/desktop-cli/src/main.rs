use log::error;
use std::{
    sync::{Arc, Mutex},
    // thread::spawn,
};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use wavetable_synth::{
    App, SampleGen,
    common::ModMatrixItem,
    config::SAMPLE_RATE,
    logger_init, run_midi,
    synth_engines::synth::{build_sine_table, osc::N_OVERTONES},
    voice::Voice,
};

fn main() -> anyhow::Result<()> {
    logger_init()?;

    // let app = Arc::new(Mutex::new(App::default()));
    // app.press();
    let voice = {
        let mut overtones = [0.0; N_OVERTONES];
        (0..N_OVERTONES).for_each(|i| overtones[i] = (i + 1) as f64);
        let wave_table = build_sine_table(&overtones);
        let mut voice = Voice::new(wave_table);
        voice.press(60, 60);

        Arc::new(Mutex::new(voice))
    };
    let mod_matrix: [Option<ModMatrixItem>; 255] = [None; 255];

    let params = OutputDeviceParameters {
        channels_count: 1,
        sample_rate: SAMPLE_RATE as usize,
        // channel_sample_count: 2048,
        channel_sample_count: 1024,
    };
    let device = run_output_device(params, {
        // let synth = app.clone();
        let synth = voice.clone();

        move |data| {
            for samples in data.chunks_mut(params.channels_count) {
                let value = synth.lock().unwrap().get_sample(&mod_matrix).unwrap();
                // let value = synth.lock().unwrap().get_sample();
                // info!("value {value}");

                for sample in samples {
                    // *sample = value.unwrap();
                    *sample = value;
                }
            }
        }
    });

    if let Err(e) = device {
        error!("starting audio playback caused error: {e}");
    }

    // run_midi(app.clone())?;
    loop {}

    Ok(())
}
