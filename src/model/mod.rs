use std::path::PathBuf;

/// Generator for tracks.
#[derive(Debug, Clone)]
pub struct Playlist {
  name: String,
  tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
  pub path: PathBuf,
}
