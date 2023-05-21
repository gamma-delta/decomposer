use std::{
  fs,
  path::{Path, PathBuf},
};

use directories_next::UserDirs;
use eyre::bail;
use log::{info, warn};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

pub const CONFIG_LOCATION_KEY: &str = "config-location";

pub const DEFAULT_CONFIG_LOCATION: &str = ".decomposer.ron";

#[derive(Debug)]
pub struct DecomposerConfig {
  cfg_location: PathBuf,
  inner: DecomposerConfigSerde,
}

#[derive(Serialize, Deserialize, Debug)]
struct DecomposerConfigSerde {
  library_root: PathBuf,
  volume: f32,
}

impl DecomposerConfig {
  /// Try to read the settings from the settings file,
  /// otherwise return the default.
  ///
  /// Give None if no config file is known
  pub fn open(path: Option<&str>) -> eyre::Result<DecomposerConfig> {
    let Some(ud) = UserDirs::new() else {
      bail!("Could not get user dirs somehow, so we can't make defaults. Ouch. If you're seeing this error hopefully you're computer-savvy enough to figure it out")
    };
    let path = match path {
      Some(it) => PathBuf::from(it),
      None => {
        info!(
          "{} was not found in the egui persistent data. Using default",
          &CONFIG_LOCATION_KEY,
        );
        ud.home_dir().join(DEFAULT_CONFIG_LOCATION)
      }
    };

    let cfg: DecomposerConfigSerde = 'ok: {
      let cfg_src = match fs::read_to_string(&path) {
        Ok(it) => it,
        Err(err) => {
          warn!(
            "Could not open {:?} for config. Using default config. {}",
            &path, err
          );
          // we have try blocks at home
          break 'ok default_config(&ud)?;
        }
      };

      match ron::from_str(&cfg_src) {
        Ok(it) => it,
        Err(err) => {
          warn!(
            "Could not parse contents of {:?}. Using default config. {}",
            &path, err
          );
          default_config(&ud)?
        }
      }
    };

    Ok(DecomposerConfig {
      cfg_location: path,
      inner: cfg,
    })
  }

  pub fn save(&self) {
    let ron_src =
      match ron::ser::to_string_pretty(&self.inner, pretty_ser_config()) {
        Ok(it) => it,
        Err(err) => {
          warn!("Could not serialize config to ron: {}", err);
          return;
        }
      };

    if let Err(err) = fs::write(&self.cfg_location, ron_src.as_bytes()) {
      warn!(
        "Could not save ron config to {:?}: {}",
        &self.cfg_location, err
      );
    }
  }

  pub fn cfg_location(&self) -> &Path {
    &self.cfg_location
  }

  pub fn volume(&mut self) -> &mut f32 {
    &mut self.inner.volume
  }

  pub fn library_root(&self) -> &Path {
    &self.inner.library_root
  }
}

/// Try to return the default
fn default_config(ud: &UserDirs) -> eyre::Result<DecomposerConfigSerde> {
  let audio_dir = if let Some(audio_dir) = ud.audio_dir() {
    audio_dir.to_owned()
  } else {
    warn!("Could not find audio dir, falling back to concated home dir");
    ud.home_dir().join("Music")
  };
  let root = audio_dir.join("decomposer");

  let volume = 1.0;

  let out = DecomposerConfigSerde {
    library_root: root,
    volume,
  };
  warn!("Had to regenerate config from defaults: {:#?}", &out);
  Ok(out)
}

fn pretty_ser_config() -> PrettyConfig {
  // For now
  PrettyConfig::default()
}
