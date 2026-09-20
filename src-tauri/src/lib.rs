use serde_json::Value;
use std::io::{BufRead,BufReader,Write};
use std::process::{Child,Command,Stdio};
use std::sync::{Arc,Mutex};
use std::thread;
use tauri::{AppHandle,Manager,State};

struct AirplayProcess{child:Child,pin:Arc<Mutex<Option<String>>>,active:Arc<Mutex<bool>>}
struct AirplayState(Mutex<Option<AirplayProcess>>);

#[tauri::command]
async fn http_request(url:String,method:Option<String>,body:Option<Value>,token:Option<String>)->Result<Value,String>{
 let client=reqwest::Client::builder().timeout(std::time::Duration::from_secs(12)).build().map_err(|e|e.to_string())?;
 let mut request=match method.as_deref().unwrap_or("GET"){"POST"=>client.post(&url),_=>client.get(&url)};
 if let Some(t)=token{request=request.bearer_auth(t).header("x-displayhub-player-version","0.3.2");}
 if let Some(value)=body{request=request.json(&value);}
 let response=request.send().await.map_err(|e|format!("Unable to reach DisplayHub: {e}"))?;
 let status=response.status();let text=response.text().await.map_err(|e|e.to_string())?;
 let data:Value=serde_json::from_str(&text).unwrap_or_else(|_|serde_json::json!({"error":text}));
 if !status.is_success(){return Err(data.get("error").and_then(|v|v.as_str()).unwrap_or("DisplayHub returned an error").to_string())}Ok(data)
}

#[tauri::command]
fn set_kiosk(app:AppHandle,enabled:bool)->Result<(),String>{let window=app.get_webview_window("main").ok_or("Player window not found")?;window.set_fullscreen(enabled).map_err(|e|e.to_string())?;window.set_decorations(!enabled).map_err(|e|e.to_string())?;Ok(())}

fn digit_rows()->Vec<Vec<String>>{
 let encoded=[
 ["0821111380","2114005113","1110000111","1110000111","1110000111","1110000111","5113002114","0751111470"],
 ["0002111000","0021111000","0000111000","0000111000","0000111000","0000111000","0000111000","0011111110"],
 ["0811112800","2114005113","0000000111","0000082114","0862111470","2114700000","1117000000","1111111111"],
 ["0821111380","2114005113","0000082114","0000111170","0000075130","1110000111","5113002114","0751111470"],
 ["0000211110","0001401110","0021401110","0214001110","2110001110","1111111111","0000001110","0000001110"],
 ["1111111110","1110000000","1110000000","1112111380","0000075113","0000000111","5113002114","0711114700"],
 ["0821111380","2114005113","1110000000","1112111380","1114075113","1110000111","5113002114","0751111470"],
 ["1111111111","0000002114","0000021140","0000211400","0002114000","0021140000","0211400000","2114000000"],
 ["0831111280","2114002114","5113802114","0751111170","8214775138","1110000111","5113002114","0751111470"],
 ["0821111380","2114005113","1110000111","5113802111","0751114111","0000000111","5113002114","0751111470"]
 ];
 let pixels=[' ','8','d','b','P','Y','o','"','.'];
 encoded.iter().map(|d|d.iter().map(|r|r.chars().map(|c|pixels[c.to_digit(10).unwrap() as usize]).collect()).collect()).collect()
}

fn decode_pin(lines:&[String])->Option<String>{
 if lines.len()<8{return None}let patterns=digit_rows();
 for start in 0..=lines.len()-8{
  let block=&lines[start..start+8];let mut out=String::new();let mut ok=true;
  for col in 0..4{
   let mut found=None;
   for(d,rows)in patterns.iter().enumerate(){
    let matches=(0..8).all(|row|{let chars:Vec<char>=block[row].chars().collect();let offset=10+col*13;if chars.len()<offset+10{return false}chars[offset..offset+10].iter().collect::<String>()==rows[row]});
    if matches{found=Some(d);break}
   }
   if let Some(d)=found{out.push(char::from_digit(d as u32,10).unwrap())}else{ok=false;break}
  }
  if ok{return Some(out)}
 }None
}

fn watch_output<R:std::io::Read+Send+'static>(reader:R,pin:Arc<Mutex<Option<String>>>,active:Arc<Mutex<bool>>){
 thread::spawn(move||{let mut recent:Vec<String>=Vec::new();for line in BufReader::new(reader).lines().map_while(Result::ok){if let Ok(mut log)=std::fs::OpenOptions::new().create(true).append(true).open("/tmp/displayhub-uxplay.log"){let _=writeln!(log,"{}",line);}let lower=line.to_lowercase();if lower.contains("client disconnected"){if let Ok(mut a)=active.lock(){*a=false}if let Ok(mut p)=pin.lock(){*p=None}}if lower.contains("connection")||lower.contains("mirroring")||lower.contains("streaming"){if !lower.contains("disconnected"){if let Ok(mut a)=active.lock(){*a=true}}}recent.push(line);if recent.len()>12{recent.remove(0);}if let Some(code)=decode_pin(&recent){if let Ok(mut p)=pin.lock(){*p=Some(code)}}}}); 
}

