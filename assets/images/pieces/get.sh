#!/usr/bin/env bash

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

URL='https://images.chesscomfiles.com/chess-themes/pieces/neo/150/'
DEST="$HERE"

EXT=.png

COLOR_CODES=( b w )
declare -A COLOR_MAP
COLOR_MAP[b]=black
COLOR_MAP[w]=white

PIECE_CODES=( r n b k q p )
declare -A PIECE_MAP
PIECE_MAP[r]=rook
PIECE_MAP[n]=knight
PIECE_MAP[b]=bishop
PIECE_MAP[k]=king
PIECE_MAP[q]=queen
PIECE_MAP[p]=pawn

for color in "${COLOR_CODES[@]}"; do
    for piece in "${PIECE_CODES[@]}"; do
        cmd="curl -L '${URL}${color}${piece}${EXT}' -o '${DEST}/${COLOR_MAP[$color]}-${PIECE_MAP[$piece]}${EXT}'"
        echo
        echo "$cmd"
        eval "$cmd"
    done
done

echo