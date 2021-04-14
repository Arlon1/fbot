use anyhow::{Context, Result};
use bots::Bot;
use futures::prelude::*;
use log::{debug, info, warn};
use qedchat::*;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  path::PathBuf,
};
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
  #[structopt(long)]
  log_mode: bool,
  //#[structopt(long, default_value = "")]
  //log_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
  tokio::spawn(run()).await?
}

async fn run() -> Result<()> {
  env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

  let opt = Opt::from_args();

  let conf: config::Config = serde_dhall::from_file(opt.config_file).parse()?;

  let bots_available: Vec<(_, Box<dyn Bot + Send + Sync>)> = vec![
    ("rubenbot", Box::new(bots::rubenbot::rubenbot())),
    ("ritabot", Box::new(bots::ritabot())),
  ];
  let bots_available = bots_available.into_iter().collect::<HashMap<_, _>>();

  let log_mode_channels = ["".to_owned()];

  let mut bots = vec![];
  let mut channels = HashSet::new();
  for (name, botconf) in &conf.bots {
    let bot = bots_available.get(name.as_str()).context("unknown bot")?;
    if opt.log_mode {
      if botconf.channels.len() > 0 {
        bots.push(bots::filter_channels(bot, log_mode_channels.iter()));
        channels.extend(log_mode_channels.iter());
      }
    } else {
      bots.push(bots::filter_channels(bot, botconf.channels.iter()));
      channels.extend(botconf.channels.iter());
    }
  }

  if opt.interactive {
    println!("starting interactive mode");
    run_bots_interactive(&bots)
  } else {
    let client = Client::new(&conf.account.user, &conf.account.pass).await?;
    if opt.log_mode {
      let log_stream = client
        .fetch_log("", &LogMode::DateRecent(chrono::Duration::weeks(4)))
        .await?;
      tokio::pin!(log_stream);
      while let Some(recv_post) = log_stream.try_next().await? {
        let mut send_posts = vec![];
        for post_str in process_post(&recv_post, &bots) {
          send_posts.push(post_str);
        }
        if !send_posts.is_empty() {
          println!(
            "> {}: {} ",
            &recv_post.post.name.trim(),
            &recv_post.post.message.trim().replace("\n", ">\n")
          );
          println!("{}\n", send_posts.join("\n"));
        }
      }
      Ok(())
    } else {
      run_bots(&client, &bots, &channels.into_iter().collect::<Vec<_>>()).await
    }
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

  let mut name: String = "name".into();
  let mut channel: String = "fbot".into();

  let mut e = rustyline::Editor::<()>::new();
  let hist_file = ".fbot.history";
  let _ = e.load_history(hist_file);
  loop {
    let line = e.readline(&format!(
      "🦀🦀🦀 channel: '{}' | name: '{}'\n🦀🦀🦀> ",
      channel, name
    ))?;
    e.add_history_entry(&line);
    e.save_history(hist_file)?;
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
      for post_str in process_post(&recv_post, bots) {
        println!("{}", post_str);
      }
    }
  }
}

fn process_post(recv_post: &RecvPost, bots: &[impl bots::Bot]) -> Vec<String> {
  let mut posts = vec![];
  for bot in bots {
    if let Some(send_post) = bot.process(&recv_post).expect("") {
      posts.push(format!(
        "[{}] {}",
        send_post.post.name, send_post.post.message
      ));
    }
  }
  posts
}
