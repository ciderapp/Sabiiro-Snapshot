use tauri::{
    async_runtime::RwLock,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

pub mod error;
use error::DiscordError;

use discord_rich_presence::{
    activity::{Activity, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

pub struct DiscordRPC {
    client_id: RwLock<Option<String>>,
    inner_client: RwLock<Option<DiscordIpcClient>>,
}

impl DiscordRPC {
    pub fn new() -> Self {
        Self {
            client_id: RwLock::new(None),
            inner_client: RwLock::new(Option::<DiscordIpcClient>::None),
        }
    }

    pub fn init(&self, client_id: impl AsRef<str>) -> Result<(), DiscordError> {
        let actual_id = match client_id.as_ref() {
            "Cider-2" => "1020414178047041627",
            "AppleMusic" => "886578863147192350",
            _ => "911790844204437504",
        };

        let mut lock = self.inner_client.blocking_write();
        if lock.is_some() {
            // already initialised
            return Ok(());
        }

        let mut l = self.client_id.blocking_write();
        *l = Some(actual_id.to_string());
        drop(l);

        if let Ok(mut client) = DiscordIpcClient::new(actual_id) {
            client.connect().unwrap();
            *lock = Some(client);
            drop(lock);
            Ok(())
        } else {
            Err(DiscordError::Init(
                "Failed to initialise Discord RPC".to_string(),
            ))
        }
    }

    pub fn remove(&self) {
        let mut lock = self.inner_client.blocking_write();

        if let Some(client) = &mut *lock {
            client.clear_activity().ok();
            client.close().ok();
        }

        *lock = None;
    }

    pub async fn reconnect(&self) -> bool {
        if let Some(client) = &mut *self.inner_client.write().await {
            let res = client.reconnect().is_ok();
            if res {
                println!("CLIENT RECONNECTED");
            } else {
                println!("CLIENT RECONNECTION FAILED");
            }
            return res;
        }
        false
    }

    pub async fn set_rpc<'a>(
        &self,
        state: String,
        details: String,
        artwork: String,
        ts: Option<(i64, i64)>,
        buttons: &'a Vec<RPCButton>,
        large_image_text: String,
    ) -> Result<(), DiscordError> {
        if let Some(client) = &mut *self.inner_client.write().await {
            println!("DISCORD RPC UPDATING");

            let mut tso = Timestamps::new();

            if let Some(t) = ts {
                tso = tso.start(t.0);
                tso = tso.end(t.1);
            }

            let buttons_converted: Vec<Button<'a>> = {
                let mut a = vec![];
                for i in buttons {
                    a.push(Button::new(i.label.as_str(), i.url.as_str()));
                }
                a
            };

            let activity_payload = Activity::new()
                .state(&state)
                .details(&details)
                .timestamps(tso)
                .buttons(buttons_converted)
                .assets(
                    Assets::new()
                        .large_image(&artwork)
                        .large_text(&large_image_text),
                );

            if let Err(e) = client.set_activity(activity_payload) {
                return Err(DiscordError::Status(e.to_string()));
            }
            Ok(())
        } else {
            Err(DiscordError::NoClient)
        }
    }

    pub async fn clear_rpc(&self) {
        if let Some(client) = &mut *self.inner_client.write().await {
            client.clear_activity().ok();
            client.close().ok();
        }
    }
}

#[tauri::command]
pub async fn init_client<R>(handle: AppHandle<R>, client_id: String) -> Result<(), DiscordError>
where
    R: Runtime,
{
    tauri::async_runtime::spawn_blocking(move || {
        let client = handle.state::<DiscordRPC>();
        client.init(client_id)
    })
    .await
    .unwrap()?;

    Ok(())
}

#[tauri::command]
async fn stop_client<R>(handle: AppHandle<R>)
where
    R: Runtime,
{
    tauri::async_runtime::spawn_blocking(move || {
        let client = handle.state::<DiscordRPC>();
        client.remove();
    })
    .await
    .unwrap();
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct RPCButton {
    pub label: String,
    pub url: String,
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn set_status<R>(
    handle: AppHandle<R>,
    state: String,
    details: String,
    artwork: String,
    start: Option<i64>,
    end: Option<i64>,
    buttons: Option<Vec<RPCButton>>,
    large_image_text: String,
) -> Result<(), DiscordError>
where
    R: Runtime,
{
    let client = handle.state::<DiscordRPC>();

    // huh? ok
    let timestamps = start.and_then(|s| end.map(|e| (s, e)));

    let b = buttons.unwrap_or(vec![]);

    if let Err(_e) = client
        .set_rpc(
            state.clone(),
            details.clone(),
            artwork.clone(),
            timestamps,
            &b,
            large_image_text.clone(),
        )
        .await
    {
        if client.reconnect().await {
            return client
                .set_rpc(state, details, artwork, timestamps, &b, large_image_text)
                .await;
        } else {
            Err(DiscordError::NoClient)
        }
    } else {
        Ok(())
    }
}

#[tauri::command]
async fn idle_status<R>(handle: AppHandle<R>) -> Result<(), String>
where
    R: Runtime,
{
    let client = handle.state::<DiscordRPC>();
    client
        .set_rpc(
            String::new(),
            "Browsing Cider".to_owned(),
            "https://cdn.discordapp.com/icons/843954443845238864/ffdf21ed4aa8748be2fbe411fdcf525b.webp?size=96".to_owned(),
            None,
            &vec![],
            "".to_owned()
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_status<R>(handle: AppHandle<R>)
where
    R: Runtime,
{
    let client = handle.state::<DiscordRPC>();
    client.clear_rpc().await;
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    PluginBuilder::new("discord")
        .setup(|handle| {
            handle.manage(DiscordRPC::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_client,
            stop_client,
            set_status,
            clear_status,
            idle_status
        ])
        .build()
}
