// Optional feature to support steamcmd for workshop content
use std::{env::home_dir, process::{Command, exit}};


pub fn get_ws() -> String{
    /*
    Gets the main steam workshop location (windows too)
     */
    return if cfg!(windows) {
        format!(
            "C:\\Program Files (x86)\\Steam\\steamapps\\workshop\\content\\{}",
            include_str!("../assets/appid")
        )
    }else{
        format!(
            "{}/.local/share/Steam/steamapps/workshop/content/{}",
            home_dir().unwrap().to_str().unwrap(),
            include_str!("../assets/appid")
        )
    };
}

pub fn list_ws(ws: &String) -> Vec<String>{
    //Check if folder is real  (might be first launch)

    if !(std::fs::exists(&ws).unwrap()){
        let _ = match std::fs::create_dir_all(&ws) {
            Ok(_) => {
                
            },
            Err(_) => {
                exit(69);
            },
        };
    }

    let content = std::fs::read_dir(ws).unwrap();
    let mut ids: Vec<String> = vec![];
    for entry in content{
        let e = entry.unwrap();
        let e_name = e.file_name().to_str().unwrap().to_owned();
            for file in std::fs::read_dir(e.path()).unwrap(){
                let fname =  file.unwrap().file_name();
                if !(include_str!("../assets/unsupported").lines().any(|ft| fname.to_str().unwrap().ends_with(&format!("{}",&ft)))){
                    ids.push(format!("{}/{}", e_name,fname.to_str().unwrap().to_owned()));
                }
            }
        
        // 
    }
    return ids;
}

// Downloads from workshop as "Steam" without account
pub fn workshop_dl(id: &String, ws: &String){
    println!("Downloading ID {} with steamcmd", id);
    let output = Command::new("steamcmd")
        .arg("+login")
        .arg("anonymous")
        .arg("+workshop_download_item")
        .arg(include_str!("../assets/appid"))
        .arg(id)
        .arg("+quit")
        .output()
        .expect("Something went wrong using SteamCMD!");
    let path  = format!(
        "{}/{}",
        ws, id
    );
    for img in std::fs::read_dir(&path).unwrap(){
        let i = img.unwrap();
        if include_str!("../assets/unsupported").lines().any(|ft| i.file_name().to_str().unwrap().ends_with(&ft)){
            let output = Command::new("ffmpeg")
                .arg("-i")
                .arg(format!("{}/{}", path, i.file_name().to_str().unwrap()))
                .arg("-loop")
                .arg("0")
                .arg(format!("{}/{}.webp", path, i.file_name().to_str().unwrap()))
                .arg("-y")
                .output();
            println!("{}", String::from_utf8(output.unwrap().stderr).unwrap())
        }
    }

    println!("{}", String::from_utf8(output.stdout).expect("Couldn't print response"))
    
}