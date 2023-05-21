//! The processing that lives on the cpal audio thread.

use cpal::OutputCallbackInfo;
use creek::{ReadDiskStream, SeekMode, SymphoniaDecoder};
use log::error;
use rtrb::{Consumer, Producer};

use crate::model::{MsgThreadToUi, MsgUiToThread, PlayingState};

pub const OUTPUT_CHANNEL_COUNT: usize = 2;

type ThreadPlayingState = PlayingState<ReadDiskStream<SymphoniaDecoder>>;

/// The struct that goes and lives on the audio thread.
pub struct DecomposerAudioDaemont {
  playback_state: ThreadPlayingState,
  looping: bool,

  tx_to_ui: Producer<MsgThreadToUi>,
  rx_from_ui: Consumer<MsgUiToThread>,

  /// I don't know what this does IDK ask Creek
  had_cache_miss_last_cycle: bool,
}

/// It's like a daemon, but it's not
impl DecomposerAudioDaemont {
  pub fn new(
    tx_to_ui: Producer<MsgThreadToUi>,
    rx_from_ui: Consumer<MsgUiToThread>,
  ) -> Self {
    Self {
      tx_to_ui,
      rx_from_ui,

      playback_state: ThreadPlayingState::Stopped,
      had_cache_miss_last_cycle: false,
      looping: false,
    }
  }

  pub fn process(&mut self, data: &mut [f32], _callback: &OutputCallbackInfo) {
    while let Ok(msg) = self.rx_from_ui.pop() {
      self.take_msg(msg);
    }

    let res = self.finagle_audio_state(data, _callback);
    if let Err(err) = res {
      // oh no
      error!("{}", err);
      self.playback_state = ThreadPlayingState::Stopped;
    }
  }

  fn take_msg(&mut self, msg: MsgUiToThread) {
    match msg {
      MsgUiToThread::StartNewTrack(stream) => {
        self.playback_state = ThreadPlayingState::Selected {
          track: stream,
          playing: true,
        }
      }
      MsgUiToThread::Resume => {
        if let ThreadPlayingState::Selected {
          ref mut playing, ..
        } = self.playback_state
        {
          *playing = true;
        }
      }
      MsgUiToThread::Pause => {
        if let ThreadPlayingState::Selected {
          ref mut playing, ..
        } = self.playback_state
        {
          *playing = false;
        }
      }
      MsgUiToThread::Stop => {
        self.playback_state = ThreadPlayingState::Stopped;
      }

      MsgUiToThread::SeekTo(pos) => {
        if let ThreadPlayingState::Selected { ref mut track, .. } =
          self.playback_state
        {
          let _ignore = track.seek(pos, creek::SeekMode::Auto);
        }
      }
      MsgUiToThread::SetLooping(looping) => {
        self.looping = looping;
      }
    }
  }

  fn finagle_audio_state(
    &mut self,
    mut data: &mut [f32],
    _callback: &OutputCallbackInfo,
  ) -> eyre::Result<()> {
    // I would be doing this with the slick new let-else but the formatter
    // does not like it
    let (stream, playing) = if let ThreadPlayingState::Selected {
      ref mut track,
      playing,
    } = self.playback_state
    {
      (track, playing)
    } else {
      return Ok(());
    };

    // The original app injects silence; instead I will pause until things
    // are ok
    if !stream.is_ready()? {
      return Ok(());
    }

    if playing {
      let frame_count = stream.info().num_frames;
      let channel_count = stream.info().num_channels as usize;
      // The original code here has the magic number 2 as a divisor;
      // I'm not sure if it gracefully handles files with non-2 channels.
      // Code and find out, I guess
      while data.len() >= channel_count {
        let must_read_count = data.len() / OUTPUT_CHANNEL_COUNT;
        let mut playhead = stream.playhead();

        // Suck the data off disc
        let read_data = stream.read(must_read_count)?;
        let actually_read_count = read_data.num_frames();
        playhead += actually_read_count;

        let must_loop = self.looping && playhead >= frame_count;
        let write_count = if must_loop {
          read_data.num_frames() - (playhead - frame_count)
        } else {
          read_data.num_frames()
        };

        // Copy all of the read data (no looping)
        if read_data.num_channels() == 1 {
          let ch = read_data.read_channel(0);

          for i in 0..write_count {
            data[i * OUTPUT_CHANNEL_COUNT] = ch[i];
            data[(i * OUTPUT_CHANNEL_COUNT) + 1] = ch[i];
          }
        } else {
          let ch1 = read_data.read_channel(0);
          let ch2 = read_data.read_channel(1);

          // For now just Delete the other channels i guess?
          // this is bad
          for chan_id in 2..read_data.num_channels() {
            read_data.read_channel(chan_id);
          }

          for i in 0..write_count {
            data[i * OUTPUT_CHANNEL_COUNT] = ch1[i];
            data[(i * OUTPUT_CHANNEL_COUNT) + 1] = ch2[i];
          }
        }

        if must_loop {
          stream.seek(0, SeekMode::Auto)?;
        }

        data = &mut data[actually_read_count * OUTPUT_CHANNEL_COUNT..];
      }
    }

    Ok(())
  }
}

fn make_silent(data: &mut [f32]) {
  for s in data.iter_mut() {
    *s = 0.0;
  }
}
