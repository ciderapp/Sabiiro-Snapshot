use super::{server::create_rpc_server, RPCServerThreadState};
use tauri::{AppHandle, Runtime, State};

#[tauri::command]
pub fn handle_js_return(input: serde_json::Value) {
    super::server::CHANNELS
        .0
        .lock()
        .unwrap()
        .send(input)
        .unwrap();
}

#[tauri::command]
pub async fn start_rpc_server<R>(
    port: Option<u16>,
    app_handle: AppHandle<R>,
    server_thread: State<'_, RPCServerThreadState>,
) -> Result<(), String>
where
    R: Runtime,
{
    let mut lock = server_thread.lock().await;

    if lock.is_some() {
        return Err("Server Already Started".into());
    }

    let server = create_rpc_server(app_handle, port.unwrap_or_else(|| 10769u16));
    let join_handle_combo = server.stoppable();

    tauri::async_runtime::spawn_blocking(|| join_handle_combo.0.join());

    *lock = Some(join_handle_combo.1);

    Ok(())
}

#[tauri::command]
pub async fn stop_rpc_server(server_thread: State<'_, RPCServerThreadState>) -> Result<(), String> {
    let mut lock = server_thread.lock().await;

    match lock.as_ref() {
      None => Ok(()),

      Some(sender) => {
        sender.send(()).map_err(|e| e.to_string())?;

        *lock = None;

        Ok(())
      }
    }
}

#[tauri::command]
pub async fn is_rpc_server_running(server_thread: State<'_, RPCServerThreadState>) -> Result<bool, String> {
  Ok(server_thread.lock().await.is_some())
}
