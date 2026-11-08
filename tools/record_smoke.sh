#!/bin/sh
# records a smoke test (race only) through hidden_run and prints the mp4 path.
# usage: tools/record_smoke.sh [name] [extra hidden_run --env K=V ...]   (default name smoke_race)
# VELOCITY_EXE=target/<dir>/debug/velocity.exe picks another build
cd "$(dirname "$0")/.." || exit 1
name=${1:-smoke_race}; [ $# -gt 0 ] && shift
dir=research/video/$name
mkdir -p research/video
game=${VELOCITY_GAME_DIR:-"D:/games/xbox/games/project velocity debug"}
.venv-ida/Scripts/python tools/hidden_run.py --timeout 300 --env SMOKE_RECORD="$dir" --env SMOKE_RECORD_RACE=1 \
  --env SMOKE_PREFIX="rec" "$@" -- "${VELOCITY_EXE:-target/debug/velocity.exe}" --smoke-test --game "$game" > "research/video/$name.log" 2>&1
code=$?
[ -f "$dir.mp4" ] && echo "$dir.mp4" || { echo "no mp4 (exit $code), see research/video/$name.log" >&2; exit 1; }
