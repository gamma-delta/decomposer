use crate::{app::DecomposerApp, emoji, model::PlayingState};

use eframe::egui::{
  self, Button, CentralPanel, ImageButton, Label, ProgressBar, RichText,
  ScrollArea, TextStyle, TopBottomPanel, WidgetText,
};

impl DecomposerApp {
  /// Pull this function out into its own file because i like doing that
  pub fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    TopBottomPanel::top("top").show(ctx, |ui| {
      ui.horizontal_centered(|ui| self.draw_top_tab_bar(ui));
    });
    TopBottomPanel::bottom("bottom")
      .resizable(false)
      .default_height(40.0)
      .show(ctx, |ui| {
        ui.vertical(|ui| self.draw_bottom_bar(ui));
      });

    CentralPanel::default().show(ctx, |ui| {
      self.draw_queue(ui);
    });

    // instead of the janky thread-spam, just do this
    ctx.request_repaint();
  }

  fn draw_top_tab_bar(&mut self, ui: &mut eframe::egui::Ui) {
    egui::widgets::global_dark_light_mode_switch(ui);

    ui.label(concat!("Decomposer v", env!("CARGO_PKG_VERSION")))
      .on_hover_text("Test!");
  }

  // In a vert layout
  fn draw_bottom_bar(&mut self, ui: &mut eframe::egui::Ui) {
    ui.horizontal(|ui| {
      ui.button(emoji::REPEAT);
      ui.button(emoji::SHUFFLE);
      ui.separator();

      let (wind_enabled, playpause_label) = match self.now_playing {
        PlayingState::Stopped => (false, emoji::PLAYING),
        PlayingState::Selected { playing: true, .. } => (true, emoji::PAUSING),
        PlayingState::Selected { playing: false, .. } => (true, emoji::PLAYING),
      };

      // Currently no-op
      ui.add_enabled(wind_enabled, Button::new(emoji::WIND_LEFT));

      if ui.button(playpause_label).clicked() {
        match self.now_playing {
          PlayingState::Stopped => {
            self.deque_and_send_track();
          }
          PlayingState::Selected {
            ref mut playing, ..
          } => {
            *playing = !*playing;
          }
        }
      }

      ui.add_enabled(wind_enabled, Button::new(emoji::WIND_RIGHT));

      // TODO: figure out how to get the length of a sound file
      // I think it's in params.
      // https://docs.rs/symphonia-core/0.5.2/symphonia_core/units/struct.TimeBase.html
      // Not sure what to multiply there
      let pb = match self.now_playing {
        PlayingState::Selected { ref track, .. } => {
          let progress =
            track.playhead as f32 / track.file_info.num_frames as f32;
          ProgressBar::new(progress)
            .text(format!("{}/{}", track.playhead, track.file_info.num_frames))
        }
        _ => ProgressBar::new(0.0),
      };
      ui.add(pb);
    });

    ui.horizontal_centered(|ui| {
      // https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/misc_demo_window.rs#L197
      let width =
        ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
      ui.spacing_mut().item_spacing.x = width;

      match &self.now_playing {
        PlayingState::Stopped => {
          ui.label("Stopped.");
        }
        PlayingState::Selected { track, playing } => {
          ui.label("Now playing:");

          ui.label(
            RichText::new(
              track.track.path.file_name().unwrap().to_string_lossy(),
            )
            .strong(),
          );
        }
      }

      if self.buffering_cooldown > 0 {
        ui.spinner();
      }
    });
  }

  fn draw_queue(&mut self, ui: &mut eframe::egui::Ui) {
    let row_count = self.queue.len();

    ScrollArea::vertical()
      .auto_shrink([false, false]) // Add padding inside
      .show_rows(
        ui,
        ui.text_style_height(&egui::TextStyle::Body),
        row_count,
        |ui, range| {
          let end = range.end;
          for i in range {
            let track = &self.queue[i];
            ui.label(format!("{}", track.path.display()));

            if i != end - 1 {
              ui.separator();
            }
          }
        },
      );
  }
}
