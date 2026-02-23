use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn get_log_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/Logs/AnimaMac.log")
}

pub fn log_to_file(msg: &str) {
    let path = get_log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{}", msg);
    }
    eprintln!("{}", msg);
}
