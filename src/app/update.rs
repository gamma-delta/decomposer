use creek::{
  Decoder, ReadDiskStream, ReadStreamOptions, SeekMode, SymphoniaDecoder,
};
use log::{debug, error, info, warn};

use crate::model::{CurrentlyPlayingTrack, MsgThreadToUi, MsgUiToThread};

use super::{AppPlayingState, DecomposerApp, BUFFERING_COOLDOWN};

impl DecomposerApp {
  pub fn update(&mut self) {
    while let Ok(msg) = self.rx_from_thread.pop() {
      self.take_message(msg);
    }

    if self.buffering_cooldown > 0 {
      self.buffering_cooldown -= 1;
    }
  }

  fn take_message(&mut self, msg: MsgThreadToUi) {
    debug!("Recv message on ui thread: {:?}", &msg);
    match msg {
      MsgThreadToUi::FinishedTrack => {
        self.deque_and_send_track();
      }
      MsgThreadToUi::PlayheadPos(pos) => {
        if let AppPlayingState::Selected { ref mut track, .. } =
          self.now_playing
        {
          track.playhead = pos;
        } else {
          warn!("audio thread sent playhead pos update (to {}) when we weren't playing", pos);
        }
      }
      MsgThreadToUi::Stop => {
        self.now_playing = AppPlayingState::Stopped;
      }
      MsgThreadToUi::Buffering => {
        self.buffering_cooldown = BUFFERING_COOLDOWN;
      }
    }
  }

  pub fn deque_and_send_track(&mut self) {
    while let Some(track) = self.queue.pop_front() {
      // I don't have that functionality so i will just have 1 cache
      let opts = ReadStreamOptions {
        num_cache_blocks: 20,
        num_caches: 1,
        ..Default::default()
      };

      let mut stream =
        match ReadDiskStream::<SymphoniaDecoder>::new(&track.path, 0, opts) {
          Ok(it) => it,
          Err(err) => {
            error!("Could not load file at {:?}: {}", &track.path, err);
            continue;
          }
        };

      // Cache frame 0 at cache index 0
      let _ignore = stream.cache(0, 0);
      if let Err(ono) = stream.seek(0, SeekMode::Auto) {
        error!(
          "Some kind of fascinating seeking error when sending {:?}: {}",
          &track.path, ono
        );
        continue;
      }

      info!("Sending {:?} to audio thread", &track);

      let info = stream.info().clone();
      self.now_playing = AppPlayingState::Selected {
        playing: true,
        track: CurrentlyPlayingTrack {
          track,
          playhead: 0,
          file_info: info,
        },
      };
      let _ignore =
        self.tx_to_thread.push(MsgUiToThread::StartNewTrack(stream));

      // and done!
      return;
    }
  }
}
