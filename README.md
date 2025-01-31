# glowworm

bias lighting for wayland _(WIP)_
 
## Demo

[![Glowworm Demonstration Video](https://img.youtube.com/vi/r1JjI3HKP88/0.jpg)](https://www.youtube.com/watch?v=r1JjI3HKP88)

## Features
- wlr-screencopy-unstable-v1 for screen capture
- dmabuf support
- adalight to drive LEDs
- static/dynamic gradient mode

## Plans
- better configuration

## Setup
- Modify config.json to match your LED strip setup (the led_config.py helper script is broken, please ignore)
- Modify main.rs with your mode and adalight device config
- `cargo run`

## Misc
I wrote [a small blog](https://zeffo.me/blog/glowworm) that goes over what I learnt while making this. If you're interested/making something similar, you might find it useful.
