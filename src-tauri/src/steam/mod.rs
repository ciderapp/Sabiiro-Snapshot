use tauri::{
    async_runtime::RwLock,
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod callbacks;

use steamworks::{Client, SingleClient};

// NOTE(d3rpp)
//
// if attempting to access the client, use `try_state` instead of `state` as
// we cannot guarantee that the steam API activated correctly.

// our steamworks app id
pub const APP_ID: u32 = 2446120u32;

// this is moved into this continuously running function and is guaranteed to only be run once at a time
// since single is for things that can only be run once at a time
// does say it has to be the main thread but hey, idgaf
fn run_loop(single: SingleClient) {
    // this allows other tasks to run for a bit, basically
    // it pauses the task loop and puts it back into the
    // queue as to not hog any executor threads when it
    // doesnt need to.
    //
    // i hope
    loop {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[tauri::command]
async fn set_rich_presence<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    key: &str,
    track: &str,
    album: &str,
    artist: &str,
) -> Result<(), String> {
    if let Some(c) = app.try_state::<RwLock<Client>>() {
        let friends = c.read().await.friends();

        friends.set_rich_presence("title", Some(track));
        friends.set_rich_presence("album", Some(album));
        friends.set_rich_presence("artist", Some(artist));
        friends.set_rich_presence("status", Some(key));
        friends.set_rich_presence("steam_display", Some(key));

        Ok(())
    } else {
        Err("Steam API Not Connected".into())
    }
}

#[tauri::command]
async fn clear_rich_presence<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<(), String> {
    if let Some(c) = app.try_state::<RwLock<Client>>() {
        let friends = c.read().await.friends();

        friends.set_rich_presence("steam_display", None);
        friends.set_rich_presence("status", None);

        Ok(())
    } else {
        Err("Steam API Not Connected".into())
    }
}

#[tauri::command]
fn active<R: Runtime>(app: tauri::AppHandle<R>, _window: tauri::Window<R>) -> bool {
    app.try_state::<RwLock<Client>>().is_some()
}

// allow achievements to work
#[tauri::command]
async fn grant_achievement<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    achievement: String,
) -> bool {
    if let Some(c) = app.try_state::<RwLock<Client>>() {
        let c_lock = c.read().await;
        let user = c_lock.user_stats();
        user.request_current_stats();

        let ach = user.achievement(achievement.as_str());

        if ach.set().is_ok() {
            user.store_stats().is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("steamworks")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app| {
            let (client, single) = match Client::init_app(APP_ID) {
                Ok(r) => r,
                Err(e) => {
                    println!("Unable to connect to Steam: {:?}", e);
                    return Ok(());
                }
            };

            callbacks::set_callbacks(&client);

            tauri::async_runtime::spawn_blocking(|| run_loop(single));

            app.manage(RwLock::new(client));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_rich_presence,
            clear_rich_presence,
            active,
            grant_achievement
        ])
        .build()
}
