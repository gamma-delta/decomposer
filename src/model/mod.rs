use std::path::PathBuf;

use creek::{ReadDiskStream, SymphoniaDecoder};

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

// The audio player needs to live on another thread so communicate via messages

pub enum MsgThreadToUi {
  FinishedTrack,
  PlayheadPos(usize),
  Stop,
}

pub enum MsgUiToThread {
  StartNewTrack(ReadDiskStream<SymphoniaDecoder>),

  Resume,
  Pause,
  Stop,
  SeekTo(usize),

  SetLooping(bool),
}

pub enum PlayingState<T> {
  /// Nothing's playing
  Stopped,
  /// Something is at the bottom of the screen;
  /// check if it's playing or not.
  Selected { track: T, playing: bool },
}
