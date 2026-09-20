use serde_json::Value;
use std::process::{Child,Command,Stdio};
use std::sync::Mutex;
use tauri::{AppHandle,Manager,State};

struct AirplayState(Mutex<Option<Child>>);

#[tauri::command]
async fn http_request(url:String,method:Option<String>,body:Option<Value>,token:Option<String>)->Result<Value,String>{
 let client=reqwest::Client::builder().timeout(std::time::Duration::from_secs(12)).build().map_err(|e|e.to_string())?;
 let mut request=match method.as_deref().unwrap_or("GET"){"POST"=>client.post(&url),_=>client.get(&url)};
 if let Some(t)=token{request=request.bearer_auth(t).header("x-displayhub-player-version","0.3.1");}
 if let Some(value)=body{request=request.json(&value);}
 let response=request.send().await.map_err(|e|format!("Unable to reach DisplayHub: {e}"))?;
 let status=response.status();let text=response.text().await.map_err(|e|e.to_string())?;
 let data:Value=serde_json::from_str(&text).unwrap_or_else(|_|serde_json::json!({"error":text}));
 if !status.is_success(){return Err(data.get("error").and_then(|v|v.as_str()).unwrap_or("DisplayHub returned an error").to_string())}Ok(data)
}

#[tauri::command]
fn set_kiosk(app:AppHandle,enabled:bool)->Result<(),String>{
 let window=app.get_webview_window("main").ok_or("Player window not found")?;
 window.set_fullscreen(enabled).map_err(|e|e.to_string())?;
 window.set_decorations(!enabled).map_err(|e|e.to_string())?;
 Ok(())
}

#[tauri::command]
fn airplay_start(name:String,pin:Option<String>,state:State<AirplayState>)->Result<Value,String>{
 let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;
 if let Some(child)=guard.as_mut(){if child.try_wait().map_err(|e|e.to_string())?.is_none(){return Ok(serde_json::json!({"running":true,"name":name}));}}
 let receiver=format!("DisplayHub – {}",name.trim());
 let mut command=Command::new("uxplay");command.args(["-n",&receiver]);if let Some(code)=pin.filter(|p|p.len()==4&&p.chars().all(|c|c.is_ascii_digit())){command.args(["-pin",&code]);}let child=command.stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e|format!("UxPlay is not available: {e}"))?;
 *guard=Some(child);
 Ok(serde_json::json!({"running":true,"name":receiver}))
}

#[tauri::command]
fn airplay_status(state:State<AirplayState>)->Result<Value,String>{
 let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;
 let running=match guard.as_mut(){Some(child)=>child.try_wait().map_err(|e|e.to_string())?.is_none(),None=>false};
 if !running{*guard=None;}
 Ok(serde_json::json!({"available":Command::new("uxplay").arg("-h").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok(),"running":running}))
}

#[tauri::command]
fn airplay_stop(state:State<AirplayState>)->Result<(),String>{let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;if let Some(mut child)=guard.take(){let _=child.kill();let _=child.wait();}Ok(())}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){tauri::Builder::default().manage(AirplayState(Mutex::new(None))).plugin(tauri_plugin_autostart::Builder::new().app_name("DisplayHub Player").build()).setup(|app|{use tauri_plugin_autostart::ManagerExt;let _=app.autolaunch().enable();Ok(())}).invoke_handler(tauri::generate_handler![http_request,set_kiosk,airplay_start,airplay_status,airplay_stop]).run(tauri::generate_context!()).expect("error while running DisplayHub Player");}
