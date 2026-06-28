#!/usr/bin/env python3
"""Generate toro-128.png / toro-32.png (RGBA) via stdlib only.

Renders the "toro" brand mark (front-view bull-head silhouette, the same path
used by `ToroIcon.tsx` and `favicon.svg`) onto an indigo rounded-square tile.

The SVG path `M3 3.5C...Z` is pre-resolved into absolute cubic Bezier segments
(relative `c` and smooth `s` commands expanded by hand), flattened via De
Casteljau subdivision, then scanline-filled with the even-odd rule at 4x
supersampling for anti-aliasing.
"""

import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent

# --- bull-head path, pre-resolved into absolute cubics in 24x24 space ---------
# Each tuple: (c1x, c1y, c2x, c2y, ex, ey). Start point is (3.0, 3.5).
START = (3.0, 3.5)
SEGMENTS = [
    (4.0, 6.0, 6.5, 7.2, 9.0, 7.0),
    (10.0, 6.5, 11.0, 6.0, 12.0, 6.0),
    (13.0, 6.0, 14.0, 6.5, 15.0, 7.0),  # smooth (reflected c1)
    (17.5, 7.2, 20.0, 6.0, 21.0, 3.5),
    (20.5, 8.0, 19.0, 10.0, 17.0, 11.0),
    (16.5, 14.0, 16.0, 17.0, 15.0, 19.0),
    (14.0, 20.5, 13.0, 21.0, 12.0, 21.0),
    (11.0, 21.0, 10.0, 20.5, 9.0, 19.0),  # smooth (reflected c1)
    (8.0, 17.0, 7.5, 14.0, 7.0, 11.0),
    (5.0, 10.0, 3.5, 8.0, 3.0, 3.5),
]

TILE_RGB = (99, 102, 241)  # #6366f1 indigo
BULL_RGB = (255, 255, 255)  # white
CORNER_RADIUS = 7.0  # in 32-unit space


def _chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, w: int, h: int, rgba: bytes) -> None:
    raw = bytearray()
    stride = w * 4
    for y in range(h):
        raw.append(0)  # filter type 0
        raw += rgba[y * stride : (y + 1) * stride]
    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)  # 8-bit RGBA
    idat = zlib.compress(bytes(raw), 9)
    path.write_bytes(
        sig + _chunk(b"IHDR", ihdr) + _chunk(b"IDAT", idat) + _chunk(b"IEND", b"")
    )


def flatten_cubic(p0, c1, c2, p1, steps=24):
    """De Casteljau subdivision of a cubic into `steps` line points (excl. p0)."""
    x0, y0 = p0
    x1, y1 = c1
    x2, y2 = c2
    x3, y3 = p1
    pts = []
    for i in range(1, steps + 1):
        t = i / steps
        mt = 1 - t
        a = mt * mt * mt
        b = 3 * mt * mt * t
        c = 3 * mt * t * t
        d = t * t * t
        x = a * x0 + b * x1 + c * x2 + d * x3
        y = a * y0 + b * y1 + c * y2 + d * y3
        pts.append((x, y))
    return pts


def build_polygon():
    """Bull-head outline in 24x24 space (closed)."""
    pts = [START]
    cur = START
    for c1x, c1y, c2x, c2y, ex, ey in SEGMENTS:
        pts.extend(flatten_cubic(cur, (c1x, c1y), (c2x, c2y), (ex, ey)))
        cur = (ex, ey)
    return pts


def render(size: int, poly24):
    """Render the toro tile at `size`x`size`, returning RGBA bytes."""
    ss = 4  # supersample factor
    iw = size * ss
    # Transform 24-space -> internal pixel space.
    # Matches favicon: 32-canvas, translate(4,4) in 32-space => scale = size/32,
    # offset = (size/32)*4. Bull is 24-space * (size/32).
    scale = size / 32.0
    off = 4.0 * scale
    poly = [((bx * scale + off) * ss, (by * scale + off) * ss) for bx, by in poly24]

    # Even-odd scanline fill at internal resolution -> boolean coverage mask.
    edges = []
    for ax, ay in poly:
        pass
    for i in range(len(poly)):
        ax, ay = poly[i]
        bx, by = poly[(i + 1) % len(poly)]
        if ay == by:
            continue
        edges.append((ax, ay, bx, by))

    cover = bytearray(iw * iw)  # 0/255 coverage
    for y in range(iw):
        yc = y + 0.5
        xs = []
        for ax, ay, bx, by in edges:
            if (ay <= yc < by) or (by <= yc < ay):
                t = (yc - ay) / (by - ay)
                xs.append(ax + t * (bx - ax))
        xs.sort()
        for k in range(0, len(xs) - 1, 2):
            x0 = max(0, int(xs[k]))
            x1 = min(iw, int(xs[k + 1] + 1))
            for x in range(x0, x1):
                cover[y * iw + x] = 255

    # Rounded-rect tile mask at output resolution.
    r = CORNER_RADIUS * scale  # in output px

    def in_tile(x, y):
        # pixel-center coords
        cx = x + 0.5
        cy = y + 0.5
        # nearest corner center if within a corner quadrant
        nx = min(cx, size - cx)
        ny = min(cy, size - cy)
        if nx < r and ny < r:
            dx = r - nx
            dy = r - ny
            return dx * dx + dy * dy <= r * r
        return True

    # Downsample coverage -> alpha, composite over tile.
    out = bytearray(size * size * 4)
    for oy in range(size):
        for ox in range(size):
            if not in_tile(ox, oy):
                continue
            # average coverage over ss*ss block
            acc = 0
            for dy in range(ss):
                for dx in range(ss):
                    acc += cover[(oy * ss + dy) * iw + (ox * ss + dx)]
            cov = acc // (ss * ss)  # 0..255
            i = (oy * size + ox) * 4
            if cov == 0:
                out[i : i + 3] = bytes(TILE_RGB)
            elif cov == 255:
                out[i : i + 3] = bytes(BULL_RGB)
            else:
                # alpha-blend white over indigo
                a = cov / 255.0
                out[i] = round(TILE_RGB[0] * (1 - a) + BULL_RGB[0] * a)
                out[i + 1] = round(TILE_RGB[1] * (1 - a) + BULL_RGB[1] * a)
                out[i + 2] = round(TILE_RGB[2] * (1 - a) + BULL_RGB[2] * a)
            out[i + 3] = 255
    return bytes(out)


def main():
    poly = build_polygon()
    for size in (1024, 128, 32):
        rgba = render(size, poly)
        write_png(OUT / f"toro-{size}.png", size, size, rgba)
        print(f"generated toro-{size}.png")


if __name__ == "__main__":
    main()
