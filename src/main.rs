use animamac::character_lib::CharacterLibrary;
use animamac::logging::log_to_file;
use animamac::settings::AppSettings;
#[cfg(feature = "steamcmd")]
use animamac::steamlib::{extract_workshop_id, get_ws, workshop_dl, DownloadResult};
use eframe::egui::{self, Color32, Frame, ImageSource};
use std::collections::HashMap;
#[cfg(feature = "lite")]
use rfd::FileDialog;
fn main() -> eframe::Result<()> {
    log_to_file("AnimaMac starting...");
    let options = eframe::NativeOptions {
        vsync: false,
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_inner_size([400.0, 520.0])
            .with_movable_by_background(true)
            .with_window_level(egui::WindowLevel::Normal)
            .with_mouse_passthrough(false),
        ..Default::default()
    };

    eframe::run_native(
        "AnimaMac",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = AtApp::default();
            #[cfg(feature = "steamcmd")]
            {
                app.ws = get_ws();
            }
            Ok(Box::new(app))
        }),
    )
}

struct CharacterUiState {
    show_settings: bool,
}

struct AtApp {
    id: String,
    ws: String,
    library: CharacterLibrary,
    settings: AppSettings,
    main_visible: bool,
    active_character: Option<String>,
    allow_main_close: bool,
    character_ui: HashMap<String, CharacterUiState>,
    #[cfg(feature = "steamcmd")]
    download_result: Option<DownloadResult>,
    #[cfg(feature = "steamcmd")]
    add_to_library: bool,
    #[cfg(feature = "steamcmd")]
    character_name: String,
}

impl Default for AtApp {
    fn default() -> Self {
        let settings = AppSettings::load();

        Self {
            id: "".to_owned(),
            ws: "".to_owned(),
            library: CharacterLibrary::load(),
            settings,
            main_visible: true,
            active_character: None,
            allow_main_close: false,
            character_ui: HashMap::new(),
            #[cfg(feature = "steamcmd")]
            download_result: None,
            #[cfg(feature = "steamcmd")]
            add_to_library: false,
            #[cfg(feature = "steamcmd")]
            character_name: String::new(),
        }
    }
}

