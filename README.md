# glowworm

## bias lighting for wayland

glowworm is a fast bias lighting program for wayland.
It acts as a wayland client and uses the wlr screencopy protocol to copy frames into linux-dmabuf buffers allocated by GBM, and renders to devices with the adalight protocol.
 
## Demo

[![Glowworm Demonstration Video](https://img.youtube.com/vi/r1JjI3HKP88/0.jpg)](https://www.youtube.com/watch?v=r1JjI3HKP88)

## Setup

The program looks for `.config/glowworm/config.json` in your home directory.
There is a sample in this repository.
The config requires:

- port: the serial port of the adalight device
- baud_rate: the baud rate of the adalight device
- leds: a list of box coordinates for each capture region (mapped to each pixel of the LED strip)

## Planned Features

1. Importing dmabufs into vulkan for processing (dithering, smoothing, etc) before rendering to the device
2. Better configuration tools for creating the led capture region list
