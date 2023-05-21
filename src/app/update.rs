use creek::{
  Decoder, ReadDiskStream, ReadStreamOptions, SeekMode, SymphoniaDecoder,
};
use log::error;

use crate::model::{MsgThreadToUi, MsgUiToThread};

use super::DecomposerApp;

impl DecomposerApp {
  pub fn update(&mut self) {
    while let Ok(msg) = self.rx_from_thread.pop() {
      self.take_message(msg);
    }
  }

  fn take_message(&mut self, msg: MsgThreadToUi) {
    match msg {
      MsgThreadToUi::FinishedTrack => {
        self.deque_and_send_track();
      }
      MsgThreadToUi::PlayheadPos(_) => todo!(),
      MsgThreadToUi::Stop => todo!(),
    }
  }

  fn deque_and_send_track(&mut self) {
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

      let _ignore =
        self.tx_to_thread.push(MsgUiToThread::StartNewTrack(stream));

      // and done!
      return;
    }
  }
}
