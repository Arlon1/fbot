use chrono::Duration;
use diesel_chrono_duration::ChronoDurationProxy;
use itertools::Itertools;
use log::error;
use regex::Regex;
use serde_json::value::Value;
use std::ops::Deref;
use url::Url;

use super::youtube_dl::*;

#[derive(serde::Deserialize, Debug, Clone)]
struct YoutubeDlVideo {
  title: String,
  //title: Option<String>,
  channel: Option<String>,
  duration: Option<Value>,
  start_time: Option<f64>,
}

#[derive(Debug)]
pub struct Youtube {
  vid_id: String,
  start_time: Option<Duration>,

  yt_dl: Option<YoutubeDlVideo>,
}
impl Youtube {
  fn new(url: &Url, vid_id: &mut String) -> Result<Self, String> {
    let vid_id_length = 11;
    vid_id.truncate(vid_id_length);
    let vid_id = vid_id.deref().to_owned();

    let mut url = url.clone();

    let pairs = url.query_pairs();
    let query_str = pairs
      .filter(|(name, _)| !["list", "index"].contains(&&name.clone().into_owned()[..]))
      .map(|(name, value)| format!("{}={}", name, value))
      .join("&");
    url.set_query(Some(&query_str));

    match youtube_dl(&url) {
      Ok(output) => {
        let metadata: Option<YoutubeDlVideo> =
          Some(serde_json::from_str(&output).map_err(|err| err.to_string())?);
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
          yt_dl: metadata,
        })
      }
      Err(err) => match err {
        YoutubeDlError::NetworkError => {
          let start_time = Self::parse_start_time_from_url(&url);
          Ok(Self {
            vid_id,
            start_time,
            yt_dl: None,
          })
        }
        _ => Err(err.to_string()),
      },
    }
  }

  pub fn parse_vid_id(url: &Url) -> Option<String> {
    let re_hostname = Regex::new(r"(www\.)?(youtube\.com)").expect("invalid regex");
    let re_hostname_shortened = Regex::new(r"youtu\.be").expect("invialid regex");

    if re_hostname.is_match(&url.host_str().unwrap_or("")) {
      let vid_id = url
        .query_pairs()
        .filter(|pair| pair.clone().0 == "v")
        .map(|pair| pair.1)
        .last()?
        .to_string();
      Some(vid_id)
    } else if re_hostname_shortened.is_match(&url.host_str().unwrap_or("")) {
      let vid_id = url.path().to_owned();
      Some(vid_id)
    } else {
      None
    }
  }

  pub fn from_url(url: &Url) -> Option<Self> {
    let mut vid_id = Self::parse_vid_id(url)?;
    Self::new(url, &mut vid_id)
      .map_err(|err| error!("{}", err))
      .ok()
  }

  pub fn title(&self) -> Option<String> {
    Some(self.yt_dl.as_ref()?.title.to_owned())
  }
  pub fn channel(&self) -> Option<String> {
    self.yt_dl.as_ref()?.channel.to_owned()
  }
  pub fn duration(&self) -> Option<Duration> {
    Some(Duration::seconds(
      self.yt_dl.as_ref()?.duration.as_ref()?.as_i64()?,
    ))
  }
  pub fn start_time(&self) -> Option<Duration> {
    self.start_time
  }

  fn parse_start_time_parameter(start_time: String) -> Option<Duration> {
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
    let start_time = {
      if let Some(frag) = url.fragment() {
        frag.to_owned()
      } else {
        url
          .query_pairs()
          .filter(|pair| pair.clone().0 == "t")
          .map(|pair| pair.1)
          .last()?
          .into_owned()
      }
    };

    Self::parse_start_time_parameter(start_time)
  }
  fn format_duration(&self) -> Option<String> {
    let duration = self.duration()?;

    let mut duration_str = "".to_owned();

    let hours = duration.num_hours();
    if hours > 0 {
      duration_str += &format!("{}:", hours.to_string());
    }

    let min_gesamt = duration.num_minutes();
    let min_display = min_gesamt - hours * 60;
    if min_gesamt > 0 {
      if duration_str.len() > 0 {
        duration_str += &format!("{:02}:", min_display);
      } else {
        duration_str += &format!("{}:", min_display);
      }
    }

    let s_gesamt = &duration.num_seconds();
    let s_display = s_gesamt - min_gesamt * 60;
    if duration_str.len() > 0 {
      duration_str += &format!("{:02}", s_display);
    } else {
      duration_str += &format!("{}s", s_display.to_string());
    }

    Some(duration_str)
  }

  pub fn annotation(&self) -> Option<String> {
    let title_max_len = 90;
    let channel_max_len = 25;
    let max_len = title_max_len + channel_max_len;

    let title = self.title();
    let mut channel = self
      .yt_dl
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

    channel = channel.map(|channel| {
      if let Some(ch) = channel.strip_suffix(" - Topic") {
        ch.to_owned()
      } else {
        channel
      }
    });

    if let Some(title) = &title {
      if let Some(ch) = &channel {
        if title.contains(ch) {
          channel = None;
        }
      }
    }
    channel = channel.map(|ch| "– ".to_owned() + &ch);

    fn none_if_empty(s: &&Option<String>) -> Option<String> {
      if s.as_ref().cloned().map(|s| s.is_empty()).unwrap_or(true) {
        None
      } else {
        s.as_ref().cloned()
      }
    }
    if title.as_ref().cloned().unwrap_or("".to_owned()).len()
      + channel.as_ref().cloned().unwrap_or("".to_owned()).len()
      <= max_len
    {
      Some(
        [
          &title.as_ref().cloned(),
          &channel.as_ref().cloned(),
          &duration,
        ]
        .iter()
        .map(|s| none_if_empty(s))
        .flatten()
        .join(" "),
      )
    } else {
      let ch_tr = truncate_render(channel, channel_max_len).map(|(s, _)| s);
      if &title.as_ref().cloned().unwrap_or("".to_owned()).len()
        + ch_tr.as_ref().cloned().unwrap_or("".to_owned()).len()
        <= max_len
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

  pub fn to_url(&self) -> Url {
    let mut url = Url::parse("https://www.youtube.com/watch").expect("invalid url");

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
        .join("&")[..],
    ));

    url
  }
  pub fn to_metadata(&self) -> crate::models::UrlMetadata {
    crate::models::UrlMetadata {
      url: self.to_url().to_string(),
      title: self.title(),
      author: self.channel(),
      duration: self.duration().map(|dur| ChronoDurationProxy(dur)),
      start_time: self.start_time().map(|dur| ChronoDurationProxy(dur)),
    }
  }
  pub fn from_metadata(metadata: &crate::models::UrlMetadata) -> Option<Self> {
    let url = Url::parse(&metadata.url).ok()?;

    Some(Self {
      vid_id: Self::parse_vid_id(&url)?,
      start_time: metadata.start_time.map(|dur| *dur),
      yt_dl: Some(YoutubeDlVideo {
        title: metadata.title.as_ref().cloned().unwrap_or("".to_owned()),
        channel: metadata.author.clone(),
        duration: Some(serde_json::json!(metadata
          .duration
          .map(|t| (*t).num_seconds()))),
        start_time: metadata.start_time.map(|t| (*t).num_seconds() as f64),
      }),
    })
  }
}

impl std::cmp::PartialEq for Youtube {
  fn eq(&self, other: &Self) -> bool {
    // deliberately ignoring `start_time`
    self.vid_id == other.vid_id
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
