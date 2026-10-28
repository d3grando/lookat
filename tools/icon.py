"""Draws icon.png, the registry icon: a block under a crosshair in a vanilla tooltip's frame, as 32 x 32 pixel art
scaled 8 times. Standard library only.

    python tools/icon.py icon.png
"""
import struct
import sys
import zlib

SIZE, SCALE = 32, 8

# The tooltip's colours, as the mod draws it.
BACKGROUND = (0x10, 0x00, 0x10, 0xF0)
FRAME_TOP = (0x50, 0x00, 0xFF, 0xFF)
FRAME_BOTTOM = (0x28, 0x00, 0x7F, 0xFF)
# The block: a grassy top over soil, lit from the top left.
TOP, TOP_EDGE = (0x6A, 0xC0, 0x4C, 0xFF), (0x8C, 0xD8, 0x64, 0xFF)
LEFT, LEFT_GRASS = (0x8A, 0x5C, 0x3A, 0xFF), (0x58, 0xA0, 0x3E, 0xFF)
RIGHT, RIGHT_GRASS = (0x66, 0x42, 0x28, 0xFF), (0x44, 0x80, 0x30, 0xFF)
CROSSHAIR, OUTLINE = (0xFF, 0xFF, 0xFF, 0xFF), (0x18, 0x10, 0x18, 0xFF)


def inside(polygon, x, y):
    """Whether (x, y) is inside the convex `polygon`, its corners in order round it."""
    sign = 0
    for (ax, ay), (bx, by) in zip(polygon, polygon[1:] + polygon[:1]):
        cross = (bx - ax) * (y - ay) - (by - ay) * (x - ax)
        if cross != 0:
            if sign and (cross > 0) != (sign > 0):
                return False
            sign = cross
    return True


def lerp(a, b, t):
    return tuple(round(ca + (cb - ca) * t) for ca, cb in zip(a, b))


def draw():
    px = [[(0, 0, 0, 0)] * SIZE for _ in range(SIZE)]
    # The tooltip: rounded by its missing corner pixels, the frame fading top to bottom.
    for y in range(SIZE):
        for x in range(SIZE):
            if (x in (0, SIZE - 1)) and (y in (0, SIZE - 1)):
                continue
            edge = x in (0, SIZE - 1) or y in (0, SIZE - 1)
            frame = x in (1, SIZE - 2) or y in (1, SIZE - 2)
            if edge:
                px[y][x] = BACKGROUND
            elif frame:
                px[y][x] = lerp(FRAME_TOP, FRAME_BOTTOM, (y - 1) / (SIZE - 3))
            else:
                px[y][x] = BACKGROUND
    # The block, isometric, its faces as pixel-centre tests.
    top = [(16, 6), (26, 11), (16, 16), (6, 11)]
    left = [(6, 11), (16, 16), (16, 27), (6, 22)]
    right = [(16, 16), (26, 11), (26, 22), (16, 27)]
    for y in range(SIZE):
        for x in range(SIZE):
            cx, cy = x + 0.5, y + 0.5
            if inside(top, cx, cy):
                near_edge = not inside([(16, 7.2), (24.6, 11), (16, 14.8), (7.4, 11)], cx, cy)
                px[y][x] = TOP_EDGE if near_edge and cy < 11 else TOP
            elif inside(left, cx, cy):
                grass = cy < 11 + (cx - 6) / 2 + 2.5
                px[y][x] = LEFT_GRASS if grass else LEFT
            elif inside(right, cx, cy):
                grass = cy < 16 - (cx - 16) / 2 + 2.5
                px[y][x] = RIGHT_GRASS if grass else RIGHT
    # The crosshair over the block's near corner, outlined so it reads on any face.
    arms = [(16, y) for y in range(12, 21) if y != 16] + [(x, 16) for x in range(12, 21) if x != 16]
    cross = set(arms) | {(16, 16)}
    for x, y in cross:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (x + dx, y + dy) not in cross:
                px[y + dy][x + dx] = OUTLINE
    for x, y in cross:
        px[y][x] = CROSSHAIR
    return px


def png(pixels, scale):
    rows = b"".join(
        b"\x00" + b"".join(bytes(p) * scale for p in row)
        for row in pixels
        for _ in range(scale)
    )
    width, height = len(pixels[0]) * scale, len(pixels) * scale

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(rows, 9))
        + chunk(b"IEND", b"")
    )


if __name__ == "__main__":
    with open(sys.argv[1], "wb") as out:
        out.write(png(draw(), SCALE))
