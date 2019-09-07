use std::fs;

extern crate serde;
//use serde::{Serialize, Deserialize};

mod config;

fn main() {
    // TODO: command line parsing
    // e.g. "-c config.toml"

    // reading the config file into a string
    let contents = fs::read_to_string("fbot_r.toml").expect("fehler");

    let _botconf: config::BotConfig = toml::from_str(&contents).unwrap();
    for element in _botconf.bots.channel {
        println!("channel ist {}", element);
        // TODO spawn a client for every channel
    }
}
