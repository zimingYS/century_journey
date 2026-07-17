from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
BLOCK_TEXTURES = ROOT / "assets" / "textures" / "blocks"
SIZE = 16


def mix(color, target, amount):
    return tuple(
        round(channel * (1.0 - amount) + target_channel * amount)
        for channel, target_channel in zip(color[:3], target)
    ) + (255,)


def shade(color, delta):
    return tuple(max(0, min(255, channel + delta)) for channel in color[:3]) + (255,)


def palette(reference):
    colors = sorted(
        reference.convert("RGBA").get_flattened_data(),
        key=lambda color: color[0] + color[1] + color[2],
    )
    return colors[len(colors) // 12], colors[len(colors) // 2], colors[-len(colors) // 10]


def make_planks(side, top):
    dark, mid, light = palette(top)
    image = Image.new("RGBA", (SIZE, SIZE))
    for y in range(SIZE):
        board = y // 4
        for x in range(SIZE):
            source = side.getpixel(((x + board * 5) % SIZE, (y * 3 + board * 2) % SIZE))
            color = mix(source, mid[:3], 0.58)
            color = shade(color, (2, 7, -3, 4)[board])
            if y % 4 == 0:
                color = mix(color, light[:3], 0.2)
            if y % 4 == 3:
                color = mix(color, dark[:3], 0.62)
            if (x + board * 3) % 7 == 1 and y % 4 in (1, 2):
                color = mix(color, dark[:3], 0.28)
            image.putpixel((x, y), color)
    return image


def make_workbench_top(planks, top):
    dark, _, light = palette(top)
    image = planks.copy()
    pixels = image.load()
    for coordinate in range(SIZE):
        pixels[coordinate, 0] = mix(pixels[coordinate, 0], dark[:3], 0.72)
        pixels[coordinate, 15] = mix(pixels[coordinate, 15], dark[:3], 0.72)
        pixels[0, coordinate] = mix(pixels[0, coordinate], dark[:3], 0.72)
        pixels[15, coordinate] = mix(pixels[15, coordinate], dark[:3], 0.72)
    for line in (5, 10):
        for coordinate in range(2, 14):
            pixels[line, coordinate] = mix(pixels[line, coordinate], dark[:3], 0.64)
            pixels[coordinate, line] = mix(pixels[coordinate, line], dark[:3], 0.64)
            if line + 1 < 14:
                pixels[line + 1, coordinate] = mix(pixels[line + 1, coordinate], light[:3], 0.16)
                pixels[coordinate, line + 1] = mix(pixels[coordinate, line + 1], light[:3], 0.16)
    return image


def make_workbench_side(planks, side):
    dark, mid, light = palette(side)
    image = planks.copy()
    pixels = image.load()
    for y in range(SIZE):
        for x in (0, 1, 14, 15):
            pixels[x, y] = mix(pixels[x, y], dark[:3], 0.68)
    for y in range(4):
        for x in range(2, 14):
            pixels[x, y] = mix(pixels[x, y], light[:3], 0.22 if y < 2 else 0.08)
    for x in range(3, 13):
        pixels[x, 4] = mix(pixels[x, 4], dark[:3], 0.65)
    for y in range(6, 14):
        for x in range(4, 12):
            pixels[x, y] = mix(pixels[x, y], mid[:3], 0.3)
    for x, y in ((5, 8), (6, 8), (9, 7), (9, 8), (8, 9), (7, 10)):
        pixels[x, y] = mix(pixels[x, y], dark[:3], 0.72)
    return image


def main():
    side = Image.open(BLOCK_TEXTURES / "wood.png").convert("RGBA")
    top = Image.open(BLOCK_TEXTURES / "wood_top.png").convert("RGBA")
    if side.size != (SIZE, SIZE) or top.size != (SIZE, SIZE):
        raise ValueError("wood references must remain 16x16 pixel textures")

    planks = make_planks(side, top)
    planks.save(BLOCK_TEXTURES / "planks.png")
    make_workbench_top(planks, top).save(BLOCK_TEXTURES / "crafting_table_top.png")
    make_workbench_side(planks, side).save(BLOCK_TEXTURES / "crafting_table_side.png")


if __name__ == "__main__":
    main()
