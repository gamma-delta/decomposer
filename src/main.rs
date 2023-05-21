// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod draw;
mod model;
mod settings;
mod util;

use app::DecomposerApp;

fn main() -> Result<(), eframe::Error> {
  let env = env_logger::Env::default().default_filter_or("decomposer");
  env_logger::init_from_env(env);

  let options = eframe::NativeOptions {
    ..Default::default()
  };

  eframe::run_native(
    concat!("Decomposer"),
    options,
    Box::new(|cc| Box::new(DecomposerApp::init(cc))),
  )
}
