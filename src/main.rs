use std::fs;

mod config;

use config::Config;

fn main() {
    let content = fs::read("./config.toml").expect("Failed to read file");
    let cfg: Config = toml::from_slice(&content).expect("Failed to parse config");
    println!("{:#?}", cfg);
}
