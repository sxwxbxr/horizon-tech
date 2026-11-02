#!/usr/bin/env python3
"""Draws the mod's 16x16 block textures (original pixel art, made by this script).

Run it after changing a colour: `python3 tools/textures.py` (needs Pillow).
"""
from pathlib import Path
from PIL import Image

OUT = Path(__file__).resolve().parents[1] / "assets/horizon_tech/textures/block"

STEEL = (118, 124, 132)
STEEL_LIGHT = (150, 156, 164)
STEEL_DARK = (78, 82, 90)
FRAME = (52, 55, 61)
COPPER = (196, 108, 72)
COPPER_LIGHT = (231, 150, 106)
COPPER_DARK = (140, 70, 48)
SLOT = (34, 36, 40)
EMBER = (255, 160, 40)
FLAME = (255, 220, 90)
CELL = (70, 200, 110)
CELL_DARK = (40, 120, 70)


def canvas(fill=STEEL):
    return Image.new("RGBA", (16, 16), fill + (255,))


def rect(img, x0, y0, x1, y1, color):
    for x in range(x0, x1 + 1):
        for y in range(y0, y1 + 1):
            img.putpixel((x, y), color + (255,))


def casing(img, edge=FRAME):
    rect(img, 0, 0, 15, 0, edge)
    rect(img, 0, 15, 15, 15, edge)
    rect(img, 0, 0, 0, 15, edge)
    rect(img, 15, 0, 15, 15, edge)
    rect(img, 1, 1, 14, 1, STEEL_LIGHT)
    rect(img, 1, 1, 1, 14, STEEL_LIGHT)
    rect(img, 1, 14, 14, 14, STEEL_DARK)
    rect(img, 14, 1, 14, 14, STEEL_DARK)
    for x, y in ((2, 2), (13, 2), (2, 13), (13, 13)):
        img.putpixel((x, y), STEEL_DARK + (255,))


def machine_side():
    img = canvas()
    casing(img)
    for y in (5, 8, 11):
        rect(img, 4, y, 11, y, STEEL_DARK)
    return img


def machine_top():
    img = canvas()
    casing(img)
    rect(img, 5, 5, 10, 10, STEEL_DARK)
    rect(img, 6, 6, 9, 9, COPPER)
    rect(img, 7, 7, 8, 8, COPPER_LIGHT)
    return img


def generator_front(lit):
    img = canvas()
    casing(img)
    rect(img, 3, 7, 12, 12, SLOT)
    for x in range(4, 12, 2):
        rect(img, x, 7, x, 12, FRAME)
    if lit:
        for x in range(5, 12, 2):
            rect(img, x, 10, x, 12, EMBER)
            img.putpixel((x, 9), FLAME + (255,))
    rect(img, 3, 3, 12, 4, COPPER_DARK)
    rect(img, 4, 3, 11, 3, COPPER)
    return img


def battery_side():
    img = canvas(COPPER_DARK)
    casing(img)
    rect(img, 3, 3, 12, 12, COPPER)
    rect(img, 4, 4, 11, 4, COPPER_LIGHT)
    rect(img, 7, 2, 8, 13, COPPER_DARK)
    return img


def battery_front():
    img = canvas()
    casing(img)
    rect(img, 4, 3, 11, 12, SLOT)
    for i, y in enumerate((4, 6, 8, 10)):
        rect(img, 5, y, 10, y + 1, CELL_DARK if i == 0 else CELL)
    rect(img, 6, 13, 9, 13, COPPER)
    return img


def cable():
    img = canvas(COPPER)
    for x in range(16):
        for y in range(16):
            if (x + y) % 6 == 0:
                img.putpixel((x, y), COPPER_LIGHT + (255,))
            elif (x + y) % 6 == 3:
                img.putpixel((x, y), COPPER_DARK + (255,))
    for x, y in ((5, 5), (10, 5), (5, 10), (10, 10)):
        img.putpixel((x, y), FRAME + (255,))
    return img


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    images = {
        "machine_side": machine_side(),
        "machine_top": machine_top(),
        "coal_generator_front": generator_front(False),
        "coal_generator_front_on": generator_front(True),
        "battery_side": battery_side(),
        "battery_front": battery_front(),
        "energy_cable": cable(),
    }
    for name, img in images.items():
        img.save(OUT / f"{name}.png", optimize=True)
        print(OUT / f"{name}.png")


if __name__ == "__main__":
    main()
