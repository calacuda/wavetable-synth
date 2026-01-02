_:
  @just -l

build:
  cargo xtask bundle wt_synth --release -F nih_plug/standalone

install: build
  cp -rf ./target/bundled/Wt\ Synth.vst3 ~/.vst3
  cp -rf ./target/bundled/Wt\ Synth.clap ~/.clap
  cp -rf ./target/bundled/Wt\ Synth ~/.local/bin/wt-synth

