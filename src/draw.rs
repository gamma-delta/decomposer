use crate::app::DecomposerApp;
use eframe::egui;

impl DecomposerApp {
  /// Pull this function out into its own file because i like doing that
  pub fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("Decomposer");
      ui.add_space(20.0);

      ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));

      // Update
      self.main_sink.set_volume(self.volume * 0.2);
    });
  }
}
