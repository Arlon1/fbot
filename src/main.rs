use anyhow::Context;
use anyhow::Result;
use bots::Bot;
use futures::prelude::*;
use log::{debug, info, warn};
use qedchat::*;
use std::collections::HashSet;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};
use structopt::StructOpt;
use tokio::task::block_in_place;

mod bots;
mod config;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
  #[structopt(short, long, default_value = "fbot.dhall")]
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

  let conf: config::Config = serde_dhall::from_file(opt.config_file).parse()?;

  let bots_available: HashMap<&'static str, _> = vec![
    (
      "better_link_bot",
      Box::new(bots::better_link_bot()) as Box<dyn Bot + Send + Sync>,
    ),
    ("rita_bot", Box::new(bots::rita_bot())),
  ]
  .into_iter()
  .collect();

  let mut bots = vec![];
  let mut channels = HashSet::new();
  for (name, botconf) in &conf.bots {
    let bot = bots_available.get(name.as_str()).context("unknown bot")?;
    bots.push(bots::filter_channels(bot, botconf.channels.iter()));
    channels.extend(botconf.channels.iter());
  }

  if opt.interactive {
    println!("starting interactive mode");
    run_bots_interactive(&bots)
  } else {
    let client = Client::new(&conf.account.user, &conf.account.pass).await?;
    run_bots(&client, &bots, &channels.into_iter().collect::<Vec<_>>()).await
  }
}

async fn run_bots(
  client: &Client,
  bots: &[impl Bot + Send],
  channels: &[impl AsRef<str>],
) -> Result<()> {
  let (mut sinks, mut recv_posts): (HashMap<_, _>, stream::SelectAll<_>) = stream::iter(channels)
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

  let mut name: String = "name".into();
  let mut channel: String = "".into();

  let stdin = io::stdin();
  for line in stdin.lock().lines() {
    let line = line?;
    if line == "stop" {
      return Ok(());
    }
    if let Some(c) = line.strip_prefix("channel := ") {
      channel = c.into();
    } else if let Some(n) = line.strip_prefix("name := ") {
      name = n.into();
    } else {
      let recv_post = RecvPost {
        post: Post {
          name: name.clone(),
          message: line,
          channel: channel.clone(),
          bottag: BotTag::Human,
          delay: 0,
        },
        id: 0,
        date: QED_TIMEZONE.timestamp(0, 0),
        username: None,
        user_id: None,
        color: Default::default(),
      };
      for bot in bots {
        if let Some(send_post) = bot.process(&recv_post)? {
          println!("[{}] {}", send_post.post.name, send_post.post.message)
        }
      }
    }
  }
  Ok(())
}
