// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod draw;

use rodio::{source::SineWave, OutputStream, Sink};

fn main() -> Result<(), eframe::Error> {
  let env = env_logger::Env::default().default_filter_or("creak");
  env_logger::init_from_env(env);

  let options = eframe::NativeOptions {
    ..Default::default()
  };

  eframe::run_native(
    concat!("Creak v", env!("CARGO_PKG_VERSION")),
    options,
    Box::new(|_cc| Box::new(CreakApp::init())),
  )
}

pub struct CreakApp {
  pub main_sink: Sink,
  /// This is RAII, so we need to keep it alive the whole time
  pub stream: OutputStream,

  pub volume: f32,
}

impl CreakApp {
  pub fn init() -> CreakApp {
    let (stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();
    // For now just make The Sin Player
    let src = SineWave::new(440.0);

    sink.append(src);

    CreakApp {
      main_sink: sink,
      stream,
      volume: 1.0,
    }
  }
}

impl eframe::App for CreakApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    self.draw(ctx, frame);
  }
}
