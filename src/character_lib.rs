use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub speed: i64,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub window_pos: Option<[f32; 2]>,
    #[serde(default)]
    pub window_size: Option<[f32; 2]>,
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterLibrary {
    pub characters: Vec<Character>,
}

impl CharacterLibrary {
    pub fn load() -> Self {
        let path = Self::get_library_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::get_library_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        let _ = fs::write(&path, content);
    }

    pub fn convert_apng_to_webp(input_path: &str) -> Option<String> {
        let path_buf = PathBuf::from(input_path);
        let Some(parent) = path_buf.parent() else {
            return None;
        };
        let stem = path_buf.file_stem()?.to_str()?;
        let webp_name = format!("{}.webp", stem);
        let webp_path = parent.join(&webp_name);
        let webp_path_str = webp_path.to_str()?;

        println!(
            "Converting APNG to WebP: {} -> {}",
            input_path, webp_path_str
        );

        match Self::convert_apng_to_webp_internal(input_path, webp_path_str) {
            Ok(_) => {
                println!("Conversion successful!");
                Some(webp_path_str.to_string())
            }
            Err(e) => {
                println!("Conversion failed: {}", e);
                None
            }
        }
    }

    fn find_command(name: &str) -> Option<String> {
        let paths = [
            format!("/opt/homebrew/bin/{}", name),
            format!("/usr/local/bin/{}", name),
            format!("/usr/bin/{}", name),
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                return Some(path.clone());
            }
        }

        if let Ok(output) = std::process::Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn convert_apng_to_webp_internal(input_path: &str, output_path: &str) -> Result<(), String> {
        use std::fs;
        use std::process::Command;
        use tempfile::TempDir;

        println!("Converting APNG to animated WebP...");

        let ffmpeg_path = Self::find_command("ffmpeg").unwrap_or_else(|| "ffmpeg".to_string());
        let img2webp_path =
            Self::find_command("img2webp").unwrap_or_else(|| "img2webp".to_string());

        println!("Using ffmpeg: {}", ffmpeg_path);
        println!("Using img2webp: {}", img2webp_path);

        let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
        let frames_dir = temp_dir.path();

        let frames_pattern = frames_dir.join("f%03d.png");

        println!("Extracting frames with ffmpeg...");
        let ffmpeg_output = Command::new(&ffmpeg_path)
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg(frames_pattern.to_str().unwrap())
            .output()
            .map_err(|e| e.to_string())?;

        if !ffmpeg_output.status.success() {
            return Err(format!(
                "ffmpeg failed: {}",
                String::from_utf8_lossy(&ffmpeg_output.stderr)
            ));
        }

        let mut png_files: Vec<String> = fs::read_dir(frames_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_string_lossy().to_string())
            .filter(|s| s.ends_with(".png"))
            .collect();

        png_files.sort();

        println!("Found {} PNG files", png_files.len());

        if png_files.is_empty() {
            return Err("No PNG files created by ffmpeg".to_string());
        }

        println!("Creating animated WebP with img2webp...");

        let mut cmd = Command::new(&img2webp_path);
        for f in &png_files {
            cmd.arg(f);
        }
        cmd.arg("-o").arg(output_path);

        let img2webp_output = cmd.output().map_err(|e| e.to_string())?;

        if !img2webp_output.status.success() {
            return Err(format!(
                "img2webp failed: {}",
                String::from_utf8_lossy(&img2webp_output.stderr)
            ));
        }

        println!("Animated WebP saved successfully!");
        Ok(())
    }

    fn convert_with_ffmpeg(input_path: &str, webp_path_str: &str) -> Option<String> {
        let output = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg("-c:v")
            .arg("libwebp")
            .arg("-lossless")
            .arg("0")
            .arg("-loop")
            .arg("0")
            .arg("-fps_mode")
            .arg("vfr")
            .arg(webp_path_str)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("Conversion successful (ffmpeg)");
                Some(webp_path_str.to_string())
            }
            Ok(o) => {
                println!("Conversion failed: {}", String::from_utf8_lossy(&o.stderr));
                None
            }
            Err(e) => {
                println!("Failed to run ffmpeg: {}", e);
                None
            }
        }
    }

    pub fn add_character(&mut self, path: &str) {
        let path_lower = path.to_lowercase();

        if !path_lower.ends_with(".apng")
            && !path_lower.ends_with(".png")
            && !path_lower.ends_with(".webp")
            && !path_lower.ends_with(".gif")
        {
            return;
        }

        let final_path: String;
        let name: String;

        if path_lower.ends_with(".apng") {
            if let Some(converted) = Self::convert_apng_to_webp(path) {
                final_path = converted;
            } else {
                println!("Failed to convert APNG, using original");
                final_path = path.to_string();
            }
        } else {
            final_path = path.to_string();
        }

        name = PathBuf::from(&final_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        self.add_named_character(&final_path, &name);
    }

    pub fn add_named_character(&mut self, path: &str, name: &str) {
        let path_lower = path.to_lowercase();

        if !path_lower.ends_with(".apng")
            && !path_lower.ends_with(".png")
            && !path_lower.ends_with(".webp")
            && !path_lower.ends_with(".gif")
        {
            return;
        }

        if !self.characters.iter().any(|c| c.path == path) {
            self.characters.push(Character {
                name: name.to_string(),
                path: path.to_string(),
                enabled: false,
                speed: 0,
                scale: 1.0,
                window_pos: None,
                window_size: None,
            });
            self.save();
        }
    }

    pub fn remove_character(&mut self, index: usize) {
        if index < self.characters.len() {
            self.characters.remove(index);
            self.save();
        }
    }

    pub fn index_by_path(&self, path: &str) -> Option<usize> {
        self.characters.iter().position(|c| c.path == path)
    }

    pub fn set_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(character) = self.characters.get_mut(index) {
            character.enabled = enabled;
            self.save();
        }
    }

    pub fn update_settings(&mut self, index: usize, speed: i64, scale: f32) {
        if let Some(character) = self.characters.get_mut(index) {
            character.speed = speed;
            character.scale = scale;
            self.save();
        }
    }

    pub fn update_position(&mut self, index: usize, pos: [f32; 2]) {
        if let Some(character) = self.characters.get_mut(index) {
            character.window_pos = Some(pos);
            self.save();
        }
    }

    fn get_library_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/AnimaMac/library.json")
        } else if cfg!(windows) {
            home.join("AppData/Roaming/AnimaMac/library.json")
        } else {
            home.join(".config/animatux/library.json")
        }
    }
}
