use std::collections::VecDeque;

use eframe::{egui, CreationContext};

use crate::{model::Track, util};

pub struct DecomposerApp {
  pub main_sink: Sink,
  /// This is RAII, so we need to keep it alive the whole time
  pub stream: OutputStream,

  pub volume: f32,
  pub queue: VecDeque<Track>,
}

impl DecomposerApp {
  pub fn init(cc: &CreationContext<'_>) -> DecomposerApp {
    let (stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();

    // For now!
    let dir = "~/Music/decomposer";
    let queue = util::get_all_children(dir)
      .map(|path| Track { path })
      .collect();

    DecomposerApp {
      main_sink: sink,
      stream,
      volume: 1.0,
      queue,
    }
  }
}

impl eframe::App for DecomposerApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    self.draw(ctx, frame);
  }

  fn persist_native_window(&self) -> bool {
    true
  }
}
