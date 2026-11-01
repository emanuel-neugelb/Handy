#!/usr/bin/env python3
"""Draw the NGLB placeholder app icon (1024px). Replace with brand artwork when available.
Usage: make-icon.py OUT.png   (needs Pillow)"""
import sys
from PIL import Image, ImageDraw

S = 1024
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
d.rounded_rectangle((40, 40, S - 40, S - 40), radius=230, fill=(26, 32, 56, 255))
white = (255, 255, 255, 255)
d.rounded_rectangle((392, 190, 632, 590), radius=120, fill=white)
d.arc((290, 380, 734, 800), 0, 180, fill=white, width=44)
d.rounded_rectangle((490, 780, 534, 860), radius=22, fill=white)
d.rounded_rectangle((390, 842, 634, 886), radius=22, fill=white)
img.save(sys.argv[1])
