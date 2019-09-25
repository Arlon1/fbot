use std::fs;

mod config;

use crossbeam::channel;

use qedchat::*;

mod parser;
use parser::parse_post;

fn main() {
    // TODO: command line parsing
    // e.g. "-c config.toml"

    // reading and parsing our config file
    let contents = fs::read_to_string("fbot.toml").expect("fehler");
    let _botconf: config::BotConfig = toml::from_str(&contents).unwrap();


    let (channel_send, channel_recv) = channel::unbounded();

    let client = Client::new(
        _botconf.account.user,
        _botconf.account.pass
    ).unwrap();

    for channel_name in _botconf.bots.channel {

        println!("starting fbot for channel \"{}\"", channel_name);

        let mut channel = client.listen_to_channel(channel_name, -10).unwrap();
        let channel_send = channel_send.clone();

        std::thread::spawn(move || loop {
            let post = channel.receive().unwrap();
            channel_send.send(post).unwrap();
        });
        loop {
            let post = channel_recv.recv().unwrap();
            println!("received {:?}", post);
            if post.post.bottag == BotTag::Human {
                let _a = parse_post(post);
                if _a.1 == true {
                    //channel_send.send(_a.0);
                }
            }
        }
    }
}
