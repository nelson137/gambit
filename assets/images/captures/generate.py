#!/usr/bin/env python3

from pathlib import Path

from PIL import Image

HERE = Path(__file__).parent

EXT = '.png'

# (min.x, min.y, max.x, max.y)
# (left, top, right, bottom)
sprite_boxes: dict[str, (int, int, int, int)] = {
    'black-pawns-8': (0, 0, 124, 36),
    'black-pawns-7': (0, 50, 110, 86),
    'black-pawns-6': (0, 100, 96, 136),
    'black-pawns-5': (0, 150, 82, 186),
    'black-pawns-4': (0, 200, 68, 236),
    'black-pawns-3': (0, 250, 54, 286),
    'black-pawns-2': (0, 300, 40, 336),
    'black-pawns-1': (0, 350, 26, 386),

    'black-bishops-2': (134, 0, 180, 36),
    'black-bishops-1': (134, 50, 164, 86),

    'black-knights-2': (190, 0, 235, 36),
    'black-knights-1': (190, 50, 221, 86),

    'black-rooks-2': (240, 0, 286, 36),
    'black-rooks-1': (240, 50, 271, 86),

    'black-queen': (289, 0, 324, 36),

    'white-pawns-8': (360, 0, 484, 36),
    'white-pawns-7': (360, 50, 470, 86),
    'white-pawns-6': (360, 100, 456, 136),
    'white-pawns-5': (360, 150, 442, 186),
    'white-pawns-4': (360, 200, 428, 236),
    'white-pawns-3': (360, 250, 414, 286),
    'white-pawns-2': (360, 300, 400, 336),
    'white-pawns-1': (360, 350, 386, 386),

    'white-bishops-2': (494, 0, 539, 36),
    'white-bishops-1': (494, 50, 523, 86),

    'white-knights-2': (550, 0, 594, 36),
    'white-knights-1': (550, 50, 580, 86),

    'white-rooks-2': (600, 0, 645, 36),
    'white-rooks-1': (600, 50, 630, 86),

    'white-queen': (649, 0, 683, 36),
}

# The keys of the desired images to generate
allow_write = []

with Image.open(HERE / 'captured-pieces.png') as sprite_sheet:
    for (basename, box) in sprite_boxes.items():
        if basename in allow_write:
            sprite_sheet.crop(box).save(HERE / (basename + EXT))
