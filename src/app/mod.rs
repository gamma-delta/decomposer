mod draw;
mod update;

use std::collections::VecDeque;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::{egui, App, CreationContext, Storage};
use log::error;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
  audio::{self, DecomposerAudioDaemont},
  model::{
    CurrentlyPlayingTrack, MsgThreadToUi, MsgUiToThread, PlayingState, Track,
  },
  settings::{DecomposerConfig, CONFIG_LOCATION_KEY},
  util,
};

pub type AppPlayingState = PlayingState<CurrentlyPlayingTrack>;

const BUFFERING_COOLDOWN: u32 = 30;

pub struct DecomposerApp {
  tx_to_thread: Producer<MsgUiToThread>,
  rx_from_thread: Consumer<MsgThreadToUi>,
  #[allow(dead_code)]
  raii_stream: cpal::Stream,

  queue: VecDeque<Track>,
  now_playing: AppPlayingState,
  buffering_cooldown: u32,

  config: DecomposerConfig,
}

impl DecomposerApp {
  /// Init the app.
  /// Also spin up the audio context.
  ///
  /// Because egui doesn't support exiting after it gives you the ctx,
  /// but before it begins drawing, we return a box.
  /// If there's an error here we return a dummy impl that prints the error and
  /// exits on the first frame.
  pub fn init(cc: &CreationContext<'_>) -> Box<dyn App> {
    match DecomposerApp::init_inner(cc) {
      Ok(app) => Box::new(app),
      Err(error) => Box::new(StartupFailureApp { error }),
    }
  }

  fn init_inner(cc: &CreationContext<'_>) -> eyre::Result<DecomposerApp> {
    let storage = cc.storage.expect("compiled with `persistence`");
    let config = DecomposerConfig::open(
      storage.get_string(CONFIG_LOCATION_KEY).as_deref(),
    )?;

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

    let queue = util::get_all_children(&config.library_root())
      .map(|path| Track { path })
      .collect();

    Ok(DecomposerApp {
      config,
      queue,

      tx_to_thread,
      rx_from_thread,
      raii_stream: stream,

      now_playing: PlayingState::Stopped,
      buffering_cooldown: 0,
    })
  }
}

impl eframe::App for DecomposerApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    self.update();
    self.draw(ctx, frame)
  }

  fn save(&mut self, storage: &mut dyn Storage) {
    storage.set_string(
      CONFIG_LOCATION_KEY,
      self.config.cfg_location().to_string_lossy().into_owned(),
    );

    self.config.save();
  }

  fn persist_native_window(&self) -> bool {
    true
  }
}

struct StartupFailureApp {
  error: eyre::Report,
}

impl eframe::App for StartupFailureApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    error!("Fatal error during startup: {}", &self.error);
    frame.close();
  }
}
