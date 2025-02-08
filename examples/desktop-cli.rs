use log::error;
use std::{
    sync::{Arc, Mutex},
    // thread::spawn,
};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use wavetable_synth::{App, SampleGen, config::SAMPLE_RATE, logger_init, run_midi};

fn main() -> anyhow::Result<()> {
    logger_init()?;

    let app = Arc::new(Mutex::new(App::default()));

    let params = OutputDeviceParameters {
        channels_count: 1,
        sample_rate: SAMPLE_RATE as usize,
        // channel_sample_count: 2048,
        channel_sample_count: 1024,
    };
    let device = run_output_device(params, {
        let synth = app.clone();

        move |data| {
            for samples in data.chunks_mut(params.channels_count) {
                let value = synth.lock().unwrap().get_sample();
                // info!("value {value}");

                for sample in samples {
                    *sample = value;
                }
            }
        }
    });

    if let Err(e) = device {
        error!("strating audio playback caused error: {e}");
    }

    run_midi(app.clone())?;

    Ok(())
}
