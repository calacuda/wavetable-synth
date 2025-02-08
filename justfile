_:
  just -l

check-all: _check-desktop _check-embeded
  
_check-desktop:
  cargo check --no-default-features -F desktop && echo "Desktop Check Successful" | lolcat || echo "Desktop Check FAILED" | lolcat

_check-embeded:
  cargo check --no-default-features -F embeded 2> /dev/null && echo "Embeded Check Successful" | lolcat || echo "Embeded Check Failed" | lolcat

_new-window NAME CMD:
  tmux new-w -t wt-synth -n "{{NAME}}"
  tmux send-keys -t wt-synth:"{{NAME}}" "{{CMD}}" ENTER

tmux:
  tmux new -ds wt-synth -n "README"
  tmux send-keys -t wt-synth:README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "Edit" ""
  @just _new-window "Run" ""
  @just _new-window "Git" "git status"
  @just _new-window "Misc" ""
  tmux a -t wt-synth
