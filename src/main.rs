use anyhow::Result;
use bots::Bot;
use crossbeam::channel;
use log::{debug, error, info};
use qedchat::*;
use std::{fs, path::PathBuf};
use structopt::StructOpt;

mod bots;
mod config;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    #[structopt(short, long, default_value = "fbot.toml")]
    config_file: PathBuf,
    #[structopt(short, long)]
    interactive: bool,
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();

    let bots: Vec<Box<dyn Bot>> = vec![Box::new(bots::ping_bot())];

    if opt.interactive {
        println!("starting interactive mode");
        run_bots_interactive(&bots)
    } else {
        let contents = fs::read_to_string(opt.config_file)?;
        let botconf: config::BotConfig = toml::from_str(&contents)?;
        let client = Client::new(&botconf.account.user, &botconf.account.pass)?;
        run_bots(
            &client,
            &bots,
            &botconf
                .bots
                .channel
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

fn run_bots(client: &Client, bots: &[Box<dyn Bot>], channels: &[&str]) -> Result<()> {
    let (error_send, error_recv) = channel::bounded(1);
    crossbeam::scope(|s| {
        for c in channels {
            let error_send = error_send.clone();
            s.spawn(move |_| {
                if let Err(e) = || -> Result<()> {
                    let mut channel = client.listen_to_channel(c, 0)?;
                    info!("listening to channel '{}'", c);
                    loop {
                        let recv_post = channel.receive()?;
                        debug!("received: {:?}", recv_post);
                        for bot in bots {
                            if let Some(send_post) = bot.process(&recv_post)? {
                                channel.send(&send_post)?;
                            }
                        }
                    }
                }() {
                    error_send.send(e).unwrap();
                }
            });
        }

        let error = error_recv.recv().unwrap();
        error!("error caught: {}", error);
        info!("now exiting");
        std::process::exit(1)
    })
    .unwrap()
}

fn run_bots_interactive(bots: &[Box<dyn Bot>]) -> Result<()> {
    use chrono::TimeZone;
    use qedchat::*;
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line == "stop" {
            return Ok(());
        }
        for bot in bots {
            let recv_post = RecvPost {
                post: Post {
                    name: "".to_string(),
                    message: line.clone(),
                    channel: "".to_string(),
                    bottag: BotTag::Human,
                    delay: 0,
                },
                id: 0,
                date: QED_TIMEZONE.timestamp(0, 0),
                username: None,
                user_id: None,
                color: 0,
            };
            if let Some(send_post) = bot.process(&recv_post)? {
                println!("[{}] {}", send_post.post.name, send_post.post.message)
            }
        }
    }
    Ok(())
}
