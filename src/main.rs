use colorgrad::{CustomGradient, Color};
use std::time::Duration;
use std::io::Write;
use libwayshot::WayshotConnection;
use color_thief::get_palette;
use serde_json::from_str;
use std::fs::read_to_string;

struct Adalight {
    port: String,
    baud_rate: u32,
    leds: u16
}

impl Adalight {
    pub fn get_header(&self) -> [u8; 6]{
        let num_leds = &self.leds - 1; 
        let hi = ((num_leds & 0xFF00) >> 8) as u8;
        let lo = (num_leds & 0xFF) as u8;
        let checksum = hi ^ lo ^ 0x55;
        [b'A', b'd', b'a', hi, lo, checksum]
    }

    pub fn get_screen(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let wayshot_conn = WayshotConnection::new().unwrap();
        let outputs = wayshot_conn.get_all_outputs();
        wayshot_conn.screenshot_single_output(&outputs[0], true).unwrap()
    }

    pub fn get_edges(&self) -> Vec<u8> {
        let screen = self.get_screen();
        let conf = read_to_string("./config.json").unwrap();
        let json: Vec<[u32;4]> = from_str(&conf).unwrap();
        assert_eq!(json.len(), self.leds as usize);
        let mut colors = Vec::new();
        for region in json {
            let mut area = Vec::new();
            for x in region[0]..region[2] {
                for y in region[1]..region[3] {
                    let rgb = screen.get_pixel(x, y);
                    for p in rgb.0.iter().take(3) {
                        area.push(*p);
                    }
                }
            }
            let color = get_palette(&area, color_thief::ColorFormat::Rgb, 10, 2).unwrap()[0];
            for c in color.iter() {
                colors.push(c);
            }
        }
        colors
    }

    pub fn get_gradient_colors(&self) -> Vec<u8> {
        let mut leds = Vec::new();
        leds.reserve_exact(self.leds as usize);
        let start = Color::from_rgba8(191, 0, 140, 255);
        let end = Color::from_rgba8(70, 0, 97, 255);
        let gradient = CustomGradient::new().colors(&[start.clone(), end.clone(), end, start]).build().unwrap().colors(self.leds as usize);
        for raw_color in gradient {
            let color = raw_color.to_rgba8();
            for seg in color.iter().take(3) {
                leds.push(*seg);
            }
        } 
        leds
    }

    pub fn get_colors(&self) -> Vec<u8> {
        self.get_gradient_colors()
        // self.get_edges()
    }

    pub fn get_packet(&self) -> Vec<u8> {
        let header = &self.get_header();
        let packet_size: usize = header.len() + (self.leds*3) as usize;
        let mut packet = Vec::new();
        packet.reserve_exact(packet_size);
        packet.extend(header);
        packet.extend(&self.get_colors());
        packet
    }

    pub fn start(&self) -> Result<(), serialport::Error> {
        let mut port = serialport::new(&self.port, self.baud_rate).timeout(Duration::from_millis(1000)).open()?;
        port.write_data_terminal_ready(true)?;
        port.set_flow_control(serialport::FlowControl::Hardware)?;
        loop {
            let packet = self.get_packet();
            port.write_all(&packet)?;
        }
    }
}

fn main() -> Result<(), serialport::Error> {
    let ada = Adalight {port: String::from("/dev/ttyACM0"), baud_rate: 115200, leds: 120};
    ada.start()
}
