use std::fs;

mod config;

use qedchat::*;
use crossbeam::scope;

//mod parser;
//use parser::test;


mod abstractBot;
use abstractBot::Bot;

mod caro;
use caro::SimpleBot;


fn main() {
    // TODO: command line parsing
    // e.g. "-c config.toml"

    // reading and parsing our config file
    let contents = fs::read_to_string("fbot.toml").expect("fehler");
    let _botconf: config::BotConfig = toml::from_str(&contents).unwrap();


    let client = Client::new(
        _botconf.account.user.clone(),
        _botconf.account.pass.clone()
    ).unwrap();


    // Liste aller Bots (müssen geboxed werden, da wir dynamic dispatch verwenden)
    let bots: Vec<Box<dyn Bot>> = vec![Box::new(SimpleBot())];

    scope(|s| {
        let bots = &bots;
        let client = &client;
        for c in _botconf.bots.channel.clone() {
            s.spawn(move |_| {
                let mut channel = client.listen_to_channel(c, -2).unwrap();
                loop {
                    let recv_post = channel.receive().unwrap();
                    for bot in bots {
                        if let Some(send_post) = bot.process(&recv_post) {
                            channel.send(&send_post).unwrap();
                        }
                    }
                }
            });
        }
    }).unwrap();

    //for channel_name in _botconf.bots.channel {
    //
    //    println!("starting fbot for channel \"{}\"", channel_name);
    //
    //    let mut channel = client.listen_to_channel(channel_name, -10).unwrap();
    //    let channel_send = channel_send.clone();
    //
    //    std::thread::spawn(move || loop {
    //        let post = channel.receive().unwrap();
    //        channel_send.send(post).unwrap();
    //    });
    //    loop {
    //        let post = channel_recv.recv().unwrap();
    //        println!("received {:?}", post);
    //        if post.post.bottag == BotTag::Human {
    //            let _a = parse_post(post);
    //            if _a.1 == true {
    //                //channel_send.send(_a.0);
    //            }
    //        }
    //    }
    //}
}
