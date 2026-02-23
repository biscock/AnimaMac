// Optional feature to support steamcmd for workshop content
use crate::logging::log_to_file;
use std::{env::home_dir, path::Path, process::Command};

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub path: String,
    pub files: Vec<String>,
}

pub fn get_ws() -> String {
    let home = match home_dir() {
        Some(h) => h,
        None => {
            log_to_file("steamlib: home_dir() returned None; defaulting to current dir");
            return ".".to_string();
        }
    };
    return if cfg!(windows) {
        format!(
            "C:\\Program Files (x86)\\Steam\\steamapps\\workshop\\content\\{}",
            include_str!("../assets/appid")
        )
    } else if cfg!(target_os = "macos") {
        format!(
            "{}/Library/Application Support/Steam/steamapps/workshop/content/{}",
            home.to_str().unwrap_or("."),
            include_str!("../assets/appid")
        )
    } else {
        format!(
            "{}/.local/share/Steam/steamapps/workshop/content/{}",
            home.to_str().unwrap_or("."),
            include_str!("../assets/appid")
        )
    };
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

pub fn extract_workshop_id(input: &str) -> String {
    let input = input.trim();

    if input.chars().all(|c| c.is_ascii_digit()) {
        return input.to_string();
    }

    if let Some(pos) = input.rfind('?') {
        let params = &input[pos + 1..];
        for param in params.split('&') {
            if param.starts_with("id=") {
                return param[3..].to_string();
            }
        }
    }

    if let Some(pos) = input.rfind('/') {
        let after_slash = &input[pos + 1..];
        let clean: String = after_slash.chars().filter(|c| c.is_ascii_digit()).collect();
        if !clean.is_empty() {
            return clean;
        }
    }

    input.to_string()
}

fn is_valid_media_file(filename: &str) -> bool {
    let filename = filename.to_lowercase();
    if filename.starts_with('.') {
        return false;
    }
    if filename == "ds_store" {
        return false;
    }
    filename.ends_with(".apng")
        || filename.ends_with(".png")
        || filename.ends_with(".webp")
        || filename.ends_with(".gif")
}

fn convert_apng_to_webp_internal(input_path: &str, output_path: &str) -> Result<(), String> {
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    println!("Converting APNG to animated WebP...");

    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
    let frames_dir = temp_dir.path();

    let frames_pattern = frames_dir.join("f%03d.png");

    println!("Extracting frames with ffmpeg...");
    let ffmpeg_output = Command::new("ffmpeg")
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

    let mut cmd = Command::new("img2webp");
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

fn convert_apng_to_webp(input_path: &str, output_path: &str) -> bool {
    println!("Converting APNG to WebP: {} -> {}", input_path, output_path);

    match convert_apng_to_webp_internal(input_path, output_path) {
        Ok(_) => true,
        Err(e) => {
            println!("Conversion failed: {}", e);
            false
        }
    }
}

pub fn list_ws(ws: &String) -> Vec<String> {
    if !(std::fs::exists(&ws).unwrap_or(false)) {
        if let Err(e) = std::fs::create_dir_all(&ws) {
            log_to_file(&format!("steamlib: failed to create workshop dir {}: {}", ws, e));
            return vec![];
        }
    }

    let content = match std::fs::read_dir(ws) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut ids: Vec<String> = vec![];
    for entry in content.flatten() {
        let e = match entry.metadata() {
            Ok(m) if m.is_dir() => entry,
            _ => continue,
        };

        let e_name = e.file_name().to_str().unwrap().to_owned();
        let files = match std::fs::read_dir(e.path()) {
            Ok(f) => f,
            Err(_) => continue,
        };

        for file in files.flatten() {
            let fname = file.file_name();
            let fname_str = fname.to_str().unwrap();

            if !is_valid_media_file(fname_str) {
                continue;
            }

            if include_str!("../assets/unsupported")
                .lines()
                .any(|ft| fname_str.ends_with(&format!("{}", &ft)))
            {
                continue;
            }

            ids.push(format!("{}/{}", e_name, fname_str));
        }
    }
    return ids;
}

pub fn workshop_dl(id: &String, ws: &String) -> Option<DownloadResult> {
    let workshop_id = extract_workshop_id(id);

    log_to_file(&format!(
        "steamlib: downloading workshop id {} (raw input: {})",
        workshop_id, id
    ));

    let steamcmd_path = match find_command("steamcmd") {
        Some(p) => p,
        None => {
            log_to_file("steamlib: steamcmd not found in PATH or common locations");
            return None;
        }
    };

    log_to_file(&format!("steamlib: using steamcmd at {}", steamcmd_path));

    let output = Command::new(&steamcmd_path)
        .arg("+login")
        .arg("anonymous")
        .arg("+workshop_download_item")
        .arg(include_str!("../assets/appid"))
        .arg(&workshop_id)
        .arg("+quit")
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            log_to_file(&format!("steamlib: failed to execute steamcmd: {}", e));
            return None;
        }
    };

    let path = format!("{}/{}", ws, workshop_id);

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        log_to_file(&format!("steamlib: steamcmd stdout:\n{}", stdout));
    }
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_to_file(&format!("steamlib: steamcmd stderr:\n{}", stderr));
    }

    if !Path::new(&path).exists() {
        log_to_file(&format!("steamlib: download folder not found: {}", path));
        return None;
    }

    process_downloaded_files(&path)
}

fn process_downloaded_files(path: &str) -> Option<DownloadResult> {
    let files = match std::fs::read_dir(path) {
        Ok(f) => f,
        Err(e) => {
            log_to_file(&format!(
                "steamlib: failed to read downloaded folder {}: {}",
                path, e
            ));
            return None;
        }
    };

    let mut downloaded_files: Vec<String> = vec![];

    for img in files.flatten() {
        let fname = img.file_name().to_str().unwrap().to_owned();

        if !is_valid_media_file(&fname) {
            continue;
        }

        let full_path = format!("{}/{}", path, fname);
        let fname_lower = fname.to_lowercase();

        if fname_lower.ends_with(".apng") {
            let webp_name = fname.trim_end_matches(".apng").to_string() + ".webp";
            let webp_path = format!("{}/{}", path, webp_name);

            if convert_apng_to_webp(&full_path, &webp_path) {
                downloaded_files.push(webp_name);
            }
        } else {
            downloaded_files.push(fname);
        }
    }

    if downloaded_files.is_empty() {
        log_to_file("steamlib: no valid media files found after download");
        return None;
    }

    Some(DownloadResult {
        path: path.to_string(),
        files: downloaded_files,
    })
}
