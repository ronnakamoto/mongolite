use crate::components::{Component, ThemedButton};
use crate::models::{ConnectionProfile, ConnectionProfileManager};
use crate::theme::Theme;
use egui::{Align, Context, Layout, RichText, Ui, Widget, Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ConnectionManager {
    connection_string: String,
    profile_manager: Rc<RefCell<ConnectionProfileManager>>,
    theme: Arc<Theme>,
    show_dialog: bool,
    new_profile: ConnectionProfile,
    edit_mode: bool,
    selected_profile: Option<ConnectionProfile>,
    delete_confirmation: Option<String>, // Stores the ID of the profile to be deleted
}

impl ConnectionManager {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            connection_string: String::new(),
            profile_manager: ConnectionProfileManager::new(),
            theme,
            show_dialog: false,
            new_profile: ConnectionProfile {
                id: String::new(),
                name: String::new(),
                connection_string: String::new(),
            },
            edit_mode: false,
            selected_profile: None,
            delete_confirmation: None,
        }
    }

    fn render_dialog_content(&mut self, ui: &mut Ui) {
        let profiles = self.profile_manager.borrow().get_profiles().to_vec();

        ui.heading(RichText::new("Manage Connection Profiles").color(self.theme.text_color));
        ui.add_space(10.0);

        if profiles.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    RichText::new("No profiles exist. Please create a connection profile.")
                        .color(self.theme.text_color)
                        .size(16.0),
                );
                ui.add_space(20.0);
            });
        } else {
            // Render profile list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for profile in profiles.iter() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&profile.name).color(self.theme.text_color));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ThemedButton::new("Delete", Arc::clone(&self.theme))
                                .ui(ui)
                                .clicked()
                            {
                                self.delete_confirmation = Some(profile.id.clone());
                            }
                            if ThemedButton::new("Edit", Arc::clone(&self.theme))
                                .ui(ui)
                                .clicked()
                            {
                                self.new_profile = profile.clone();
                                self.edit_mode = true;
                            }
                            if ThemedButton::new("Select", Arc::clone(&self.theme))
                                .ui(ui)
                                .clicked()
                            {
                                self.selected_profile = Some(profile.clone());
                                self.connection_string = profile.connection_string.clone();
                            }
                        });
                    });
                    ui.add_space(5.0);
                }
            });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Render add/edit form
        ui.horizontal(|ui| {
            ui.add_sized(
                [100.0, 20.0],
                egui::Label::new(RichText::new("Name:").color(self.theme.text_color)),
            );
            ui.add_sized(
                [200.0, 20.0],
                egui::TextEdit::singleline(&mut self.new_profile.name),
            );
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [100.0, 20.0],
                egui::Label::new(RichText::new("Connection String:").color(self.theme.text_color)),
            );
            ui.add_sized(
                [200.0, 20.0],
                egui::TextEdit::singleline(&mut self.new_profile.connection_string),
            );
        });

        ui.add_space(10.0);

        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            if self.edit_mode {
                if ThemedButton::new("Update Profile", Arc::clone(&self.theme))
                    .ui(ui)
                    .clicked()
                {
                    self.profile_manager
                        .borrow_mut()
                        .save_profile(&self.new_profile);
                    self.edit_mode = false;
                    self.new_profile = ConnectionProfile {
                        id: String::new(),
                        name: String::new(),
                        connection_string: String::new(),
                    };
                }
            } else {
                if ThemedButton::new("Add New Profile", Arc::clone(&self.theme))
                    .ui(ui)
                    .clicked()
                {
                    self.new_profile.id = Uuid::new_v4().to_string();
                    self.profile_manager
                        .borrow_mut()
                        .save_profile(&self.new_profile);
                    self.new_profile = ConnectionProfile {
                        id: String::new(),
                        name: String::new(),
                        connection_string: String::new(),
                    };
                }
            }
            if ThemedButton::new("Cancel", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                self.edit_mode = false;
                self.new_profile = ConnectionProfile {
                    id: String::new(),
                    name: String::new(),
                    connection_string: String::new(),
                };
            }
        });
    }

    fn show_delete_confirmation(&mut self, ctx: &Context) {
        let mut delete_confirmed = false;
        let mut cancel_confirmed = false;

        if let Some(profile_id) = &self.delete_confirmation {
            let profile_name = self
                .profile_manager
                .borrow()
                .get_profiles()
                .iter()
                .find(|p| p.id == *profile_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            Window::new("Confirm Deletion")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Are you sure you want to delete the profile '{}'?",
                            profile_name
                        ))
                        .color(self.theme.text_color),
                    );
                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if ThemedButton::new("Yes", Arc::clone(&self.theme))
                            .ui(ui)
                            .clicked()
                        {
                            delete_confirmed = true;
                        }
                        if ThemedButton::new("No", Arc::clone(&self.theme))
                            .ui(ui)
                            .clicked()
                        {
                            cancel_confirmed = true;
                        }
                    });
                });
        }

        // Handle the confirmation outside of the closure
        if delete_confirmed {
            if let Some(profile_id) = &self.delete_confirmation {
                self.profile_manager.borrow_mut().delete_profile(profile_id);
            }
            self.delete_confirmation = None;
        } else if cancel_confirmed {
            self.delete_confirmation = None;
        }
    }

    pub fn show(&mut self, ctx: &Context) {
        let mut show_dialog = self.show_dialog;
        Window::new("Connection Profiles")
            .open(&mut show_dialog)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                self.render_dialog_content(ui);
            });
        self.show_dialog = show_dialog;

        self.show_delete_confirmation(ctx);
    }
}

impl Component for ConnectionManager {
    fn render(&mut self, ui: &mut Ui, _id_prefix: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Connection:")
                    .color(self.theme.text_color)
                    .strong(),
            )
            .on_hover_text("Select or manage database connections");

            ui.text_edit_singleline(&mut self.connection_string);

            if ThemedButton::new("Connect", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                // Handle connection logic
            }

            if ThemedButton::new("Manage Profiles", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                self.show_dialog = true;
            }
        });

        self.show(ui.ctx());
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
