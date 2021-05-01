use chrono::Duration;
use itertools::Itertools;
use log::error;
use regex::Regex;
use serde_json::value::Value;
use std::process::Command;
use url::Url;

use super::{simple_enhancer, LinkEnhancer};

pub fn youtube_link_enhancer() -> impl LinkEnhancer {
  simple_enhancer(|(stated_url, extra_texts)| {
    let mut stated_url = stated_url.clone();
    let mut extra_texts = extra_texts.clone();

    if let Some(y) = Youtube::from_url(&stated_url.get_url()) {
      if y.was_enhanced {
        stated_url.set_url(y.to_url());
      }

      if let Some(a) = y.annotation() {
        extra_texts.push(a);
      }
    }
    (stated_url, extra_texts)
  })
}

#[derive(serde::Deserialize, Debug, Clone)]
struct SingleVideo {
  title: String,
  channel: Option<String>,
  duration: Option<Value>,
  start_time: Option<f64>,
}

#[derive(Debug)]
struct Youtube {
  vid_id: String,
  start_time: Option<Duration>,

  was_enhanced: bool,
  url: Url,

  metadata: Option<SingleVideo>,
}
impl Youtube {
  fn new(url: &Url, vid_id: String) -> Result<Self, String> {
    let mut url = url.clone();
    let mut was_enhanced = false;
    let query_filtered = url
      .query_pairs()
      .filter(|(key, _value)| !vec!["list", "index"].contains(&&key.to_owned().to_string()[..]))
      .map(|(key, value)| key + "=" + value)
      .join("&");
    if url.query().unwrap_or("") != query_filtered {
      url.set_query(Some(&query_filtered));
      was_enhanced = true;
    }

    if let Ok(output) = Command::new("youtube-dl")
      .args(&["-j", &url.to_string()])
      .output()
      .map_err(|err| err.to_string())
    {
      if !output.status.success() {
        error!(
          "youtube-dl exited with {} and said\n{}\nurl was {}",
          match output.status.code() {
            Some(code) => format!("Exited with status code: {}", code),
            None => format!("Process terminated by signal"),
          },
          String::from_utf8_lossy(&output.stderr),
          url.to_string()
        );
      }

      let metadata: Option<SingleVideo> = Some(
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
          .map_err(|err| err.to_string())?,
      );
      let start_time = {
        if let Some(metadata) = &metadata {
          Some(Duration::seconds(metadata.start_time.unwrap_or(0.0) as i64))
        } else {
          None
        }
      };

      Ok(Self {
        vid_id,
        start_time,
        url: url.to_owned(),
        was_enhanced,
        metadata,
      })
    } else {
      let start_time = Self::parse_start_time_from_url(&url);
      Ok(Self {
        vid_id,
        start_time,
        url: url.to_owned(),
        was_enhanced,
        metadata: None,
      })
    }
  }
  pub fn from_url(url: &Url) -> Option<Self> {
    let re_hostname = Regex::new(r"(www\.)?(youtube\.com)").expect("invalid regex");
    let re_hostname_shortened = Regex::new(r"youtu\.be").expect("invialid regex");

    if re_hostname.is_match(&url.host_str().unwrap_or("")) {
      let vid_id = url
        .query_pairs()
        .filter(|pair| pair.clone().0 == "v")
        .map(|pair| pair.1)
        .last()?
        .to_string();
      Some(
        Self::new(url, vid_id)
          .map_err(|err| {
            error!("{}", err);
          })
          .ok()?,
      )
    } else if re_hostname_shortened.is_match(&url.host_str().unwrap_or("")) {
      let vid_id = url.path().to_owned();
      Some(
        Self::new(url, vid_id)
          .map_err(|err| error!("{}", err))
          .ok()?,
      )
    } else {
      None
    }
  }

  fn title(&self) -> Option<String> {
    Some(self.metadata.as_ref()?.title.to_owned())
  }
  fn duration(&self) -> Option<Duration> {
    Some(Duration::seconds(
      self.metadata.as_ref()?.duration.as_ref()?.as_i64()?,
    ))
  }
  fn format_duration(&self) -> Option<String> {
    let duration = self.duration()?;

    let mut duration_str = "".to_owned();

    let hours = duration.num_hours();
    if hours > 0 {
      duration_str += &format!("{}:", hours.to_string());
    }

    let min_gesamt = duration.num_minutes();
    let min_display = min_gesamt - hours * 60 * 60;
    if min_gesamt > 0 {
      if duration_str.len() > 0 {
        duration_str += &format!("{:02}:", min_display);
      } else {
        duration_str += &format!("{}:", min_display.to_string());
      }
    }

    let s_gesamt = &duration.num_seconds();
    let s_display = s_gesamt - min_gesamt * 60 - hours * 60 * 60;
    if duration_str.len() > 0 {
      duration_str += &format!("{:02}", s_display);
    } else {
      duration_str += &format!("{}s", s_display.to_string());
    }

    Some(duration_str)
  }