#[cfg(target_os="windows")]
fn windows_airplay_exe()->Option<std::path::PathBuf>{
 if let Ok(p)=std::env::var("DISPLAYHUB_UXPLAY"){let p=std::path::PathBuf::from(p);if p.exists(){return Some(p)}}
 if let Ok(exe)=std::env::current_exe(){if let Some(dir)=exe.parent(){for p in [dir.join("airplay/bin/uxplay.exe"),dir.join("resources/airplay/bin/uxplay.exe"),dir.join("../Resources/airplay/bin/uxplay.exe")]{if p.exists(){return Some(p)}}}}
 ["C:\\Program Files\\DisplayHub Player\\airplay\\bin\\uxplay.exe","C:\\Program Files\\DisplayHub Player\\resources\\airplay\\bin\\uxplay.exe","C:\\msys64\\ucrt64\\bin\\uxplay.exe"].iter().map(std::path::PathBuf::from).find(|p|p.exists())
}

#[tauri::command]
fn airplay_start(name:String,dynamic_code:Option<bool>,state:State<AirplayState>)->Result<Value,String>{
 let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;
 if let Some(process)=guard.as_mut(){if process.child.try_wait().map_err(|e|e.to_string())?.is_none(){return Ok(serde_json::json!({"running":true,"name":name}));}}
 let receiver=format!("DisplayHub – {}",name.trim());
 #[cfg(target_os="windows")]
 let mut command={let exe=windows_airplay_exe().ok_or("Bundled Windows AirPlay runtime is missing")?;let bin=exe.parent().ok_or("AirPlay runtime path is invalid")?;let root=bin.parent().unwrap_or(bin);let plugins=root.join("lib/gstreamer-1.0");let mut cmd=Command::new(&exe);cmd.env("PATH",format!("{};{}",bin.display(),std::env::var("PATH").unwrap_or_default())).env("GST_PLUGIN_PATH",plugins).args(["-n",&receiver]);cmd};
 #[cfg(not(target_os="windows"))]
 let mut command={let mut cmd=Command::new("stdbuf");cmd.args(["-oL","-eL","uxplay","-n",&receiver]);cmd};
 if dynamic_code.unwrap_or(true){command.arg("-pw");}
 let mut child=command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|format!("UxPlay is not available: {e}"))?;
 let pin=Arc::new(Mutex::new(None));let active=Arc::new(Mutex::new(false));
 if let Some(stdout)=child.stdout.take(){watch_output(stdout,pin.clone(),active.clone())}if let Some(stderr)=child.stderr.take(){watch_output(stderr,pin.clone(),active.clone())}
 *guard=Some(AirplayProcess{child,pin,active});Ok(serde_json::json!({"running":true,"name":receiver}))
}

#[tauri::command]
fn airplay_status(state:State<AirplayState>)->Result<Value,String>{
 let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;let mut running=false;let mut pin=None;let mut active=false;
 if let Some(process)=guard.as_mut(){running=process.child.try_wait().map_err(|e|e.to_string())?.is_none();if running{pin=process.pin.lock().ok().and_then(|p|p.clone());active=process.active.lock().map(|a|*a).unwrap_or(false)}}
 if !running{*guard=None;}
 #[cfg(target_os="windows")]
 let available=windows_airplay_exe().is_some();
 #[cfg(not(target_os="windows"))]
 let available=Command::new("uxplay").arg("-h").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok();
 Ok(serde_json::json!({"available":available,"running":running,"pin":pin,"active":active}))
}

#[tauri::command]
fn airplay_stop(state:State<AirplayState>)->Result<(),String>{let mut guard=state.0.lock().map_err(|_|"AirPlay state unavailable")?;if let Some(mut process)=guard.take(){let _=process.child.kill();let _=process.child.wait();}Ok(())}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){tauri::Builder::default().manage(AirplayState(Mutex::new(None))).plugin(tauri_plugin_autostart::Builder::new().app_name("DisplayHub Player").build()).setup(|app|{use tauri_plugin_autostart::ManagerExt;let _=app.autolaunch().enable();Ok(())}).invoke_handler(tauri::generate_handler![http_request,set_kiosk,airplay_start,airplay_status,airplay_stop]).run(tauri::generate_context!()).expect("error while running DisplayHub Player");}
