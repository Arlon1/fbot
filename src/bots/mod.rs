mod util;

use anyhow::Result;
use itertools::Itertools;
use qedchat::{BotTag, Post, RecvPost, SendPost};
use std::{collections::HashSet, ops::Deref};
use structopt::{clap::AppSettings, StructOpt};

pub mod rubenbot;

const CMD_PREFIX: &str = "!";

pub trait Bot {
  fn process(&self, post: &RecvPost) -> Result<Option<SendPost>>;
}

impl<A: Deref<Target = impl Bot + ?Sized>> Bot for A {
  fn process(&self, post: &RecvPost) -> Result<Option<SendPost>> {
    self.deref().process(post)
  }
}

fn plain_bot(f: impl Fn(&RecvPost) -> Result<Option<SendPost>>) -> impl Bot {
  struct PlainBot<A>(A);
  impl<A: Fn(&RecvPost) -> Result<Option<SendPost>>> Bot for PlainBot<A> {
    fn process(&self, post: &RecvPost) -> Result<Option<SendPost>> {
      (self.0)(post)
    }
  }
  PlainBot(f)
}

fn filter_input(b: impl Bot, pred: impl Fn(&RecvPost) -> bool) -> impl Bot {
  plain_bot(move |p| if pred(p) { b.process(p) } else { Ok(None) })
}

pub fn filter_channels<S: Into<String>>(b: impl Bot, cs: impl IntoIterator<Item = S>) -> impl Bot {
  let cs: HashSet<_> = cs.into_iter().map(Into::into).collect();
  filter_input(b, move |p| cs.contains(&p.post.channel))
}

fn filter_human_posts(b: impl Bot) -> impl Bot {
  filter_input(b, |p| p.post.bottag == BotTag::Human)
}

pub fn simple_bot(f: impl Fn(&RecvPost) -> Result<Option<(String, String)>>) -> impl Bot {
  let bot = plain_bot(move |post: &RecvPost| {
    Ok(f(post)?.map(|(name, message)| SendPost {
      post: Post {
        name,
        message,
        channel: post.post.channel.clone(),
        bottag: BotTag::Bot,
        delay: post.id + 1,
      },
      publicid: true,
    }))
  });
  filter_human_posts(bot)
}

pub fn structopt_bot<S: StructOpt>(
  cmd_name: &str,
  nick_name: &str,
  f: impl Fn(S, &RecvPost) -> Result<Option<String>>,
) -> impl Bot {
  let cmd_name = format!("{}{}", CMD_PREFIX, cmd_name);
  let nick_name = nick_name.to_owned();
  simple_bot(move |post| match util::tokenize_args(&post.post.message) {
    Some(args) if args.first() == Some(&cmd_name) => {
      // clap::App is not Send, therefore we can't cache it :(
      let app = S::clap()
        .global_settings(&[
          AppSettings::ColorNever,
          AppSettings::DisableVersion,
          AppSettings::NoBinaryName,
          AppSettings::DisableHelpSubcommand,
        ])
        .name(&cmd_name)
        .bin_name(&cmd_name)
        .version("")
        .long_version("");
      let msg = match app.get_matches_from_safe(args.into_iter().skip(1)) {
        Ok(matches) => f(S::from_clap(&matches), &post)?,
        Err(e) => {
          let s = e.to_string();
          let e = if e.kind == structopt::clap::ErrorKind::HelpDisplayed {
            let index = s.find('\n').unwrap_or(0);
            s.split_at(index + 1).1
          } else {
            s.as_str()
          };

          Some(
            e.replace("\n\n", "\n")
              .replace("\nFor more information try --help\n", ""),
          )
        }
      };
      Ok(msg.map(|msg| (nick_name.clone(), msg)))
    }
    _ => Ok(None),
  })
}

pub fn ritabot() -> impl Bot {
  #[derive(StructOpt)]
  enum Opt {
    /// Sag was
    Sag {
      /// Soll ich schreien?
      #[structopt(short, long)]
      laut: bool,
      text: Vec<String>,
    },
    /// Rechne krass rum
    Addiere { a1: usize, a2: usize },
  }
  use Opt::*;
  structopt_bot("rita", "        Dr. Ritarost", |opt: Opt, post| {
    Ok(Some(match opt {
      Sag { laut, text } => format!(
        "{}, {}{}",
        text.into_iter().join(" "),
        post.post.name,
        if laut { "!!!!!!" } else { "" }
      ),
      Addiere { a1, a2 } => a1.saturating_add(a2).to_string(),
    }))
  })
}
