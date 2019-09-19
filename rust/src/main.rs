use std::fs;

extern crate serde;
mod config;

mod channel_handler;
use channel_handler::ChannelHandler;

fn main() {
    // TODO: command line parsing
    // e.g. "-c config.toml"

    // reading the config file into a string
    let contents = fs::read_to_string("fbot.toml").expect("fehler");

    let _botconf: config::BotConfig = toml::from_str(&contents).unwrap();
    for channel in _botconf.bots.channel {
        println!("starting fbot for channel \"{}\"", channel);

        let handler = ChannelHandler {channel_name: "fbot".to_string()};
        // TODO: async
        handler.botloop();
        
    }
}
