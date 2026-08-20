// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_keychain_token() -> Result<String, String> {
    let entry = keyring::Entry::new("kestrel_auth", "user")
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(password) => Ok(password),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn set_keychain_token(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("kestrel_auth", "user")
        .map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_keychain_token() -> Result<(), String> {
    let entry = keyring::Entry::new("kestrel_auth", "user")
        .map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_keychain_token,
            set_keychain_token,
            delete_keychain_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
