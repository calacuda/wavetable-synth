use log::error;
use std::{
    sync::{Arc, Mutex},
    // thread::spawn,
};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use wavetable_synth::{
    App, SampleGen,
    common::{
        LfoParam, LowPass, LowPassParam, ModMatrixDest, ModMatrixItem, ModMatrixSrc, OscParam,
    },
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
        voice.press(48, 60);

        Arc::new(Mutex::new(voice))
    };
    let mut mod_matrix: [Option<ModMatrixItem>; 256] = [None; 256];
    mod_matrix[0] = Some(ModMatrixItem {
        src: ModMatrixSrc::Lfo(0),
        dest: ModMatrixDest::LowPass {
            low_pass: LowPass::LP1,
            param: LowPassParam::Res,
        },
        // dest: ModMatrixDest::Env {
        //     env: 0,
        //     param: EnvParam::Sus,
        // },
        amt: 0.5,
        bipolar: true,
    });
    mod_matrix[1] = Some(ModMatrixItem {
        src: ModMatrixSrc::Env(1),
        dest: ModMatrixDest::LowPass {
            low_pass: LowPass::LP1,
            param: LowPassParam::Cutoff,
        },
        amt: 1.0,
        bipolar: false,
    });
    // mod_matrix[3] = Some(ModMatrixItem {
    //     src: ModMatrixSrc::Lfo(0),
    //     dest: ModMatrixDest::Osc {
    //         osc: 0,
    //         param: OscParam::Level,
    //     },
    //     amt: 0.5,
    //     bipolar: true,
    // });
    // mod_matrix[4] = Some(ModMatrixItem {
    //     src: ModMatrixSrc::Env(1),
    //     dest: ModMatrixDest::Lfo {
    //         lfo: 0,
    //         param: LfoParam::Speed,
    //     },
    //     amt: 1.0,
    //     bipolar: false,
    // });

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
                let value = synth.lock().unwrap().get_sample(&mod_matrix);
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

    // Ok(())
}
