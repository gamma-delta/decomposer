use crate::CreakApp;

impl CreakApp {
  /// Pull this function out into its own file because i like doing that
  pub fn draw(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("Creak");
      ui.add_space(20.0);

      ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));

      // Update
      self.main_sink.set_volume(self.volume * 0.2);
    });
  }
}
