# Wt Synth

Wavetable audio synthesis plugin written in rust. supports LFOs, Envelopes, filters, & a Mod-Matrix.

## Building

After installing [Rust](https://rustup.rs/), you can compile Wt Synth as follows:

```shell
cargo xtask bundle wt_synth --release
```

  Optionally: you may add: `-F standalone`, to get a standalong binary

You will then find the compiled plugins at:

standalone => 'target/bundled/Wt Synth'
CLAP => 'target/bundled/Wt Synth.clap'
VST3 => 'target/bundled/Wt Synth.vst3'
