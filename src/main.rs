use animamac::character_lib::CharacterLibrary;
use animamac::logging::log_to_file;
use animamac::settings::AppSettings;
#[cfg(feature = "steamcmd")]
use animamac::steamlib::{extract_workshop_id, get_ws, workshop_dl, DownloadResult};
use eframe::egui::{self, Color32, Frame, ImageSource};
#[cfg(feature = "lite")]
use rfd::FileDialog;
fn main() -> eframe::Result<()> {
    log_to_file("AnimaMac starting...");
    let options = eframe::NativeOptions {
        vsync: false,
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_inner_size([400.0, 520.0])
            .with_movable_by_background(true)
            .with_window_level(egui::WindowLevel::AlwaysOnTop)
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

struct AtApp<'a> {
    uimode: bool,
    id: String,
    path: ImageSource<'a>,
    current_image_path: Option<String>,
    ws: String,
    speed: i64,
    scale: f32,
    library: CharacterLibrary,
    settings: AppSettings,
    selected_character: Option<usize>,
    #[cfg(feature = "steamcmd")]
    download_result: Option<DownloadResult>,
    #[cfg(feature = "steamcmd")]
    add_to_library: bool,
    #[cfg(feature = "steamcmd")]
    character_name: String,
}

impl Default for AtApp<'_> {
    fn default() -> Self {
        let settings = AppSettings::load();

        let path = if let Some(ref img_path) = settings.current_image_path {
            if std::path::Path::new(img_path).exists() {
                ImageSource::Uri(std::borrow::Cow::Owned(format!("file://{}", img_path)))
            } else {
                egui::include_image!("../assets/workplease.webp")
            }
        } else {
            egui::include_image!("../assets/workplease.webp")
        };

        Self {
            uimode: false,
            id: "".to_owned(),
            path,
            current_image_path: settings.current_image_path.clone(),
            ws: "".to_owned(),
            speed: settings.speed,
            scale: settings.image_scale,
            library: CharacterLibrary::load(),
            settings,
            selected_character: None,
            #[cfg(feature = "steamcmd")]
            download_result: None,
            #[cfg(feature = "steamcmd")]
            add_to_library: false,
            #[cfg(feature = "steamcmd")]
            character_name: String::new(),
        }
    }
}

impl eframe::App for AtApp<'_> {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.settings.current_image_path = self.current_image_path.clone();
        self.settings.speed = self.speed;
        self.settings.image_scale = self.scale;
        self.settings.save();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                self.show_dialog(ui);

                if self.uimode {
                    let settings_frame = egui::Frame::default()
                        .fill(Color32::from_rgba_premultiplied(30, 27, 25, 255));
                    settings_frame.show(ui, |ui| {
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
                                    }
                                    None => {}
                                };
                            }
                        }

                        let char_count = self.library.characters.len();
                        if char_count > 0 {
                            ui.horizontal_wrapped(|ui| {
                                for (i, char) in self.library.characters.iter().enumerate() {
                                    let btn_text = if self.selected_character == Some(i) {
                                        format!("✓ {}", char.name)
                                    } else {
                                        char.name.clone()
                                    };
                                    if ui.add(egui::Button::new(btn_text).small()).clicked() {
                                        self.selected_character = Some(i);
                                        self.current_image_path = Some(char.path.clone());
                                        self.path = ImageSource::Uri(std::borrow::Cow::Owned(
                                            format!("file://{}", char.path),
                                        ));
                                        self.settings.current_image_path = Some(char.path.clone());
                                        self.settings.save();
                                        println!("Selected: {}", char.path);
                                    }
                                }
                            });

                            if ui
                                .add(egui::Button::new("Remove Selected").small())
                                .clicked()
                            {
                                if let Some(idx) = self.selected_character {
                                    self.library.remove_character(idx);
                                    self.selected_character = None;
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
                                        self.current_image_path = Some(final_path.clone());
                                        self.path = ImageSource::Uri(std::borrow::Cow::Owned(
                                            format!("file://{}", final_path),
                                        ));
                                        self.settings.current_image_path = Some(final_path);
                                        self.settings.save();
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
                        ui.label("Framerate (0 for default)");
                        let slider = ui.add(egui::Slider::new(&mut self.speed, 0..=240));
                        if slider.changed() {
                            self.settings.speed = self.speed;
                            self.settings.save();
                        }
                        
                        ui.separator();
                        ui.label("Image Scale");
                        ui.horizontal(|ui| {
                            let mut scale_value = self.scale;
                            let slider = ui.add(egui::Slider::new(&mut scale_value, 0.1f32..=5.0f32).step_by(0.1));
                            if slider.changed() {
                                self.scale = scale_value;
                                self.settings.image_scale = scale_value;
                                self.settings.save();
                            }
                        });
                        
                        ui.separator();
                        if ui.add(egui::Button::new("Exit")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }

                let mut img = egui::Image::new(self.path.clone());
                img.fps = u128::try_from(self.speed).unwrap();
                let scaled_size = egui::vec2(320.0 * self.scale, 320.0 * self.scale);
                img = img.max_size(scaled_size);
                ui.add(img);
                if ui.input(|i| i.key_released(egui::Key::Escape)) {
                    if self.uimode == false {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                        self.uimode = true;
                    } else {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        self.uimode = false;
                    }
                }
            });
    }
}

#[cfg(feature = "steamcmd")]
impl AtApp<'_> {
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
                                if self.add_to_library && !self.character_name.is_empty() {
                                    if let Some(first_file) = result_files.first() {
                                        let full_path = format!("{}/{}", result_path, first_file);
                                        self.library
                                            .add_named_character(&full_path, &self.character_name);
                                    }
                                }
                                if let Some(first_file) = result_files.first() {
                                    let full_path =
                                        format!("file://{}/{}", result_path, first_file);
                                    self.path =
                                        ImageSource::Uri(std::borrow::Cow::Owned(full_path));
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
