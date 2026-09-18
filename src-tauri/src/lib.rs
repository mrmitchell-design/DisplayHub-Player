use serde_json::Value;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){tauri::Builder::default().invoke_handler(tauri::generate_handler![http_request]).run(tauri::generate_context!()).expect("error while running DisplayHub Player");}
