use chrono::Duration;
use itertools::Itertools;
use regex::Regex;
use url::Url;

use super::{simple_enhancer, LinkEnhancer};

pub fn youtube_link_enhancer() -> impl LinkEnhancer {
  simple_enhancer(|(stated_url, extra_texts)| {
    let mut stated_url = stated_url.clone();
    let mut extra_texts = extra_texts.clone();

    if let Some(y) = Youtube::from_url(&stated_url.get_url()) {
      let enh_url = y.to_url();
      let mut enh_url_with_www = enh_url.clone();
      enh_url_with_www
        .set_host(Some("www.youtube.com"))
        .expect("could not set host");
      if stated_url
        .get_url()
        .query_pairs()
        .filter(|(name, _)| name == "list")
        .collect::<Vec<_>>()
        .len()
        > 0
      {
        stated_url.set_url(enh_url);
      }

      if let Some(a) = y.annotation() {
        extra_texts.push(a);
      }
    }
    (stated_url, extra_texts)
  })
}

struct Youtube {
  vid_id: String,
  start_time: Option<Duration>,

  metadata: Option<youtube_dl::model::SingleVideo>,
}
impl Youtube {
  fn new(url: &Url, vid_id: String) -> Option<Self> {
    if let Some(output) = std::process::Command::new("youtube-dl")
      .args(&["-j", &url.to_string()])
      .output()
      .ok()
    {
      {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if stderr.len() > 0 {
          log::info!("[youtube.rs] stderr of youtube-dl is:\n{}", stderr);
        }
      }
      let metadata = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok();
      Some(Self {
        vid_id,
        start_time: None,
        metadata,
      })
    } else {
      let start_time = Self::parse_start_time_from_url(url);
      Some(Self {
        vid_id,
        start_time,
        metadata: None,
      })
    }
  }
  pub fn from_url(url: &Url) -> Option<Self> {
    let re_hostname = Regex::new(r"https://(www\.)(youtube\.com)").expect("invalid regex");
    let re_hostname_shortened = Regex::new(r"https://youtu\.be").expect("invialid regex");

    if re_hostname.is_match(&url.to_string()) {
      let vid_id = url
        .query_pairs()
        .filter(|pair| pair.clone().0 == "v")
        .map(|pair| pair.1)
        .last()?
        .to_string();
      Self::new(url, vid_id)
    } else if re_hostname_shortened.is_match(&url.to_string()) {
      let vid_id = url.path().to_owned();
      Self::new(url, vid_id)
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
      query.insert("t".to_owned(), start_time.num_seconds().to_string());
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
    let sections = vec![
      self.title(),
      self.format_duration().map(|dur| format!("[{}]", dur)),
    ];

    Some(sections.iter().flatten().map(|s| s.to_owned()).join(" "))
  }
}
