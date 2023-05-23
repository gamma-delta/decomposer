// The given progress bar doesn't let you click on it
// so let's fix that.

use eframe::{
  egui::{Response, Sense, TextStyle, Ui, Widget, WidgetText},
  emath::{NumExt, Rect},
  epaint::{vec2, Color32, Rgba, Stroke},
};

pub struct TrackProgressBar {
  progress: f32,

  text: WidgetText,
}

impl TrackProgressBar {
  pub fn new(progress: f32, text: impl Into<WidgetText>) -> Self {
    Self {
      progress,
      text: text.into(),
    }
  }
}

impl Widget for TrackProgressBar {
  fn ui(self, ui: &mut Ui) -> Response {
    let TrackProgressBar { progress, text } = self;

    let desired_width = ui.available_size_before_wrap().x.at_least(96.0);
    let height = ui.spacing().interact_size.y;
    // This is the critical change, make the sense right
    let (outer_rect, response) = ui.allocate_exact_size(
      vec2(desired_width, height),
      Sense::click_and_drag(),
    );

    if ui.is_rect_visible(response.rect) {
      let visuals = ui.style().visuals.clone();
      let rounding = outer_rect.height() / 2.0;
      ui.painter().rect(
        outer_rect,
        rounding,
        visuals.extreme_bg_color,
        Stroke::NONE,
      );
      let inner_rect = Rect::from_min_size(
        outer_rect.min,
        vec2(
          (outer_rect.width() * progress).at_least(outer_rect.height()),
          outer_rect.height(),
        ),
      );

      ui.painter().rect(
        inner_rect,
        rounding,
        Color32::from(visuals.selection.bg_fill),
        Stroke::NONE,
      );

      let galley =
        text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
      let text_pos = outer_rect.left_center()
        - vec2(0.0, galley.size().y / 2.0)
        + vec2(ui.spacing().item_spacing.x, 0.0);
      let text_color = visuals
        .override_text_color
        .unwrap_or(visuals.selection.stroke.color);
      galley.paint_with_fallback_color(
        &ui.painter().with_clip_rect(outer_rect),
        text_pos,
        text_color,
      );
    } // ^ is_rect_visible

    response
  }
}
