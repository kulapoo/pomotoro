#!/usr/bin/env python3
"""Generate tray-idle.png and tray-active.png (32x32 RGBA) via stdlib only.

Idle  -> grayscale tomato outline (dimmed)
Active-> red tomato with green stem (accent)
"""

import math
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent


def _chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, pixels: list[list[tuple[int, int, int, int]]]) -> None:
    h = len(pixels)
    w = len(pixels[0])
    raw = bytearray()
    for row in pixels:
        raw.append(0)  # filter type 0
        for r, g, b, a in row:
            raw += bytes((r, g, b, a))
    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)  # 8-bit RGBA
    idat = zlib.compress(bytes(raw), 9)
    png = sig + _chunk(b"IHDR", ihdr) + _chunk(b"IDAT", idat) + _chunk(b"IEND", b"")
    path.write_bytes(png)


def draw(filled_rgb, outline_rgb, stem_rgb):
    size = 32
    cx = cy = 15.5
    body_r = 12.0
    # transparent canvas
    px = [[(0, 0, 0, 0) for _ in range(size)] for _ in range(size)]
    for y in range(size):
        for x in range(size):
            dx = x - cx
            dy = y - cy
            d = math.sqrt(dx * dx + dy * dy)
            # body
            if d <= body_r - 1.0:
                px[y][x] = (*filled_rgb, 235)
            elif d <= body_r:
                px[y][x] = (*outline_rgb, 255)
    # stem (small green rect on top)
    sx0, sx1, sy0, sy1 = 14, 18, 2, 6
    for y in range(sy0, sy1):
        for x in range(sx0, sx1):
            if 0 <= x < size and 0 <= y < size:
                px[y][x] = (*stem_rgb, 255)
    # leaf
    for lx, ly in [(18, 3), (19, 4), (20, 3)]:
        if 0 <= lx < size and 0 <= ly < size:
            px[ly][lx] = (*stem_rgb, 255)
    return px


write_png(
    OUT / "tray-idle.png",
    draw(filled_rgb=(120, 120, 120), outline_rgb=(70, 70, 70), stem_rgb=(90, 110, 90)),
)

write_png(
    OUT / "tray-active.png",
    draw(filled_rgb=(214, 64, 54), outline_rgb=(150, 30, 25), stem_rgb=(90, 170, 80)),
)

print("generated tray-idle.png and tray-active.png")
