use std::time::Duration;

use serde_json::Value;
use tauri::{
  plugin::{
    Builder as PluginBuilder,
    TauriPlugin
  }, Manager, Runtime, Window
};
use tokio::sync::Mutex;

pub mod server;
mod commands;

type RPCServerThreadState = Mutex<Option<std::sync::mpsc::Sender<()>>>;

pub fn execute_and_receive_js<R>(window: &Window<R>, script: &str) -> serde_json::Value
where
  R: Runtime,
{
  window
      .eval(
          format!(
              "window.__TAURI__.invoke('plugin:rpc|handle_js_return', {{input: {}}})",
              script
          )
          .as_str(),
      )
      .unwrap();

  server::CHANNELS
      .1
      .lock()
      .unwrap()
      .recv_timeout(Duration::from_millis(250))
      .unwrap_or(Value::Null)
}

pub fn init<R>() -> TauriPlugin<R> where R: Runtime {
  PluginBuilder::new("rpc")
    .invoke_handler(tauri::generate_handler![commands::handle_js_return, commands::start_rpc_server, commands::stop_rpc_server, commands::is_rpc_server_running])
    .setup(|app| {
      app.manage::<RPCServerThreadState>(Mutex::new(None));

      Ok(())
    })
    .build()
}
