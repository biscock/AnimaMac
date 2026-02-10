use eframe::egui::{self, Color32, Frame, ImageSource};
#[cfg(feature = "steamcmd")]
use animatux::steamlib::{workshop_dl, list_ws, get_ws};
#[cfg(feature = "lite")]
use rfd::FileDialog;
fn main() -> eframe::Result<()>{
    println!("AnimaTux");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_decorations(false).with_transparent(true).with_has_shadow(false).with_inner_size([320.0,320.0]).with_movable_by_background(true).with_window_level(egui::WindowLevel::AlwaysOnTop),
        ..Default::default()
    };

    
    eframe::run_native("AnimaTux", options, Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = AtApp::default();
        #[cfg(feature = "steamcmd")]{
            app.ws = get_ws();
        }
        Ok(Box::new(app))
    }))
}

struct AtApp<'a> {
    uimode: bool,
    id: String,
    path: ImageSource<'a>,
    ids: Vec<String>,
    ws: String
}
impl Default for AtApp<'_> {
    fn default() -> Self {
        Self { uimode: false, id: "".to_owned(), ids: vec![], path: egui::include_image!("../assets/workplease.webp"), ws: "".to_owned()}
    }
}
impl eframe::App for AtApp<'_> {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] 
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            if self.uimode{
                #[cfg(feature = "lite")]{
                    // Just gives a button for a file dialog, smaller unintended use
                    if ui.add(egui::Button::new("Select Animation/Image")).clicked(){
                        let _ = match FileDialog::new()
                                                    .add_filter("Image", &["png", "webp"])
                                                    .add_filter("Animation", &["webp", "gif"])
                                                    //.set_directory("~")
                                                    .pick_file() {
                            Some(path) => {
                                println!("{}", path.to_str().unwrap());
                                self.path = ImageSource::Uri(std::borrow::Cow::Owned(format!("file://{}", path.to_str().unwrap()).to_string()));
                            },
                            None => {},
                        };
                        
                    }
                }
                
                
                #[cfg(feature = "steamcmd")]{
                        let settings_frame = egui::Frame::default().fill(Color32::from_rgba_premultiplied(30, 27, 25, 255));
                        settings_frame.show(ui, |ui|{
                        ui.heading("Download from Workshop");
                        let id = ui.add(egui::TextEdit::singleline(&mut self.id));
                        if id.lost_focus() && ui.input(|i|i.key_pressed(egui::Key::Enter)){
                            //Attempt download
                            workshop_dl(&self.id, &self.ws);
                        }
                        // WorkShop buttons (WIP)
                        
                        
                            ui.horizontal_wrapped(|ui|{
                                for e in &self.ids{
                                    if ui.add(egui::Button::new(e)).clicked(){
                                        let path = format!(
                                            "file:///home/seth/.local/share/Steam/steamapps/workshop/content/{}/{}",
                                            include_str!("../assets/appid"),
                                            e
                                        );
                                        println!("{}",path);
                                        self.path = ImageSource::Uri(std::borrow::Cow::Owned(path));
                                    }
                                }
                            });
                        });
                        
                        
                    }
            }
            ui.image(self.path.clone());
            if ui.input(|i| i.key_released(egui::Key::Escape)) {
                if self.uimode == false{
                    // Enable UI
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                    self.uimode = true;

                    //Steam Integration
                    #[cfg(feature = "steamcmd")]{
                        self.ids = list_ws(&self.ws);
                    }
                    
                    
                }else{
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                    self.uimode = false;
                }
            }
        });
    }
}