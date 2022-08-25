#!/usr/bin/env python3

from pathlib import Path
from PIL import Image

HERE = Path(__file__).parent

tiles = {
    'black': '#769656',
    'white': '#eeeed2',
}

for (name, color) in tiles.items():
    Image.new(mode='RGB', size=(150,150), color=color).save(HERE / (name + '.png'))
