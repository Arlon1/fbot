mod util;

use anyhow::Result;
use qedchat::{BotTag, Post, RecvPost, SendPost};
use std::{collections::HashSet, ops::Deref};
use structopt::{clap::AppSettings, StructOpt};

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

pub fn filter_channels<S: Into<String>>(b: impl Bot, cs: impl Iterator<Item = S>) -> impl Bot {
  let cs: HashSet<_> = cs.map(Into::into).collect();
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
      publicid: false,
    }))
  });
  filter_human_posts(bot)
}

pub fn structopt_bot<S: StructOpt>(
  name: &str,
  f: impl Fn(S) -> Result<Option<String>>,
) -> impl Bot {
  let name = name.to_string();
  simple_bot(move |post| match util::tokenize_args(&post.post.message) {
    Some(args) if args.first() == Some(&name) => {
      // clap::App is not Send, therefore we can't cache it :(
      let app = S::clap()
        .global_settings(&[
          AppSettings::ColorNever,
          AppSettings::DisableVersion,
          AppSettings::NoBinaryName,
          AppSettings::DisableHelpSubcommand,
        ])
        .name(&name)
        .bin_name(&name)
        .version("")
        .long_version("");
      let msg = match app.get_matches_from_safe(args.into_iter().skip(1)) {
        Ok(matches) => f(S::from_clap(&matches))?,
        Err(e) => Some(e.to_string()),
      };
      Ok(msg.map(|msg| (name.clone(), msg)))
    }
    _ => Ok(None),
  })
}

pub fn example_bot() -> impl Bot {
  #[derive(StructOpt)]
  enum Opt {
    /// Sag was
    Sag {
      /// Soll ich schreien?
      #[structopt(short, long)]
      laut: bool,
      text: String,
    },
    /// Rechne krass rum
    Addiere { a1: usize, a2: usize },
  }
  use Opt::*;
  structopt_bot("!example", |opt: Opt| {
    Ok(Some(match opt {
      Sag { laut, text } => format!("{}{}", text, if laut { "!!!!!!" } else { "" }),
      Addiere { a1, a2 } => a1.saturating_add(a2).to_string(),
    }))
  })
}
