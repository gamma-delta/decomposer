mod draw;
mod update;

use std::collections::VecDeque;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::{egui, CreationContext};
use log::error;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
  audio::{self, DecomposerAudioDaemont},
  model::{MsgThreadToUi, MsgUiToThread, Track},
  util,
};

pub struct DecomposerApp {
  pub volume: f32,
  pub queue: VecDeque<Track>,

  pub tx_to_thread: Producer<MsgUiToThread>,
  pub rx_from_thread: Consumer<MsgThreadToUi>,
}

impl DecomposerApp {
  /// Also spin up the audio context
  pub fn init(cc: &CreationContext<'_>) -> DecomposerApp {
    let (tx_to_thread, rx_from_ui) = RingBuffer::new(64);
    let (tx_to_ui, rx_from_thread) = RingBuffer::new(256);

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let sample_rate = device.default_output_config().unwrap().sample_rate();
    let audio_cfg = cpal::StreamConfig {
      channels: audio::OUTPUT_CHANNEL_COUNT as u16,
      sample_rate,
      buffer_size: cpal::BufferSize::Default,
    };

    let mut looks_like_youre_going_to_the_shadow_thread_jimbo =
      DecomposerAudioDaemont::new(tx_to_ui, rx_from_ui);

    let stream = device
      .build_output_stream(
        &audio_cfg,
        move |data: &mut [f32], ci| {
          looks_like_youre_going_to_the_shadow_thread_jimbo.process(data, ci);
        },
        move |err| {
          error!("{}", err);
        },
        None,
      )
      .unwrap();
    stream.play().unwrap();

    // For now!
    let dir = "~/Music/decomposer";
    let queue = util::get_all_children(dir)
      .map(|path| Track { path })
      .collect();
    println!("{:?}", &queue);

    DecomposerApp {
      volume: 1.0,
      queue,
      tx_to_thread,
      rx_from_thread,
    }
  }
}

impl eframe::App for DecomposerApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    self.update();
    self.draw(ctx, frame);
  }

  fn persist_native_window(&self) -> bool {
    true
  }
}
