#![allow(unused)]

use tokio::sync::mpsc::Receiver;

use tauri::{
    api::process::{Command, CommandChild, CommandEvent},
    async_runtime::{JoinHandle, RwLock},
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

pub mod error;
use error::AirPlayError;

use std::convert::TryInto;

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use fon::chan::{Ch16, Ch32};
use fon::Audio;

pub struct AirPlayClient {
    airtunes_child: RwLock<(Option<CommandChild>, Option<JoinHandle<()>>)>,
}

impl AirPlayClient {
    pub fn new() -> Self {
        Self {
            airtunes_child: RwLock::new((None, None)),
        }
    }

    pub async fn send_audio(&self, audio: String) {
        let mut lock = self.airtunes_child.write().await;

        let c = &mut lock.0;

        if let Some(client) = c {
            let orig_bytes = general_purpose::STANDARD.decode(audio).unwrap();

            let mut audio0 = Vec::new();
            for sample in orig_bytes.chunks(4) {
                audio0.push(f32::from_le_bytes(sample.try_into().unwrap()));
            }
            let audio1 = Audio::<Ch32, 2>::with_f32_buffer(96000, audio0);
            // Stream resampler into new audio type.
            let mut audio2 = Audio::<Ch16, 2>::with_audio(44100, &audio1);
            // Write file as i16 buffer.
            let mut bytes = Vec::new();
            for sample in audio2.as_i16_slice() {
                bytes.extend(sample.to_le_bytes());
            }
            client.write(&bytes).unwrap();
        }
    }

    pub async fn start_client(&self) -> Result<(), String> {
        let mut lock = self.airtunes_child.write().await;
        if lock.0.is_some() {
            return Ok(());
        }

        match Command::new_sidecar("airtunes2") {
            Ok(mut airtunes) => {
                let (mut rx, mut child) = airtunes.spawn().expect("Failed to spawn sidecar");
                let join_handle = tauri::async_runtime::spawn(async move {
                    // read events such as stdout
                    while let Some(event) = rx.recv().await {
                        if let CommandEvent::Stdout(message) = event {
                            println!("{}", message);
                        }
                        // else if let CommandEvent::Stderr(message) = event {
                        //     println!("{}", message);
                        // } else if let CommandEvent::Error(message) = event {
                        //     println!("{}", message);
                        // }
                    }
                });

                *lock = (Some(child), Some(join_handle));
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                Ok(())
            }

            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn stop_client(&self) {
        let mut lock = self.airtunes_child.write().await;

        // std::mem::take replaced with defaults (`None` in this case) so we don't need to set
        let c = std::mem::take(&mut lock.0);
        let j = std::mem::take(&mut lock.1);

        if let Some(c) = c {
            c.kill().ok();
        }

        if let Some(j) = j {
            j.abort();
        }
    }
}

#[tauri::command]
fn send_query<R>(
    handle: AppHandle<R>,
    state: String,
    details: String,
    artwork: String,
    start: i64,
    end: i64,
    large_image_text: String,
) where
    R: Runtime,
{
}

#[tauri::command]
async fn send_audio<R>(handle: AppHandle<R>, state: String)
where
    R: Runtime,
{
    tauri::async_runtime::spawn(async move {
        let client = handle.state::<AirPlayClient>();
        client.send_audio(state).await;
    })
    .await
    .unwrap();
}

#[tauri::command]
async fn start_client<R>(handle: AppHandle<R>) -> Result<(), String>
where
    R: Runtime,
{
    let client = handle.state::<AirPlayClient>();
    client.start_client().await
}

#[tauri::command]
async fn stop_client<R>(handle: AppHandle<R>)
where
    R: Runtime,
{
    let client = handle.state::<AirPlayClient>();
    client.stop_client().await;
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    PluginBuilder::new("airtunes")
        .setup(|handle| {
            handle.manage(AirPlayClient::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_query,
            send_audio,
            start_client,
            stop_client
        ])
        .build()
}
