use crate::components::Component;
use crate::theme::Theme;
use egui::{Color32, Frame, Layout, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};
use std::sync::Arc;

pub struct Tab<T> {
    id: String,
    titles: Vec<String>,
    contents: Vec<Box<dyn Fn(&mut Ui, &mut T, &Theme, &str)>>,
    active_tab: usize,
    theme: Arc<Theme>,
}

impl<T> Tab<T> {
    pub fn new(id: String, theme: Arc<Theme>) -> Self {
        Self {
            id,
            titles: Vec::new(),
            contents: Vec::new(),
            active_tab: 0,
            theme,
        }
    }

    pub fn add_tab<F>(&mut self, title: String, content: F)
    where
        F: Fn(&mut Ui, &mut T, &Theme, &str) + 'static,
    {
        self.titles.push(title);
        self.contents.push(Box::new(content));
    }

    pub fn render(&mut self, ui: &mut Ui, data: &mut T) {
        let theme = &self.theme;

        ui.vertical(|ui| {
            // Tab headers
            let header_height = 30.0;
            let header_spacing = 2.0;
            let total_width = ui.available_width();
            let tab_width = (total_width - (self.titles.len() as f32 - 1.0) * header_spacing)
                / self.titles.len() as f32;

            ui.horizontal(|ui| {
                for (index, title) in self.titles.iter().enumerate() {
                    let is_active = self.active_tab == index;

                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(tab_width, header_height), Sense::click());

                    if response.clicked() {
                        self.active_tab = index;
                    }

                    let text_color = if is_active {
                        theme.accent_color
                    } else {
                        theme.text_color
                    };

                    // Draw the tab background
                    if is_active {
                        let active_rect = rect.expand2(Vec2::new(0.0, 2.0));
                        ui.painter()
                            .rect_filled(active_rect, Rounding::ZERO, theme.bg_color);
                        // Draw the accent color line at the bottom of the active tab
                        ui.painter().line_segment(
                            [active_rect.left_bottom(), active_rect.right_bottom()],
                            Stroke::new(2.0, theme.accent_color),
                        );
                    } else if response.hovered() {
                        ui.painter().rect_filled(
                            rect,
                            Rounding::ZERO,
                            theme.bg_color.linear_multiply(0.97), // Slightly darker on hover
                        );
                    }

                    // Draw the text
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        title,
                        egui::TextStyle::Button.resolve(ui.style()),
                        text_color,
                    );

                    if index < self.titles.len() - 1 {
                        ui.add_space(header_spacing);
                    }
                }
            });

            // Separator line
            let separator_rect = ui.max_rect();
            ui.painter().line_segment(
                [separator_rect.left_top(), separator_rect.right_top()],
                Stroke::new(1.0, theme.separator_color),
            );

            ui.add_space(10.0);

            // Tab content
            Frame::none().fill(theme.bg_color).show(ui, |ui| {
                if let Some(content) = self.contents.get(self.active_tab) {
                    content(ui, data, theme, &self.id);
                }
            });
        });
    }

    pub fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
