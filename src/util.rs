use std::{
  fs,
  path::{Path, PathBuf},
};

pub fn get_all_children<P: AsRef<Path>>(path: P) -> IterAllChildren {
  IterAllChildren {
    to_explore: vec![path.as_ref().to_owned()],
    to_yield: Vec::new(),
  }
}

pub struct IterAllChildren {
  /// directories to explore
  to_explore: Vec<PathBuf>,
  to_yield: Vec<PathBuf>,
}

impl Iterator for IterAllChildren {
  type Item = PathBuf;

  fn next(&mut self) -> Option<Self::Item> {
    if self.to_yield.is_empty() {
      let explore = self.to_explore.pop()?;
      let read_dir = fs::read_dir(&explore).ok()?;
      for entry in read_dir {
        let Ok(entry) = entry else { continue };
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
