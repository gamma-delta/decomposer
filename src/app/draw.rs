use crate::app::DecomposerApp;
use eframe::egui::{self, ProgressBar};

impl DecomposerApp {
  /// Pull this function out into its own file because i like doing that
  pub fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
      ui.heading("Decomposer");
    });
    egui::TopBottomPanel::bottom("bottom")
      .resizable(false)
      .default_height(40.0)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          // TODO: figure out how to get the length of a sound file
          // I think it's in params.
          // https://docs.rs/symphonia-core/0.5.2/symphonia_core/units/struct.TimeBase.html
          // Not sure what to multiply there
        })
      });

    // instead of the janky thread-spam, just do this
    ctx.request_repaint();
  }
}
