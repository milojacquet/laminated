use crate::egui::*;

#[derive(PartialEq)]
pub enum KeyLabelStatus {
    Unpressed,
    Pressed,
    Clicked,
}

/// https://docs.rs/egui/latest/src/egui/widgets/selected_label.rs.html
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct KeyLabel {
    selected: KeyLabelStatus,
    text: WidgetText,
}

impl KeyLabel {
    pub fn new(selected: KeyLabelStatus, text: impl Into<WidgetText>) -> Self {
        Self {
            selected,
            text: text.into(),
        }
    }
}

impl Widget for KeyLabel {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { selected, text } = self;

        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + button_padding;

        let wrap_width = ui.available_width() - total_extra.x;
        let text = text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = total_extra + text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        /*response.widget_info(|| {
            WidgetInfo::selected(WidgetType::SelectableLabel, selected, text.text())
        });*/

        if ui.is_rect_visible(response.rect) {
            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                .min;

            let visuals = match selected {
                KeyLabelStatus::Unpressed | KeyLabelStatus::Pressed => ui
                    .style()
                    .interact_selectable(&response, selected == KeyLabelStatus::Pressed),
                KeyLabelStatus::Clicked => style::WidgetVisuals {
                    // copied from somewhere and changed to green
                    bg_fill: Color32::from_rgb(0, 128, 64),
                    weak_bg_fill: Color32::from_rgb(0, 128, 64),
                    bg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgb(60, 60, 60),
                    },
                    rounding: Rounding {
                        nw: 2.0,
                        ne: 2.0,
                        sw: 2.0,
                        se: 2.0,
                    },
                    fg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgb(192, 222, 255),
                    },
                    expansion: 0.0,
                },
            };

            if selected != KeyLabelStatus::Unpressed
                || response.hovered()
                || response.highlighted()
                || response.has_focus()
            {
                let rect = rect.expand(visuals.expansion);

                ui.painter().rect(
                    rect,
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                );
            }

            text.paint_with_visuals(ui.painter(), text_pos, &visuals);
        }

        response
    }
}
