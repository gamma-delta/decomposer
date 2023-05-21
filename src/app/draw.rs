use crate::{
  app::DecomposerApp,
  emoji,
  model::{MsgUiToThread, PlayingState},
};

use eframe::{
  egui::{
    self, Button, CentralPanel, ImageButton, Label, Layout, ProgressBar,
    RichText, ScrollArea, Slider, TextStyle, TopBottomPanel, WidgetText,
  },
  emath::Align,
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
            let msg = if *playing {
              MsgUiToThread::Resume
            } else {
              MsgUiToThread::Pause
            };
            let _ignore = self.tx_to_thread.push(msg);
          }
        }
      }

      ui.add_enabled(wind_enabled, Button::new(emoji::WIND_RIGHT));

      // We want the progress bar to just take whatever's remaining in the center
      // so lay out right to left.
      ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let old_volume = *self.config.volume();

        ui.add(
          Slider::new(self.config.volume(), 0.0..=2.0)
            .custom_formatter(|f, _| format!("{:.0}%", f * 100.0)),
        );
        if old_volume != *self.config.volume() {
          let _ignore = self
            .tx_to_thread
            .push(MsgUiToThread::SetVolume(*self.config.volume()));
        }

        ui.separator();

        let (pb, hover) = match self.now_playing {
          PlayingState::Selected { ref track, .. } => {
            let progress =
              track.playhead as f32 / track.file_info.num_frames as f32;

            let text = if let Some(timesize) = track.file_info.params.time_base
            {
              let here = timesize.calc_time(track.playhead as u64);
              let end = timesize.calc_time(track.file_info.num_frames as u64);

              format!(
                "{}:{:02}/{}:{:02}",
                here.seconds / 60,
                here.seconds % 60,
                end.seconds / 60,
                end.seconds % 60
              )
            } else {
              format!("xx:xx/xx:xx")
            };

            let hover = format!(
              "{}/{} frames",
              track.playhead, track.file_info.num_frames
            );

            (ProgressBar::new(progress).text(text), Some(hover))
          }
          _ => (ProgressBar::new(0.0), None),
        };
        let res = ui.add(pb);
        if let Some(hover) = hover {
          res.on_hover_text(&hover);
        }
      });
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
