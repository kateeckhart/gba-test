#!/usr/bin/env python3
import png
import sys

(width, height, data, info) = png.Reader(filename = sys.argv[1]).asRGB8()

if width != 240 or height != 160:
    raise "Not gba sized."

output_data = bytearray()

for row in data:
    for i in range(0, len(row), 3):
        red = row[i] >> 3
        green = row[i + 1] >> 3
        blue = row[i + 2] >> 3

        combined_color = red | green << 5 | blue << 10

        output_data += combined_color.to_bytes(2, byteorder='little')

with open(sys.argv[2], mode="wb") as output_file:
    output_file.write(output_data)
