"""Rebuild the existing power-button artwork as antialiased Windows icons.

Build-time asset tool using only Python's standard library; never run by the app.
"""
from pathlib import Path
import math
import struct
import zlib

ROOT = Path(__file__).resolve().parent.parent
SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
COLORS = {
    "app": (0, 210, 166),
    "balanced": (98, 167, 255),
    "performance": (255, 115, 122),
    "unknown": (160, 166, 176),
}


def pixels(size, foreground):
    result = bytearray()
    for y in range(size):
        for x in range(size):
            rgba = [0, 0, 0, 0]
            for sy in range(4):
                for sx in range(4):
                    dx = (x + (sx + 0.5) / 4) * 32 / size - 16
                    dy = (y + (sy + 0.5) / 4) * 32 / size - 16
                    distance = math.hypot(dx, dy)
                    ring = 8 <= distance <= 11 and not (dy < -4 and abs(dx) < 5)
                    stem = abs(dx) < 1.8 and -12 <= dy <= 0
                    if distance <= 15:
                        color = foreground if ring or stem else (18, 25, 33)
                        for i in range(3):
                            rgba[i] += color[i]
                        rgba[3] += 255
            covered = rgba[3] / 255
            result.extend(round(c / covered) if covered else 0 for c in rgba[:3])
            result.append(round(rgba[3] / 16))
    return bytes(result)


def png(size, rgba):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    rows = b"".join(b"\0" + rgba[y * size * 4:(y + 1) * size * 4] for y in range(size))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


def ico(foreground):
    images = [png(size, pixels(size, foreground)) for size in SIZES]
    offset = 6 + 16 * len(SIZES)
    directory = bytearray(struct.pack("<HHH", 0, 1, len(SIZES)))
    for size, data in zip(SIZES, images):
        directory.extend(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    return bytes(directory) + b"".join(images)


if __name__ == "__main__":
    destination = ROOT / "assets" / "icons"
    destination.mkdir(parents=True, exist_ok=True)
    for name, color in COLORS.items():
        path = destination / f"{name}.ico"
        path.write_bytes(ico(color))
        print(f"{path.name}: {path.stat().st_size} bytes")
