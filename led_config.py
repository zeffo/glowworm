import json
from itertools import cycle

from PIL import Image, ImageDraw

img = Image.open("screen.png")

h = 1440
w = 2560

SIZE_MOD = 200

edges = cycle([h, w])
segments = [20, 40, 20, 40]  # 120 leds
mods = ((1, 1), (1, 0), (0, 0), (0, 1))
"""
0, 0                2560, 0


0, 1440             2560, 1440
"""

# with open("config.json") as f:
#     conf = json.load(f)["leds"]

for i, segment in enumerate(segments):
    start = (w * mods[i][0], h * mods[i][1])  # the coordinate to start calculating from
    end = (w * mods[(i + 1) % 4][0], h * mods[(i + 1) % 4][1])
    edge = next(edges)  # the current edge size
    size = edge // segment  # the length of the pixel square edge
    coords = []

    if edge == h:
        size_mod = 0
        dmod = -1 if start[1] > end[1] else 1
        for y in range(start[1], end[1], size * dmod):
            if dmod == -1:
                coords.append(
                    [
                        (
                            start[0] + (size * dmod) + (size_mod * dmod),
                            y + (size * dmod),
                        ),
                        (start[0], y),
                    ]
                )
            else:
                coords.append(
                    [
                        (start[0], y),
                        (
                            start[0] + (size * dmod) + (size_mod * dmod),
                            y + (size * dmod),
                        ),
                    ]
                )
    else:
        size_mod = SIZE_MOD
        dmod = -1 if start[0] > end[0] else 1
        for x in range(start[0], end[0], size * dmod):
            if dmod == -1:
                bottom_right = (
                    x,
                    start[1] + (size * dmod * -1) + (size_mod * dmod * -1),
                )
                top_left = (x + (size * dmod), start[1])
            else:
                top_left = (x, start[1] + (size * dmod * -1) + (size_mod * dmod * -1))
                bottom_right = (x + (size * dmod), start[1])
            coords.append([top_left, bottom_right])
    # print(f"Segment {i} {start} - {end}: {len(coords)}, {coords}")
    #
    # assert (
    #     len(coords) == segment
    # ), f"Segment {i} {segment} does not have enough LEDs: {start}-{end}, {edge}, {coords}, {len(coords)}"
    #

    draw = ImageDraw.Draw(img)
    for tleft, bright in coords:
        draw.rectangle((tleft, bright), fill=None, outline=(255, 255, 255))
# img.show()
print(json.dumps({"leds": coords}))

# with open("config.json", "w") as f:
#     json.dump({"leds": conf}, f, indent=4)
