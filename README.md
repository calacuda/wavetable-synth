# wavetable-synth

Wavetable audio synthesis crate and audio plugin written in rust. supports LFOs, Envelopes, filters, & a Mod-Matrix.

This project is modeled after [Vital](https://vital.audio/). It is desined to be used as either a rust crate, or as an audio plugin (VST3 & Clap specifically).

## Audio Samples

https://github.com/user-attachments/assets/9aba5e1f-4ebd-4d82-92ab-b97708481b46

<!-- https://github.com/user-attachments/assets/957e734e-4d97-46eb-b9ed-6a1b9170ad14 /-->

[octave-down-oscilator.wav](https://gabalpha.github.io/read-audio/?p=https://github.com/calacuda/wavetable-synth/blob/main/audio-samples/octave-down-oscilator.wav)

## Project Structure

| **Directory** | **Whats There** |
|----|----|
|`./wavetable-synth.lib`| the underlying library that can be used as a rust crate and also the backing for the plugin |
|`./wavetable-synth.plugin`| contains the code for the VST3 & Clap plugins as well as the stand alone verison |
|`./audio-samples`| stores example audio files |

## What is Wavetable Synthesis

## Plugin TODOs

- [ ] mk GUI
  - [ ] enable switching wavetables
  - [ ] enable param editing from GUI
  - [ ] mod matrix routing from GUI
  - [ ] adding MIDI CC learning/setting
- [x] add github-actions to build & linux, MacOS, & Windows builds.
  - [x] build
  - [x] add release tag

## Lib TODOs

- [x] enable detune
- [x] enable mod matrix routing
- [ ] add Macros
