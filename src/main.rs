use colorgrad::Color;
use std::io::Write;
use std::time::Duration;
mod gamma;
mod modes;

use gamma::GammaLookup;
use modes::{GlowMode, DynamicGradient};

struct Adalight {
    port: String,
    baud_rate: u32,
    leds: u16,
    mode: Box<dyn GlowMode>,
}

impl Adalight {
    pub fn get_header(&self) -> [u8; 6] {
        let num_leds = &self.leds - 1;
        let hi = ((num_leds & 0xFF00) >> 8) as u8;
        let lo = (num_leds & 0xFF) as u8;
        let checksum = hi ^ lo ^ 0x55;
        [b'A', b'd', b'a', hi, lo, checksum]
    }

    pub fn get_colors(&mut self) -> Vec<u8> {
        let mut colors = Vec::new();
        let lookup = GammaLookup::new();
        for rgb in self.mode.get_colors().chunks(3) {
            colors.push(lookup.red(rgb[0]));
            colors.push(lookup.green(rgb[1]));
            colors.push(lookup.blue(rgb[2]));
        }
        colors
    }

    pub fn get_packet(&mut self) -> Vec<u8> {
        let header = &self.get_header();
        let packet_size: usize = header.len() + (self.leds * 3) as usize;
        let mut packet = Vec::new();
        packet.reserve_exact(packet_size);
        packet.extend(header);
        packet.extend(&self.get_colors());
        packet
    }

    pub fn start(&mut self) -> Result<(), serialport::Error> {
        let mut port = serialport::new(&self.port, self.baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;
        port.write_data_terminal_ready(true)?;
        port.set_flow_control(serialport::FlowControl::Hardware)?;
        loop {
            let packet = self.get_packet();
            port.write_all(&packet)?;
        }
    }
}

fn main() -> Result<(), serialport::Error> {
    let start = Color::from_rgba8(184, 64, 151, 255);
    let end = Color::from_rgba8(151, 0, 189, 255);
    let leds = 120;
    let mut ada = Adalight {
        port: String::from("/dev/ttyACM0"),
        baud_rate: 115200,
        leds,
        mode: Box::new(DynamicGradient::from_colors(vec![start, end], leds)),
    };
    ada.start()
}
