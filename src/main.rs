use anyhow::Result;
use bots::Bot;
use futures::prelude::*;
use log::{debug, info, warn};
use qedchat::*;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};
use structopt::StructOpt;
use tokio::{fs, task::block_in_place};

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

#[tokio::main]
async fn main() -> Result<()> {
  tokio::spawn(run()).await?
}

async fn run() -> Result<()> {
  env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

  let opt = Opt::from_args();

  let bots: Vec<Box<dyn Bot + Send + Sync>> = vec![Box::new(bots::example_bot())];

  if opt.interactive {
    println!("starting interactive mode");
    run_bots_interactive(&bots)
  } else {
    let contents = fs::read_to_string(opt.config_file).await?;
    let botconf: config::BotConfig = toml::from_str(&contents)?;
    let client = Client::new(&botconf.account.user, &botconf.account.pass).await?;
    run_bots(&client, &bots, &botconf.bots.channel).await
  }
}

async fn run_bots(
  client: &Client,
  bots: &[impl Bot + Send],
  channels: &[impl AsRef<str>],
) -> Result<()> {
  let (mut sinks, recv_posts): (HashMap<_, _>, Vec<_>) = stream::iter(channels)
    .map::<Result<_>, _>(Ok)
    .and_then(|c| async move {
      let c = c.as_ref();
      let (send, recv) = client.listen_to_channel(c, 0).await?;
      info!("listening to channel '{}'", c);
      Ok(((c, Box::pin(send)), Box::pin(recv)))
    })
    .try_collect::<Vec<_>>()
    .await?
    .into_iter()
    .unzip();
  // stream::SelectAll does not implement Extend :(
  let mut recv_posts: stream::SelectAll<_> = recv_posts.into_iter().collect();
  while let Some(recv_post) = recv_posts.try_next().await? {
    debug!("received: {:?}", recv_post);
    for bot in bots {
      if let Some(send_post) = block_in_place(|| bot.process(&recv_post))? {
        let c = send_post.post.channel.as_str();
        if let Some(sink) = sinks.get_mut(c) {
          sink.send(Cow::Owned(send_post)).await?;
        } else {
          warn!("invalid channel for post {:?}", recv_post);
        }
      }
    }
  }
  Ok(())
}

fn run_bots_interactive(bots: &[impl Bot]) -> Result<()> {
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
        color: Default::default(),
      };
      if let Some(send_post) = bot.process(&recv_post)? {
        println!("[{}] {}", send_post.post.name, send_post.post.message)
      }
    }
  }
  Ok(())
}
