# wavetable-synth

Wavetable audio synthesis crate and plugin written in rust. supports LFOs, Envelopes, filters, & a Mod-Matrix.

this project is modeled after [Vital](https://vital.audio/)

# Project Structure

| **Directory** | **Whats There** |
|----|----|
|`wavetable-synth.lib`| the underlying library that can be used as a rust crate and also the backing for the plugin |
|`wavetable-synth.plugin`| contains the code for the VST3 & Clap plugins as well as the stand alone verison |

# Plugin TODOs

- [ ] mk GUI
  - [ ] enable switching wavetables
  - [ ] enable param editing from GUI
  - [ ] mod matrix routing from GUI
  - [ ] adding MIDI CC learning/setting
- [x] add github-actions to build & linux, MacOS, & Windows builds.
  - [x] build
  - [x] add release tag

# Lib TODOs

- [x] enable detune
- [x] enable mod matrix routing
- [ ] add Macros
