use std::{fs, path::PathBuf};

use directories_next::UserDirs;
use eyre::bail;
use log::warn;

pub const CONFIG_LOCATION: &str = "~/.creak.kdl";

#[derive(knuffel::Decode)]
pub struct CreakSettings {
  #[knuffel(child, unwrap(argument))]
  creak_root: PathBuf,
  #[knuffel(child, unwrap(argument))]
  volume: f32,
}

impl CreakSettings {
  /// Try to read the settings from the settings file, otherwise return the default
  pub fn open() -> eyre::Result<CreakSettings> {
    let cfg_src = match fs::read_to_string(CONFIG_LOCATION) {
      Ok(it) => it,
      Err(err) => {
        warn!("Could not open {}. Using defaults.", &CONFIG_LOCATION);
        warn!("{}", err);
        return CreakSettings::try_default();
      }
    };
    let cfg = match knuffel::parse(CONFIG_LOCATION, &cfg_src) {
      Ok(it) => it,
      Err(err) => {
        warn!(
          "Could not parse contents of {}. Using defaults.",
          &CONFIG_LOCATION
        );
        warn!("{}", err);
        return CreakSettings::try_default();
      }
    };

    Ok(cfg)
  }

  /// Try to return the default
  pub fn try_default() -> eyre::Result<CreakSettings> {
    let Some(ud) = UserDirs::new() else {
      bail!("Could not get user dirs somehow, so we can't make defaults. Ouch. If you're seeing this error hopefully you're computer-savvy enough to figure it out")
    };

    let Some(creak_root) = ud.audio_dir() else {
      bail!("Could not get user audio dir.")
    };
    let creak_root = creak_root.to_path_buf();

    let volume = 1.0;

    Ok(CreakSettings { creak_root, volume })
  }
}
