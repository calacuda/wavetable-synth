// use my_plugin::MyPlugin;
use nih_plug::prelude::*;
use wt_synth::WtSynth;

// mod lib;

fn main() {
    nih_export_standalone::<WtSynth>();
}
