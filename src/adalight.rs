use serialport::SerialPort;

use crate::{modes::GlowMode, GammaLookup};
use std::time::Duration;

pub struct Packet {
    _data: Vec<u8>,
}

#[allow(dead_code)]
impl Packet {
    fn slice(&self) -> &[u8] {
        &self._data
    }
    fn header(&self) -> &[u8] {
        &self._data[..6]
    }
    fn payload(&self) -> &[u8] {
        &self._data[6..]
    }
}

#[allow(dead_code)]
pub struct Adalight {
    port: Box<dyn SerialPort>,
    baud_rate: u32,
    leds: u16,
    header: [u8; 6],
    gamma: GammaLookup,
    max_brightness: u8,
}

impl Adalight {
    pub fn new(
        port: &str,
        baud_rate: u32,
        leds: u16,
        timeout: Duration,
        max_brightness: u8,
    ) -> Self {
        let header = Self::get_header(leds);
        let mut port = serialport::new(port, baud_rate)
            .timeout(timeout)
            .open()
            .unwrap();
        port.write_data_terminal_ready(true).unwrap();
        port.set_flow_control(serialport::FlowControl::Hardware)
            .unwrap();
        let gamma = GammaLookup::new();
        Self {
            port,
            baud_rate,
            leds,
            header,
            gamma,
            max_brightness,
        }
    }

    pub fn get_header(leds: u16) -> [u8; 6] {
        let num_leds = leds - 1;
        let hi = ((num_leds & 0xFF00) >> 8) as u8;
        let lo = (num_leds & 0xFF) as u8;
        let checksum = hi ^ lo ^ 0x55;
        [b'A', b'd', b'a', hi, lo, checksum]
    }

    fn gamma_correct(&self, payload: &mut [u8]) {
        for i in (0..payload.len()).step_by(3) {
            self.gamma.correct_rgb(&mut payload[i..i + 3]);
        }
    }

    pub fn pack(&self, payload: &mut [u8]) -> Packet {
        self.gamma_correct(payload);
        let _data = [&self.header, &*payload].concat();
        Packet { _data }
    }

    pub fn send_packet(&mut self, packet: &Packet) {
        self.port.write_all(packet.slice()).unwrap();
    }
}
