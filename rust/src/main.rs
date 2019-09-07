use std::fs;

extern crate serde;
mod config;

extern crate qedchat;
use qedchat::{BotTag, Client, Post, SendPost};


fn botloop(channel: String) {
    let client = Client::new("FranzBots", "").unwrap();
    let mut channel = client.listen_to_channel(channel, -10).unwrap();

    loop {
        let post = channel.receive().unwrap();
        println!("[{}] [{}] {}", post.date, post.post.name, post.post.message);
        if post.post.bottag == BotTag::Human && post.post.message.starts_with("!ping") {
            channel
                .send(&SendPost {
                    post: Post {
                        name: "fbot".to_owned(),
                        message: "pong".to_owned(),
                        channel: post.post.channel,
                        bottag: BotTag::Bot,
                        delay: post.id + 1,
                    },
                    publicid: false,
                })
                .unwrap();
        }
    }
}


fn main() {
    // TODO: command line parsing
    // e.g. "-c config.toml"

    // reading the config file into a string
    let contents = fs::read_to_string("fbot_r.toml").expect("fehler");

    let _botconf: config::BotConfig = toml::from_str(&contents).unwrap();
    for channel in _botconf.bots.channel {
        println!("channel ist {}", channel);

        /* async */ botloop(channel);
    }
}
