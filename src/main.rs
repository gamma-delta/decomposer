// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Thanks to this for the help:
// https://github.com/MeadowlarkDAW/creek/tree/main/demos/player

mod app;
mod audio;
mod emoji;
mod model;
mod settings;
mod util;

use app::DecomposerApp;

use eyre::eyre;

fn main() -> eyre::Result<()> {
  let env = env_logger::Env::default().default_filter_or("decomposer=info");
  env_logger::init_from_env(env);

  let options = eframe::NativeOptions {
    ..Default::default()
  };

  let res = eframe::run_native(
    concat!("Decomposer"),
    options,
    Box::new(|cc| DecomposerApp::init(cc)),
  );

  if let Err(err) = res {
    let err = eyre!(err.to_string())
      .wrap_err("Could not init egui (this is very very bad)");
    Err(err)
  } else {
    Ok(())
  }
}
