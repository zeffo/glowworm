use std::time::Duration;

use serialport::SerialPort;

use crate::{gamma::GammaMap, modes::Mode};

/// A packet of data to send to the serial port.
struct Packet(Vec<u8>);

impl Packet {

    fn slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

pub struct Adalight<'a> {
    port: Box<dyn SerialPort>,
    header: [u8; 6],
    gamma: GammaMap,
    mode: &'a mut dyn Mode
}


impl<'a> Adalight<'a> {
    pub fn new(
        port: &str,
        baud_rate: u32,
        leds: u16,
        timeout: Duration,
        mode: &'a mut dyn Mode,
    ) -> Self {
        let header = Self::get_header(leds);
        let mut port = serialport::new(port, baud_rate)
            .timeout(timeout)
            .open()
            .unwrap();
        port.write_data_terminal_ready(true).unwrap();
        port.set_flow_control(serialport::FlowControl::Hardware)
            .unwrap();
        let gamma = GammaMap::new();
        Self {
            port,
            header,
            gamma,
            mode,
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

    fn pack(&self, payload: &mut [u8]) -> Packet {
        self.gamma_correct(payload);
        let data = [&self.header, &*payload].concat();
        Packet(data)
    }

    fn send_packet(&mut self, packet: &Packet) {
        self.port.write_all(packet.slice()).unwrap();
    }

    pub fn start(&mut self) {
        loop {
            let mut colors = self.mode.render();
            let packet = self.pack(&mut colors);
            self.send_packet(&packet);
        }
    }
}
