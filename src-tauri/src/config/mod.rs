use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Runtime,
};

use tokio::fs::{
  read_to_string,
  write as write_str,
  create_dir_all,
};

#[tauri::command]
async fn read<R>(handle: AppHandle<R>) -> Result<Option<String>, String>
where
    R: Runtime,
{
    let cfg_path = handle
        .path_resolver()
        .app_config_dir()
        .expect("Unknown Application Config Dir");
    let file_path = cfg_path.join("spa-config.json");

    if file_path.exists() {
        match read_to_string(file_path).await {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(format!("Read Config Error: {}", e.to_string())),
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn write<R>(handle: AppHandle<R>, content: String) -> Result<(), String>
where
    R: Runtime,
{
    let cfg_path = handle
        .path_resolver()
        .app_config_dir()
        .expect("Unknown Application Config Dir");

    if !cfg_path.exists() {
      create_dir_all(cfg_path.clone()).await.map_err(|e| format!("Write Config Error: {}", e.to_string()))?;
    }

    let file_path = cfg_path.join("spa-config.json");

    write_str(file_path, content)
        .await
        .map_err(|e| format!("Write Config Error: {}", e.to_string()))
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    PluginBuilder::new("config")
        .invoke_handler(tauri::generate_handler![read, write])
        .build()
}
