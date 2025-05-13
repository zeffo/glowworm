use std::fs::read_to_string;
use std::time::Duration;

mod adalight;
mod ambient;
mod gamma;
mod modes;

use adalight::Adalight;
use ambient::{Algorithm, Ambient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LEDConfig {
    leds: Vec<(u16, u16, u16, u16)>, // vec of capture areas for each LED
}

fn main() {
    const LEDS: u16 = 120;

    let conf = serde_json::from_str(
        &read_to_string("/home/aman/.config/glowworm/config.json")
            .expect("failed to read config file"),
    ).expect("failed to parse config file");
    let mut mode = Ambient::new(Algorithm::Samples, conf);
    let mut ada = Adalight::new(
        "/dev/ttyACM0",
        115200,
        LEDS,
        Duration::from_millis(1000),
        &mut mode,
    );
    ada.start();
}
