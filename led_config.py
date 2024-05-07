from PIL import Image, ImageDraw
import json

img = Image.open("screen.png")

h = 1440
w = 2560

edges = [w, h]
segments = [20, 40, 20, 40]  # 120 leds
mods = [(1, 1), (1, 0), (0, 0), (0, 1)]
mods2 = [()]
"""
2560, 1440 -> (1, 1)
2560, 0 -> (1, 0)
0, 0 -> 0, 0
0, 1440 -> 0, 1


"""

with open("config.json") as f:
    conf = json.load(f)

i = 3
seg = segments[i]
edge = edges[i % 2]
next_edge = edges[(i + 1) % 2]
gap = next_edge / seg
xmod, ymod = mods[i % 4]
start = w * xmod, h * ymod
for y in range(seg):
    draw = ImageDraw.Draw(img)
    coords = [
        start[0] + (gap * y),
        start[1] - gap,
        start[0] + (gap * (y + 1)),
        start[1],
    ]
    conf.append([int(x) for x in coords])
    draw.rectangle(coords, fill=None, outline=(255, 255, 255))
img.show()


with open("config.json", "w") as f:
    json.dump(conf, f, indent=4)
