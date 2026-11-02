#!/usr/bin/env python3
"""Draws the mod's 16x16 block textures (original pixel art, made by this script).

Run it after changing a colour: `python3 tools/textures.py` (needs Pillow).
"""
from pathlib import Path
import random
from PIL import Image

TEXTURES = Path(__file__).resolve().parents[1] / "assets/horizon_tech/textures"
OUT = TEXTURES / "block"
ITEMS = TEXTURES / "item"

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
COBALT = (58, 96, 196)
COBALT_LIGHT = (110, 152, 236)
COBALT_DARK = (32, 54, 120)
VIOLET = (120, 60, 170)
VIOLET_LIGHT = (176, 110, 226)
VIOLET_DARK = (70, 32, 104)
STONE = (126, 126, 126)
DEEPSLATE = (78, 78, 84)
HEAT = (255, 96, 40)
METALS = {
    "cobalt": (COBALT_DARK, COBALT, COBALT_LIGHT),
    "iron": ((150, 140, 132), (206, 198, 190), (240, 236, 230)),
    "gold": ((176, 128, 24), (238, 196, 52), (255, 240, 140)),
    "copper": (COPPER_DARK, COPPER, COPPER_LIGHT),
}


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


def machine_side(trim=None):
    img = canvas()
    casing(img)
    for y in (5, 8, 11):
        rect(img, 4, y, 11, y, STEEL_DARK)
    if trim:
        dark, mid, light = trim
        rect(img, 1, 1, 14, 2, mid)
        rect(img, 1, 13, 14, 14, dark)
        rect(img, 1, 1, 14, 1, light)
    return img


def machine_top():
    img = canvas()
    casing(img)
    rect(img, 5, 5, 10, 10, STEEL_DARK)
    rect(img, 6, 6, 9, 9, COPPER)
    rect(img, 7, 7, 8, 8, COPPER_LIGHT)
    return img


def generator_front(lit, trim=(COPPER_DARK, COPPER, COPPER_LIGHT)):
    dark, mid, _ = trim
    img = canvas()
    casing(img)
    rect(img, 3, 7, 12, 12, SLOT)
    for x in range(4, 12, 2):
        rect(img, x, 7, x, 12, FRAME)
    if lit:
        for x in range(5, 12, 2):
            rect(img, x, 10, x, 12, EMBER)
            img.putpixel((x, 9), FLAME + (255,))
    rect(img, 3, 3, 12, 4, dark)
    rect(img, 4, 3, 11, 3, mid)
    return img


def tier_front(lit, trim, lights):
    """A higher tier's generator: its trim colour and a lamp per tier."""
    img = generator_front(lit, trim)
    for i in range(lights):
        img.putpixel((5 + 3 * i, 5), (FLAME if lit else STEEL_DARK) + (255,))
    return img


def crusher_front(lit):
    img = canvas()
    casing(img)
    rect(img, 3, 3, 12, 12, SLOT)
    # Two rollers with teeth, turned a step when running.
    shift = 1 if lit else 0
    for x0 in (3, 8):
        rect(img, x0, 4, x0 + 4, 11, STEEL_DARK)
        for y in range(4 + shift, 12, 2):
            rect(img, x0 + 1, y, x0 + 3, y, STEEL_LIGHT)
    rect(img, 7, 3, 8, 12, FRAME)
    rect(img, 3, 13, 12, 13, COBALT if lit else COBALT_DARK)
    return img


def smelter_front(lit):
    img = canvas()
    casing(img)
    rect(img, 3, 3, 12, 12, SLOT)
    for y in range(4, 12, 2):
        rect(img, 4, y, 11, y, (HEAT if lit else COPPER_DARK))
    if lit:
        rect(img, 6, 6, 9, 9, FLAME)
    rect(img, 3, 13, 12, 13, COBALT if lit else COBALT_DARK)
    return img


def rock(base, seed):
    rng = random.Random(seed)
    img = Image.new("RGBA", (16, 16))
    for x in range(16):
        for y in range(16):
            d = rng.choice((-14, -8, -4, 0, 0, 4, 8))
            img.putpixel((x, y), tuple(max(0, min(255, c + d)) for c in base) + (255,))
    return img


