# glowworm

bias lighting for wayland _(WIP)_

## Features
- wlr-screencopy-unstable-v1 for screen capture
- adalight to drive LEDs
- static/dynamic gradient mode

## Plans
- better configuration
- faster screencopy (for some reason wlr-screencopy still feels a little slow to me although it's likely user error)

## Setup
- Modify config.json to match your LED strip setup (the led_config.py helper script is broken, please ignore)
- Modify main.rs with your mode and adalight device config
- `cargo run`





