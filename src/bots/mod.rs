pub mod freiepunkte;
pub mod nickname;
pub mod ping;
pub mod ritabot;
pub mod rubenbot;
mod util;

use anyhow::Result;
use clap::AppSettings;
use clap::Parser;
use itertools::Itertools;
use qedchat::{BotTag, Post, RecvPost, SendPost};
use regex::Regex;
use std::{collections::HashSet, ops::Deref};

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

fn args_bot(
  nick_name: &str,
  f: impl Fn(Vec<String>, &RecvPost) -> Result<Option<String>>,
) -> impl Bot {
  let nick_name = nick_name.to_owned();
  //let tok_args = vec!["#test".to_owned(), "franz".to_owned()];

  simple_bot(
    move |post: &RecvPost| match util::tokenize_args(&post.post.message) {
      Some(args) => {
        let msg = f(args.clone(), post)?;
        Ok(msg.map(|msg| (nick_name.clone(), msg)))
      }
      _ => Ok(None),
    },
  )
}

fn clap_bot<C: Parser>(
  cmd_name: &str,
  nick_name: &str,
  f: impl Fn(C, &RecvPost) -> Result<Option<String>>,
) -> impl Bot {
  clap_bot_proto(cmd_name, nick_name, true, f)
}

fn clap_bot_notrigger<C: Parser>(
  nick_name: &str,
  f: impl Fn(C, &RecvPost) -> Result<Option<String>>,
) -> impl Bot {
  clap_bot_proto(nick_name, nick_name, false, f)
}

fn clap_bot_proto<C: Parser>(
  cmd_name: &str,
  nick_name: &str,
  filter_args: bool,
  f: impl Fn(C, &RecvPost) -> Result<Option<String>>,
) -> impl Bot {
  let cmd_name = format!("{}{}", CMD_PREFIX, cmd_name);
  let nick_name = nick_name.to_owned();
  let app = C::into_app()
    .color(clap::ColorChoice::Never)
    .global_setting(AppSettings::DisableVersionFlag)
    .global_setting(AppSettings::NoBinaryName)
    .global_setting(AppSettings::DisableHelpSubcommand)
    .name(&cmd_name)
    .bin_name(&cmd_name)
    .version("")
    .long_version("");

  simple_bot(move |post| {
    let args = util::tokenize_args(&post.post.message);
    match args {
      Some(mut args)
        if (args.first().as_ref().map(|arg| arg.to_lowercase())
          == Some(cmd_name.to_lowercase())
          && filter_args)
          || !filter_args =>
      {
        if filter_args {
          args = args.clone().into_iter().skip(1).collect();
        }
        let msg = match app.clone().try_get_matches_from(args) {
          Ok(matches) => f(C::from_arg_matches(&matches).unwrap(), &post)?,
          Err(e) => {
            if e.kind == clap::ErrorKind::MissingRequiredArgument && e.info == ["<NAME_WITH_HASH>"]
            {
              None
            } else {
              let s = e.to_string();
              let e = if e.kind == clap::ErrorKind::DisplayHelp {
                let index = s.find('\n').unwrap_or(0);
                s.split_at(index + 1).1
              } else {
                s.as_str()
              };

              let e = e
                .replace("\n\n", "\n")
                .replace("\nFor more information try --help\n", "");
              let re_help = Regex::new(
                r"If you tried to supply `[^`]+` as a PATTERN use `[^`]+`
|
If you believe you received this message in error, try re-running with '[^']+'",
              )
              .expect("invalid regex");
              let e = re_help.replace_all(&e, "").to_string();
              Some(e)
            }
          }
        };
        Ok(msg.map(|msg| (nick_name.clone(), msg)))
      }
      _ => Ok(None),
    }
  })
}
