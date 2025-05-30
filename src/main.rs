use std::fs::read_to_string;
use std::time::Duration;
use std::env;

mod adalight;
mod ambient;
mod gamma;
mod modes;

use adalight::Adalight;
use ambient::{Algorithm, Ambient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config<'a> {
    port: &'a str,
    baud_rate: u32,
    leds: Vec<(u16, u16, u16, u16)>, // vec of capture areas for each LED
}

fn main() {

    let mut path = env::home_dir().unwrap();
    path.push(".config/glowworm/config.json");


    let data = read_to_string(path)
        .expect("failed to read config file");
    let conf: Config = serde_json::from_str(&data).expect("failed to parse config file");

    let mut mode = Ambient::new(Algorithm::Samples, conf.leds.clone());
    let mut ada = Adalight::new(
        conf.port,
        conf.baud_rate,
        conf.leds.len().try_into().unwrap(),
        Duration::from_millis(1000),
        &mut mode,
    );
    ada.start();
}