impl eframe::App for AtApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.settings.save();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.allow_main_close {
                // Allow app to quit.
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.main_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        if let Some(active_path) = self.active_character.clone() {
            let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            if escape {
                let state = self
                    .character_ui
                    .entry(active_path.clone())
                    .or_insert(CharacterUiState { show_settings: false });
                state.show_settings = !state.show_settings;
                let vp_id =
                    egui::ViewportId::from_hash_of(format!("character:{}", active_path));
                ctx.request_repaint_of(vp_id);
            }

            let cmd_m =
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::M));
            if cmd_m {
                self.main_visible = !self.main_visible;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.main_visible));
                if self.main_visible {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
            }
        }

        if self.main_visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        let main_frame =
            egui::Frame::default().fill(Color32::from_rgba_premultiplied(30, 27, 25, 255));
        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui| {
                self.show_dialog(ui);
                ui.heading("AnimaMac Settings");

                ui.separator();
                ui.heading("My Characters");

                #[cfg(feature = "lite")]
                {
                    if ui.add(egui::Button::new("Add from File")).clicked() {
                        let _ = match FileDialog::new()
                            .add_filter("Animation/Image", &["png", "apng", "webp", "gif"])
                            .pick_file()
                        {
                            Some(path) => {
                                let path_str = path.to_str().unwrap();
                                println!("Added: {}", path_str);
                                self.library.add_character(path_str);
                                if let Some(idx) = self.library.index_by_path(path_str) {
                                    self.library.set_enabled(idx, true);
                                    self.character_ui
                                        .entry(path_str.to_string())
                                        .or_insert(CharacterUiState { show_settings: false });
                                    self.active_character = Some(path_str.to_string());
                                }
                            }
                            None => {}
                        };
                    }
                }

                let char_count = self.library.characters.len();
                if char_count > 0 {
                    let mut toggle_index: Option<usize> = None;
                    let mut remove_index: Option<usize> = None;
                    ui.vertical(|ui| {
                        for (i, char) in self.library.characters.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let btn_text = if char.enabled {
                                    format!("✓ {}", char.name)
                                } else {
                                    char.name.clone()
                                };
                                if ui.add(egui::Button::new(btn_text)).clicked() {
                                    toggle_index = Some(i);
                                }
                                let remove_btn = egui::Button::new("×")
                                    .small()
                                    .min_size(egui::vec2(18.0, 18.0));
                                if ui.add(remove_btn).on_hover_text("Remove").clicked() {
                                    remove_index = Some(i);
                                }
                            });
                        }
                    });

                    if let Some(i) = toggle_index {
                        let enabled = !self.library.characters[i].enabled;
                        self.library.set_enabled(i, enabled);
                        let path = self.library.characters[i].path.clone();
                        if enabled {
                            self.character_ui
                                .entry(path)
                                .or_insert(CharacterUiState { show_settings: false });
                        } else {
                            self.character_ui.remove(&path);
                            if self.active_character.as_deref() == Some(&path) {
                                self.active_character = None;
                            }
                        }
                    }

                    if let Some(i) = remove_index {
                        let path = self.library.characters[i].path.clone();
                        self.library.remove_character(i);
                        self.character_ui.remove(&path);
                        if self.active_character.as_deref() == Some(&path) {
                            self.active_character = None;
                        }
                    }
                } else {
                    ui.label("No characters yet. Add some!");
                }

                ui.separator();

                #[cfg(feature = "lite")]
                {
                    if ui
                        .add(egui::Button::new("Quick Select Animation/Image"))
                        .clicked()
                    {
                        let _ = match FileDialog::new()
                            .add_filter("Animation/Image", &["png", "apng", "webp", "gif"])
                            .pick_file()
                        {
                            Some(path) => {
                                let path_str = path.to_str().unwrap();
                                let path_lower = path_str.to_lowercase();
                                let final_path: String;

                                if path_lower.ends_with(".apng") {
                                    final_path = match animamac::character_lib::CharacterLibrary::convert_apng_to_webp(path_str) {
                                        Some(converted) => converted,
                                        None => path_str.to_string(),
                                    };
                                } else {
                                    final_path = path_str.to_string();
                                }

                                println!("Selected: {}", final_path);
                                self.library.add_character(&final_path);
                                    if let Some(idx) = self.library.index_by_path(&final_path) {
                                        self.library.set_enabled(idx, true);
                                    self.character_ui
                                        .entry(final_path.clone())
                                        .or_insert(CharacterUiState { show_settings: false });
                                    self.active_character = Some(final_path);
                                    }
                            }
                            None => {}
                        };
                    }
                }

                #[cfg(feature = "steamcmd")]
                {
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        ui.heading("Download from Workshop");
                        let id = ui.add(egui::TextEdit::singleline(&mut self.id));
                        if id.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            let extracted_id = extract_workshop_id(&self.id);
                            self.download_result = workshop_dl(&extracted_id, &self.ws);
                            if self.download_result.is_none() {
                                log_to_file("steamlib: workshop download failed; see earlier logs");
                            }
                        }
                    });
                }

                ui.separator();
                if ui.add(egui::Button::new("Exit")).clicked() {
                    self.allow_main_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

        let monitor_size = ctx.input(|i| i.viewport().monitor_size);
        let characters_snapshot: Vec<(usize, String, String, i64, f32, Option<[f32; 2]>)> = self
            .library
            .characters
            .iter()
            .enumerate()
            .filter(|(_, c)| c.enabled)
            .map(|(i, c)| {
                (
                    i,
                    c.name.clone(),
                    c.path.clone(),
                    c.speed,
                    c.scale,
                    c.window_pos,
                )
            })
            .collect();

        for (index, name, path, speed, scale, window_pos) in characters_snapshot {
            let viewport_id = egui::ViewportId::from_hash_of(format!("character:{}", path));
            let mut builder = egui::ViewportBuilder::default()
                .with_title(name.clone())
                .with_decorations(false)
                .with_transparent(true)
                .with_has_shadow(false)
                .with_movable_by_background(true)
                .with_window_level(egui::WindowLevel::AlwaysOnTop)
                .with_mouse_passthrough(false);

            let scaled_size = egui::vec2(320.0 * scale, 320.0 * scale);
            builder = builder.with_inner_size([scaled_size.x, scaled_size.y]);

            if let Some(pos) = window_pos {
                builder = builder.with_position([pos[0], pos[1]]);
            } else if let Some(monitor_size) = monitor_size {
                let center_pos = egui::pos2(
                    (monitor_size.x - scaled_size.x) * 0.5,
                    (monitor_size.y - scaled_size.y) * 0.5,
                );
                builder = builder.with_position(center_pos);
            }

            let mut show_settings = self
                .character_ui
                .get(&path)
                .map(|state| state.show_settings)
                .unwrap_or(false);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.library.set_enabled(index, false);
                    self.character_ui.remove(&path);
                    return;
                }

                let focused = ctx.input(|i| i.viewport().focused.unwrap_or(false));
                if ctx.input(|i| i.pointer.any_pressed()) && !focused {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                if focused && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                    show_settings = !show_settings;
                    if let Some(state) = self.character_ui.get_mut(&path) {
                        state.show_settings = show_settings;
                    }
                    ctx.request_repaint();
                }

                if focused
                    && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::M))
                {
                    self.main_visible = !self.main_visible;
                    ctx.send_viewport_cmd_to(
                        egui::ViewportId::ROOT,
                        egui::ViewportCommand::Visible(self.main_visible),
                    );
                    if self.main_visible {
                        ctx.send_viewport_cmd_to(
                            egui::ViewportId::ROOT,
                            egui::ViewportCommand::Focus,
                        );
                    }
                }

                ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));

                egui::CentralPanel::default()
                    .frame(Frame::NONE)
                    .show(ctx, |ui| {
                        let capture_id =
                            egui::Id::new(format!("focus-capture:{}", path));
                        let capture =
                            ui.interact(ui.max_rect(), capture_id, egui::Sense::click());
                        if capture.clicked() {
                            self.active_character = Some(path.clone());
                            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        }
                        let mut img = egui::Image::new(ImageSource::Uri(
                            std::borrow::Cow::Owned(format!("file://{}", path)),
                        ));
                        img.fps = u128::try_from(speed).unwrap_or(0);
                        img = img.max_size(scaled_size);
                        ui.add(img);
                    });

                if show_settings {
                    let settings_frame = egui::Frame::default()
                        .fill(Color32::from_rgba_premultiplied(30, 27, 25, 240));
                    egui::Area::new(egui::Id::new(format!("settings:{}", path)))
                        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
                        .show(ctx, |ui| {
                            settings_frame.show(ui, |ui| {
                                ui.heading(name.clone());
                                ui.separator();
                                let mut new_speed = speed;
                                ui.label("Framerate (0 for default)");
                                if ui
                                    .add(egui::Slider::new(&mut new_speed, 0..=240))
                                    .changed()
                                {
                                    self.library.update_settings(index, new_speed, scale);
                                }

                                ui.separator();
                                let mut new_scale = scale;
                                ui.label("Image Scale");
                                if ui
                                    .add(
                                        egui::Slider::new(&mut new_scale, 0.1f32..=5.0f32)
                                            .step_by(0.1),
                                    )
                                    .changed()
                                {
                                    self.library.update_settings(index, speed, new_scale);
                                }
                            });
                        });
                }

                if let Some(rect) = ctx.input(|i| i.viewport().outer_rect) {
                    let pos = [rect.min.x, rect.min.y];
                    if window_pos.map(|p| p != pos).unwrap_or(true) {
                        self.library.update_position(index, pos);
                    }
                }
            });
        }
    }
}

