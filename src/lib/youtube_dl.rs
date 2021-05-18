use regex::Regex;
use std::process::Command;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum YoutubeDlError {
  #[error("This video is no longer available due to a copyright claim by {claimer}")]
  CopyrightClaim { claimer: String },
  #[error("This video is no longer available because the YouTube account associated with this video has been terminated.")]
  AccountTerminated,
  #[error("Sign in if you've been granted access to this video")]
  PrivateVideo,
  #[error("The uploader has not made this video available in your country.")]
  UnavailableInYourCountry,
  #[error("Video Unavailable")]
  VideoUnavailable,

  #[error("Unable to download webpage: Name or service not known>")]
  NetworkError,
  #[error("ERROR: No video formats found")]
  NoVideoFormats,

  #[error("{0}")]
  CommandError(String),
  #[error("{0}")]
  Other(String),
}

pub fn youtube_dl(url: &Url) -> Result<String, YoutubeDlError> {
  let re_copyright =
    Regex::new(r"This video is no longer available due to a copyright claim by (?P<claimer>.+).\n")
      .expect("invalid regex");

  let output = Command::new("youtube-dl")
    .args(&["-j", &url.to_string()])
    .output()
    .map_err(|e| YoutubeDlError::CommandError(e.to_string()))?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stderr_split = stderr
      .split("\n")
      .into_iter()
      .filter(|line| !line.starts_with("ERROR"))
      .collect::<Vec<&str>>();

    match stderr_split[0] {
      "ERROR: Video unavailable" => {
        if let Some(line) = stderr_split.iter().skip(0).nth(0) {
          if let Some(captures) = re_copyright.captures(line) {
	    let claimer = captures.name("claimer").ok_or(YoutubeDlError::Other("output malformed: no claimer found".to_owned()))?;
	    Err(YoutubeDlError::CopyrightClaim
		{claimer: claimer.as_str().to_owned()})?
	  } else if line == &"This video is no longer available because the YouTube account associated with this video has been terminated." {
	    Err(YoutubeDlError::AccountTerminated)?
	  }
	  else {
	    Err(YoutubeDlError::Other(stderr))?
	  }
        } else {
          Err(YoutubeDlError::VideoUnavailable)?
        }
      }
      "ERROR: Private video" => Err(YoutubeDlError::PrivateVideo)?,
      "ERROR: The uploader has not made this video available in your country." => {
        Err(YoutubeDlError::UnavailableInYourCountry)?
      }

      "ERROR: Unable to download API page: <urlopen error [Errno -2] Name or service not known> (caused by URLError(gaierror(-2, 'Name or service not known')))" => Err(YoutubeDlError::NetworkError)?,

      "[ERROR] ERROR: No video formats found; please report this issue on https://yt-dl.org/bug . Make sure you are using the latest version; see  https://yt-dl.org/update  on how to update. Be sure to call youtube-dl with the --verbose flag and include its complete output.
" => Err(YoutubeDlError::NoVideoFormats)?,


      _ => Err(YoutubeDlError::Other(stderr))?,
    }
  } else {
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  }
}

//ERROR: Video unavailable
//This video is no longer available due to a copyright claim by MUSO TNT LTD.
//https://www.youtube.com/watch?v=v8Djdi6z0m8
//
//ERROR: Video unavailable
//This video is no longer available because the YouTube account associated with this video has been terminated.
//https://www.youtube.com/watch?v=XzEHCCLeoN0
//
//ERROR: Video unavailable
//https://www.youtube.com/watch?v=tDvCAtRb8dE#
//https://www.youtube.com/watch?v=a3ZNFtv-mPE
//
//ERROR: Private video
//Sign in if you've been granted access to this video
//https://www.youtube.com/watch?v=PIpn68XKFK0
//
//ERROR: The uploader has not made this video available in your country.
//You might want to use a VPN or a proxy server (with --proxy) to workaround.
//https://www.youtube.com/watch?v=lRAIx3ykzFM
//
//ERROR: Unable to download API page: <urlopen error [Errno -2] Name or service not known> (caused by URLError(gaierror(-2, 'Name or service not known')))
