mod clickable_progress_bar;

use crate::{
  app::DecomposerApp,
  emoji,
  model::{MsgUiToThread, PlayingState},
  util,
};

use eframe::{
  egui::{
    self, Button, CentralPanel, ImageButton, Label, Layout, ProgressBar,
    RichText, ScrollArea, Slider, TextStyle, TopBottomPanel, Visuals,
    WidgetText,
  },
  emath::Align,
  epaint::{vec2, Pos2},
};

use self::clickable_progress_bar::TrackProgressBar;

impl DecomposerApp {
  /// Pull this function out into its own file because i like doing that
  pub fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // clone out of the arc
    let style = (*ctx.style()).clone();
    ctx.set_visuals(Visuals {
      slider_trailing_fill: true,
      ..style.visuals
    });

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

    ui.label(concat!("Decomposer v", env!("CARGO_PKG_VERSION")));
  }

  // In a vert layout
  fn draw_bottom_bar(&mut self, ui: &mut eframe::egui::Ui) {
    ui.add_space(ui.spacing().item_spacing.y * 2.0);

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

        if let PlayingState::Selected { ref track, .. } = self.now_playing {
          let progress =
            track.playhead as f32 / track.file_info.num_frames as f32;

          let text = if let Some(timesize) = track.file_info.params.time_base {
            let here = timesize.calc_time(track.playhead as u64);
            let end = timesize.calc_time(track.file_info.num_frames as u64);

            format!(
              "{}/{}",
              util::format_symphonia_time(here),
              util::format_symphonia_time(end),
            )
          } else {
            format!("xx:xx/xx:xx")
          };

          let res = ui.add(TrackProgressBar::new(progress, text));
          // we have chained if-let at home
          if let (Some(mousepos), Some(timesize), true) = (
            ui.ctx().pointer_latest_pos(),
            track.file_info.params.time_base,
            res.hovered(),
          ) {
            let bar_span = res.rect;
            let x_prop = (mousepos.x - bar_span.left()) / bar_span.width();

            let frames_in =
              (x_prop * track.file_info.num_frames as f32) as usize;
            let mouse_time = timesize.calc_time(frames_in as u64);
            let hover = util::format_symphonia_time(mouse_time);

            // TODO: figure out how to actually put the tooltip centered above
            // the cursor without this bullshit
            let tooltip_pos = Pos2::new(mousepos.x, bar_span.top() - 32.0);
            egui::containers::show_tooltip_at(
              ui.ctx(),
              res.id.with("__mouse_time"),
              Some(tooltip_pos),
              |ui| {
                ui.label(hover);
              },
            );

            if res.clicked() {
              let _ignore =
                self.tx_to_thread.push(MsgUiToThread::SeekTo(frames_in));
            }
          }
        } else {
          ui.add(ProgressBar::new(0.0));
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
            // Do striping manually
            if i % 2 == 0 {
              let col = ui.style().visuals.faint_bg_color;
              ui.style_mut().visuals.panel_fill = col;
            }
            ui.label(format!("{}", track.path.display()));

            if i != end - 1 {
              ui.separator();
            }
          }
        },
      );
  }
}
