use anyhow::{Context, Result};
use bots::Bot;
use chrono::{Duration, DurationRound, Local};
use clap::{crate_name, Parser};
use diesel::{pg::PgConnection, prelude::*};
use futures::prelude::*;
use itertools::Itertools;
use log::{debug, info, warn};
use parking_lot::Mutex;
use qedchat::*;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  ops::Deref,
  path::PathBuf,
};
use tokio::task::block_in_place;

mod bots;
pub mod config;

mod lib;
use lib::*;

#[macro_use]
extern crate diesel;
pub mod models;
pub mod schema;

#[derive(Debug, Parser)]
//#[clap(setting(clap::AppSettings::ColoredHelp))]

struct Opt {
  #[clap(short, long, default_value = "fbot.dhall")]
  config_file: PathBuf,
  #[clap(subcommand)]
  mode: Option<BotMode>,
}
#[derive(Parser, Debug)]
enum BotMode {
  BotServer,
  #[clap(aliases = &["i", "-i"])]
  Interactive,
  UpdateUserDatabase {
    csv_file: PathBuf,
  },
  LogMode {
    log_file: Option<PathBuf>,
  },
}

#[tokio::main]
async fn main() -> Result<()> {
  tokio::spawn(run()).await?
}

async fn run() -> Result<()> {
  use std::time::Duration;
  setup_logging();

  let opt = Opt::parse();

  let conf: config::Config = serde_dhall::from_file(opt.config_file).parse()?;

  let mutex_urbandictionary = Mutex::new(InstantWaiter::new(Duration::from_secs(2)));
  let conn = Mutex::new(establish_connection(conf.db));

  let bots_available: Vec<(_, Box<dyn Bot + Send + Sync>)> = vec![
    (
      "freiepunkte",
      Box::new(bots::freiepunkte::freiepunkte(&conn)),
    ),
    ("nickname", Box::new(bots::nickname::nickname(&conn))),
    ("ping_readdb", Box::new(bots::ping::ping_readdb(&conn))),
    ("ping_sendtodb", Box::new(bots::ping::ping_sendtodb(&conn))),
    ("rubenbot", Box::new(bots::rubenbot::rubenbot(&conn))),
    (
      "ritabot",
      Box::new(bots::ritabot::ritabot(mutex_urbandictionary, &conn)),
    ),
  ];
  let bots_available = bots_available.into_iter().collect::<HashMap<_, _>>();

  let mut bots = vec![];
  let mut channels = HashSet::new();

  println!(
    "bots available: {:#?}",
    bots_available.iter().map(|(name, _)| name).collect_vec()
  );

  for (name, botconf) in &conf.bots {
    let bot = bots_available
      .get(name.as_str())
      .context(format!("unknown bot: {}", name))?;
    bots.push(bots::filter_channels(bot, botconf.channels.iter()));
    channels.extend(botconf.channels.iter());
  }

  let botmode = opt.mode.unwrap_or(BotMode::BotServer);
  match botmode {
    BotMode::Interactive => {
      println!("starting interactive mode");
      run_bots_interactive(&bots)
    }
    _ => match botmode {
      BotMode::UpdateUserDatabase { csv_file } => {
        let mut userlist = vec![];
        csv::Reader::from_path(csv_file)?
          .deserialize()
          .for_each(|line| {
            let line: models::Qedmitglied = line.expect("");
            userlist.push(line);
          });
        let c = &conn.lock();
        let cc = c.deref();
        diesel::insert_into(schema::qedmitglied::table)
          .values(userlist)
          .on_conflict_do_nothing()
          .execute(cc)?;
        Ok(())
      }
      _ => {
        let client = Client::new(&conf.account.user, &conf.account.pass).await?;
        match botmode {
          BotMode::LogMode { log_file } => {
            match log_file {
              None => {
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
                      "> {}\t{}:\t{} ",
                      recv_post.date.format("%m-%d %H:%M:%S"),
                      &recv_post.post.name.trim(),
                      &recv_post.post.message.trim().replace("\n", ">\n")
                    );
                    println!("{}\n", send_posts.join("\n"));
                  }
                }
                Ok(())
              }
              Some(_log_file) => {
                // todo
                Ok(())
              }
            }
          }
          _ => run_bots(&client, &bots, &channels.into_iter().collect::<Vec<_>>()).await,
        }
      }
    },
  }
}

fn setup_logging() {
  env_logger::Builder::from_env(
    env_logger::Env::default()
      .default_filter_or(format!("error,{}=info", crate_name!().replace('-', "_"))),
  )
  .format_timestamp(None)
  .format_module_path(false)
  .target(env_logger::Target::Stderr)
  .init();
}

async fn run_bots(
  client: &Client,
  bots: &[impl Bot + Send],
  channels: &[impl AsRef<str>],
) -> Result<()> {
  use stream_throttle::*;
  let throttle_pool = ThrottlePool::new(ThrottleRate::new(10, std::time::Duration::from_secs(10)));
  let (mut sinks, recv_posts): (HashMap<_, _>, stream::SelectAll<_>) = stream::iter(channels)
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

  tokio::pin! {let recv_posts = recv_posts.throttle(throttle_pool);}

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
  let mut name: String = "fbot_interactive".into();
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
        date: Local::now()
          .with_timezone(&QED_TIMEZONE)
          .duration_trunc(Duration::seconds(1))
          .expect("failed to round timestamp"),
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
    if let Some(send_post) = block_in_place(|| bot.process(&recv_post)).expect("") {
      posts.push(format!(
        "[{}] {}",
        send_post.post.name, send_post.post.message
      ));
    }
  }
  posts
}

fn establish_connection(db: config::Db) -> PgConnection {
  let postgres_url = db.database_url();

  PgConnection::establish(&postgres_url).expect(&format!("Error connecting to {}", postgres_url))
}
