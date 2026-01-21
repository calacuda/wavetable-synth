_:
  just -l

_new-window NAME CMD:
  tmux new-w -t wt-synth -n "{{NAME}}"
  tmux send-keys -t wt-synth:"{{NAME}}" "{{CMD}}" ENTER

tmux:
  tmux new -ds wt-synth -n "README"
  tmux send-keys -t wt-synth:README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "Edit-Lib" "cd ./wavetable-synth.lib/"
  @just _new-window "check" "cd ./wavetable-synth.lib/ && cargo check"
  @just _new-window "Edit-Plug" "cd ./wavetable-synth.plugin/"
  @just _new-window "Run" "cd ./wavetable-synth.plugin/"
  @just _new-window "Git" "git status"
  @just _new-window "Misc" ""
  tmux a -t wt-synth
