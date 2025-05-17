#!/usr/bin/env bash

COMMANDS=(
    'uci / isready / - / ucinewgame / isready'
    # 'position startpos moves d2d4'
    # 'position startpos moves d2d4 g8f6 e2e3'
    # 'position startpos moves d2d4 g8f6 e2e3 e7e6 g1f3'
    # 'position startpos moves d2d4 g8f6 e2e3 e7e6 g1f3 d7d5 f3e5'
    'position startpos moves d2d4 g8f6 e2e3 e7e6 g1f3 d7d5 f3e5 c7c5 d1h5'
    # 'position startpos moves d2d4 g8f6 e2e3 e7e6 g1f3 d7d5 f3e5 c7c5 d1h5 f6h5 e5f7'
    'go infinite / -- / stop'
)

split_multi_command() {
    tr / $'\n' | gsed -E 's/\s*(.*)\s*/\1/'
}

run_command() {
    case "$1" in
        -*) eval "sleep ${#cmd}" ;;
        */*) echo "$1" | split_multi_command | while read cmd; do run_command "$cmd"; done ;;
        *) echo "> $cmd" >&2; echo "$cmd" ;;
    esac
}

for cmd in "${COMMANDS[@]}"; do run_command "$cmd"; done
