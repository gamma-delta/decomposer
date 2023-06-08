use std::path::PathBuf;

/// A definite track with a known location on disc.
#[derive(Debug, Clone)]
pub struct Track {
  pub path: PathBuf,
}

/// Uniquely identifies a track on disc, via diagnostic information
pub struct TrackLocator {}