  pub fn start_time(&self) -> Option<Duration> {
    self.start_time
  }
  fn parse_start_time(start_time: String) -> Option<Duration> {
    let re_splitted =
      Regex::new("(?P<h>[[:digit:]]+h)?(?P<min>[[:digit:]]+min)?(?P<s>[[:digit:]]+s)")
        .expect("invalid_regex");
    if let Some(start_time) = start_time.parse::<i64>().ok() {
      Some(Duration::seconds(start_time))
    } else if let Some(captures) = re_splitted.captures(&start_time) {
      Some(
        Duration::hours(
          captures
            .name("h")
            .map(|cap| cap.as_str())
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0),
        ) + Duration::minutes(
          captures
            .name("min")
            .map(|cap| cap.as_str())
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0),
        ) + Duration::seconds(
          captures
            .name("s")
            .map(|cap| cap.as_str())
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0),
        ),
      )
    } else {
      None
    }
  }
  pub fn parse_start_time_from_url(url: &Url) -> Option<Duration> {
    let start_time = url
      .query_pairs()
      .filter(|pair| pair.clone().0 == "t")
      .map(|pair| pair.1)
      .last()?
      .into_owned();
    Self::parse_start_time(start_time)
  }

  pub fn to_url(&self) -> Url {
    let mut url = Url::parse("https://youtube.com/watch").expect("invalid url");

    let mut query = std::collections::HashMap::<_, _>::new();
    query.insert("v".to_owned(), self.vid_id.clone());
    if let Some(start_time) = self.start_time() {
      let start_time_sec = start_time.num_seconds();
      if start_time_sec != 0 {
        query.insert("t".to_owned(), start_time_sec.to_string());
      }
    }
    url.set_query(Some(
      &query
        .iter()
        .map(|(name, value)| name.to_string() + "=" + value)
        .collect::<String>()[..],
    ));

    url
  }

  pub fn annotation(&self) -> Option<String> {
    let title_max_len = 90;
    let channel_max_len = 25;
    let max_len = title_max_len + channel_max_len;

    let title = self.title();
    let mut channel = self
      .metadata
      .clone()
      .map(|m| {
        if let Some(channel) = m.channel {
          Some(channel)
        } else {
          None
        }
      })
      .flatten();
    let duration = self.format_duration().map(|dur| format!("[{}]", dur));

    if let Some(title) = &title {
      if let Some(ch) = &channel {
        if title.contains(ch) {
          channel = None;
        }
      }
    }
    channel = channel.map(|ch| "– ".to_owned() + &ch);

    if title.as_ref().cloned().unwrap_or("".to_owned()).len()
      + 1
      + channel.as_ref().cloned().unwrap_or("".to_owned()).len()
      > max_len
    {
      Some(
        [
          &title.as_ref().cloned(),
          &channel.as_ref().cloned(),
          &duration,
        ]
        .iter()
        .cloned()
        .flatten()
        .join(" "),
      )
    } else {
      let ch_tr = truncate_render(channel, channel_max_len).map(|(s, _)| s);
      if &title.as_ref().cloned().unwrap_or("".to_owned()).len()
        + ch_tr.as_ref().cloned().unwrap_or("".to_owned()).len()
        < max_len
      {
        Some(
          [
            &title.as_ref().cloned(),
            &ch_tr.as_ref().cloned(),
            &duration,
          ]
          .iter()
          .cloned()
          .flatten()
          .join(" "),
        )
      } else {
        let title_tr = truncate_render(title, title_max_len).map(|(s, _)| s);
        Some([title_tr, ch_tr, duration].iter().flatten().join(" "))
      }
    }
  }
}

fn truncate_render(s: Option<String>, len: usize) -> Option<(String, bool)> {
  let ellipsis = "...".to_owned();
  if let Some(s) = s {
    if s.len() > len {
      let mut s = s.clone();
      s.truncate(len - ellipsis.len());
      s += &ellipsis;
      Some((s, true))
    } else {
      Some((s, false))
    }
  } else {
    None
  }
}
