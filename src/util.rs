use std::{
  fs,
  path::{Path, PathBuf},
};

use log::warn;
use symphonia::core::units::{Time, TimeStamp};

pub fn get_all_children<P: AsRef<Path>>(path: P) -> IterAllChildren {
  match path.as_ref().canonicalize() {
    Ok(path) => IterAllChildren {
      root: path.to_path_buf(),
      to_explore: vec![path.to_path_buf()],
      to_yield: Vec::new(),
    },
    Err(ono) => {
      warn!(
        "Could not canonicalize {:?} for get_all_children: {}",
        path.as_ref(),
        ono
      );
      IterAllChildren {
        root: "nope".into(),
        to_explore: Vec::new(),
        to_yield: Vec::new(),
      }
    }
  }
}

pub struct IterAllChildren {
  /// Mostly for printing information
  root: PathBuf,
  /// directories to explore
  to_explore: Vec<PathBuf>,
  /// Children to yield
  to_yield: Vec<PathBuf>,
}

impl Iterator for IterAllChildren {
  type Item = PathBuf;

  fn next(&mut self) -> Option<Self::Item> {
    while self.to_yield.is_empty() {
      // If popping the explore is empty, exit
      let explore = self.to_explore.pop()?;
      let read_dir = match fs::read_dir(&explore) {
        Ok(it) => it,
        Err(err) => {
          warn!(
            "searching file children of {:?} -> {:?}: {}",
            &self.root, &explore, err
          );
          continue;
        }
      };
      for entry in read_dir {
        let entry = match entry {
          Ok(entry) => entry,
          Err(err) => {
            warn!(
              "searching entry child of {:?} -> {:?}: {}",
              &self.root, &explore, err
            );
            continue;
          }
        };
        let Ok(ty) = entry.file_type() else { continue };

        let full_path = explore.join(entry.path());
        if ty.is_dir() {
          self.to_explore.push(full_path);
        } else if ty.is_file() {
          self.to_yield.push(full_path);
        }
      }
    }

    self.to_yield.pop()
  }
}

pub fn format_symphonia_time(time: Time) -> String {
  if time.seconds >= 60 * 60 {
    format!(
      "{}:{:02}:{:02}",
      time.seconds / 3600,
      time.seconds / 60,
      time.seconds % 60
    )
  } else {
    format!("{}:{:02}", time.seconds / 60, time.seconds % 60)
  }
}
