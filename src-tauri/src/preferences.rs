use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all="camelCase")]
pub struct Preferences { pub theme: String, pub manager_font: String, pub manager_font_size: u16, pub cyoa_font: String, pub archive_limit: usize }
impl Default for Preferences { fn default()->Self {Self{theme:"mocha".into(),manager_font:"system-ui".into(),manager_font_size:16,cyoa_font:String::new(),archive_limit:5}} }
pub fn load()->Preferences { std::fs::read(crate::library::data_root_dir().join("save/preferences.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or_default() }
#[tauri::command]
pub fn set_preferences(mut preferences: Preferences)->Result<(),String>{
    preferences.archive_limit=preferences.archive_limit.clamp(3,5);
    preferences.manager_font_size=preferences.manager_font_size.clamp(12,24);
    for font in [&preferences.manager_font,&preferences.cyoa_font] {if font.len()>200 || font.chars().any(|c| c=='\n' || c=='\r') {return Err("Invalid font name".into());}}
    let file=crate::library::data_root_dir().join("save/preferences.json");
    std::fs::create_dir_all(file.parent().unwrap()).map_err(|e|e.to_string())?;
    let pending=file.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));
    std::fs::write(&pending,serde_json::to_vec_pretty(&preferences).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    std::fs::rename(pending,file).map_err(|e|e.to_string())
}
#[tauri::command(async)]
pub fn get_fonts()->Vec<String>{
    let mut fonts=vec!["system-ui".into(),"sans-serif".into(),"serif".into(),"monospace".into()];
    if let Ok(result)=std::process::Command::new("fc-list").arg("--format=%{family}\\n").output(){
        if result.status.success(){ for line in String::from_utf8_lossy(&result.stdout).lines(){for family in line.split(','){let family=family.trim();if !family.is_empty(){fonts.push(family.into());}}} }
    }
    fonts.sort();fonts.dedup();fonts
}

pub fn background_color(theme:&str)->tauri::window::Color {let(r,g,b)=match theme{"latte"=>(239,241,245),"frappe"=>(48,52,70),"macchiato"=>(36,39,58),_=>(30,30,46)};tauri::window::Color(r,g,b,255)}
pub fn base_color(theme:&str)->&'static str{match theme{"latte"=>"#eff1f5","frappe"=>"#303446","macchiato"=>"#24273a",_=>"#1e1e2e"}}
pub fn serve_regular_font(uri:&tauri::http::Uri)->tauri::http::Response<Vec<u8>>{
 let name=percent_encoding::percent_decode_str(uri.path().trim_start_matches('/')).decode_utf8_lossy();
 let response=(||->Result<Vec<u8>,String>{if name.len()>200||name.contains(['\n','\r']){return Err("Invalid font family".into());}
  let result=std::process::Command::new("fc-match").args(["--format=%{file}", &format!("{}:weight=80:style=Regular",name)]).output().map_err(|e|e.to_string())?;
  let file=String::from_utf8_lossy(&result.stdout);let path=std::path::Path::new(file.trim());
  if !path.extension().and_then(|x|x.to_str()).is_some_and(|ext|matches!(ext,"ttf"|"otf"|"ttc")){return Err("No regular font face available".into());}std::fs::read(path).map_err(|e|e.to_string())
 })();match response{Ok(bytes)=>tauri::http::Response::builder().header("Content-Type","font/ttf").header("Access-Control-Allow-Origin","*").header("Cache-Control","max-age=86400").body(bytes).unwrap(),Err(error)=>tauri::http::Response::builder().status(404).body(error.into_bytes()).unwrap()}
}