def ore(base, seed):
    img = rock(base, seed)
    dark, mid, light = METALS["cobalt"]
    for cx, cy in ((4, 4), (11, 6), (6, 11), (12, 12)):
        for dx, dy in ((0, 0), (1, 0), (0, 1), (-1, 0), (1, 1)):
            img.putpixel((cx + dx, cy + dy), mid + (255,))
        img.putpixel((cx, cy), light + (255,))
        img.putpixel((cx + 1, cy + 1), dark + (255,))
    return img


def storage_block(metal, raw):
    dark, mid, light = METALS[metal]
    img = canvas(mid)
    if raw:
        rng = random.Random(7)
        for x in range(16):
            for y in range(16):
                img.putpixel((x, y), rng.choice((dark, mid, mid, light)) + (255,))
        return img
    rect(img, 0, 0, 15, 0, light)
    rect(img, 0, 0, 0, 15, light)
    rect(img, 0, 15, 15, 15, dark)
    rect(img, 15, 0, 15, 15, dark)
    rect(img, 3, 3, 12, 12, dark)
    rect(img, 4, 4, 11, 11, mid)
    return img


def blank():
    return Image.new("RGBA", (16, 16), (0, 0, 0, 0))


def ingot(metal):
    dark, mid, light = METALS[metal]
    img = blank()
    for row, (x0, x1) in enumerate(((5, 12), (4, 12), (3, 11), (2, 10), (2, 9))):
        y = 6 + row
        rect(img, x0, y, x1, y, mid)
        img.putpixel((x0, y), light + (255,))
        img.putpixel((x1, y), dark + (255,))
    rect(img, 5, 6, 12, 6, light)
    rect(img, 2, 11, 9, 11, dark)
    return img


def raw_ore(metal):
    dark, mid, light = METALS[metal]
    img = blank()
    rng = random.Random(3)
    for x in range(3, 13):
        for y in range(4, 13):
            if (x - 7.5) ** 2 + (y - 8.5) ** 2 * 1.3 < 22:
                img.putpixel((x, y), rng.choice((dark, mid, mid, light)) + (255,))
    return img


def dust(metal):
    dark, mid, light = METALS[metal]
    img = blank()
    rng = random.Random(11)
    for y in range(7, 14):
        half = (y - 6) * 5 // 7 + 1
        for x in range(8 - half, 8 + half):
            img.putpixel((x, y), rng.choice((dark, mid, mid, light)) + (255,))
    for x, y in ((4, 5), (11, 4), (12, 9), (3, 10)):
        img.putpixel((x, y), mid + (255,))
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
    ITEMS.mkdir(parents=True, exist_ok=True)
    cobalt = METALS["cobalt"]
    violet = (VIOLET_DARK, VIOLET, VIOLET_LIGHT)
    images = {
        "reinforced_side": machine_side(cobalt),
        "overcharged_side": machine_side(violet),
        "reinforced_generator_front": tier_front(False, cobalt, 2),
        "reinforced_generator_front_on": tier_front(True, cobalt, 2),
        "overcharged_generator_front": tier_front(False, violet, 3),
        "overcharged_generator_front_on": tier_front(True, violet, 3),
        "crusher_front": crusher_front(False),
        "crusher_front_on": crusher_front(True),
        "induction_smelter_front": smelter_front(False),
        "induction_smelter_front_on": smelter_front(True),
        "cobalt_ore": ore(STONE, 1),
        "deepslate_cobalt_ore": ore(DEEPSLATE, 2),
        "cobalt_block": storage_block("cobalt", False),
        "raw_cobalt_block": storage_block("cobalt", True),
        "machine_side": machine_side(),
        "machine_top": machine_top(),
        "coal_generator_front": generator_front(False),
        "coal_generator_front_on": generator_front(True),
        "battery_side": battery_side(),
        "battery_front": battery_front(),
        "energy_cable": cable(),
    }
    items = {"raw_cobalt": raw_ore("cobalt"), "cobalt_ingot": ingot("cobalt")}
    for metal in METALS:
        items[f"{metal}_dust"] = dust(metal)
    for folder, group in ((OUT, images), (ITEMS, items)):
        for name, img in group.items():
            img.save(folder / f"{name}.png", optimize=True)
            print(folder / f"{name}.png")


if __name__ == "__main__":
    main()
