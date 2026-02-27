use nih_plug::prelude::*;
use wt_synth::WtSynth;

// mod lib;

fn main() {
    #[cfg(feature = "standalone")]
    nih_export_standalone::<WtSynth>();
    #[cfg(not(feature = "standalone"))]
    println!("Binary was compiled without suport for a standalone app. Compile with the \"standalone\" feature enabled to get a standalone app.");
}
