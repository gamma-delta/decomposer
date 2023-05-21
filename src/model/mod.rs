use std::path::PathBuf;

use creek::{FileInfo, ReadDiskStream, SymphoniaDecoder};
use symphonia::core::codecs::CodecParameters;

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

#[derive(derive_debug::Dbg)]
pub struct CurrentlyPlayingTrack {
  pub track: Track,
  pub playhead: usize,
  #[dbg(placeholder = "...")]
  pub file_info: FileInfo<CodecParameters>,
}

// The audio player needs to live on another thread so communicate via messages

pub enum MsgThreadToUi {
  FinishedTrack,
  PlayheadPos(usize),
  Stop,
  Buffering,
}

pub enum MsgUiToThread {
  StartNewTrack(ReadDiskStream<SymphoniaDecoder>),

  Resume,
  Pause,
  Stop,
  SeekTo(usize),

  SetLooping(bool),
}

#[derive(Debug)]
pub enum PlayingState<T> {
  /// Nothing's playing
  Stopped,
  /// Something is at the bottom of the screen;
  /// check if it's playing or not.
  Selected { track: T, playing: bool },
}
