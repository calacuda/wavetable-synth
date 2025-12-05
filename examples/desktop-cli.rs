use log::error;
use std::sync::{Arc, RwLock};
use tinyaudio::{run_output_device, OutputDeviceParameters};
use wavetable_synth::{config::SAMPLE_RATE, logger_init, run_midi, App, SampleGen};

fn main() -> anyhow::Result<()> {
    logger_init()?;

    let app = Arc::new(RwLock::new(App::default()));

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
                if let Ok(mut synth) = synth.write() {
                    let value = synth.get_sample();

                    for sample in samples {
                        *sample = value;
                    }
                }
            }
        }
    });

    if let Err(e) = device {
        error!("strating audio playback caused error: {e}");
    }

    // if let Ok(mut synth) = app.lock() {
    //     synth.play(48, 97);
    // }

    run_midi(app.clone())?;

    Ok(())
}