#[cfg(feature = "steamcmd")]
impl AtApp {
    fn show_dialog(&mut self, ui: &mut egui::Ui) {
        if let Some(ref download_result) = self.download_result {
            let result_path = download_result.path.clone();
            let result_files = download_result.files.clone();
            let dialog_frame =
                egui::Frame::default().fill(Color32::from_rgba_premultiplied(30, 27, 25, 255));
            egui::Area::new("download_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    dialog_frame.show(ui, |ui| {
                        ui.heading("Download Complete");
                        ui.separator();
                        ui.label(format!("Downloaded to: {}", result_path));
                        ui.label("Files:");
                        for file in &result_files {
                            ui.label(format!("  - {}", file));
                        }
                        ui.separator();

                        ui.checkbox(&mut self.add_to_library, "Add to Characters library");

                        if self.add_to_library {
                            ui.label("Character name:");
                            ui.text_edit_singleline(&mut self.character_name);
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Add to Library & Use").clicked() {
                                if let Some(first_file) = result_files.first() {
                                    let full_path = format!("{}/{}", result_path, first_file);
                                    if self.add_to_library && !self.character_name.is_empty() {
                                        self.library
                                            .add_named_character(&full_path, &self.character_name);
                                    } else {
                                        self.library.add_character(&full_path);
                                    }
                                    if let Some(idx) = self.library.index_by_path(&full_path) {
                                        self.library.set_enabled(idx, true);
                                        self.character_ui
                                            .entry(full_path.clone())
                                            .or_insert(CharacterUiState { show_settings: false });
                                        self.active_character = Some(full_path);
                                    }
                                }
                                self.download_result = None;
                                self.character_name.clear();
                                self.add_to_library = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.download_result = None;
                                self.character_name.clear();
                                self.add_to_library = false;
                            }
                        });
                    });
                });
        }
    }
}
