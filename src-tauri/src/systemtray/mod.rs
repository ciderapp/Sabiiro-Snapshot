use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::bridge::Bridge;

#[tauri::command]
pub fn play(app: AppHandle) {
    app.tray_handle()
        .get_item("play")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("play")
        .set_title("Pause")
        .unwrap();
    app.tray_handle()
        .get_item("previous")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("next")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("addToLibrary")
        .set_enabled(true)
        .unwrap();
}

#[tauri::command]
pub fn change_song(app: AppHandle, song: String) {
    app.tray_handle()
        .get_item("songString")
        .set_title(song)
        .unwrap();
}

#[tauri::command]
pub fn pause(app: tauri::AppHandle) {
    app.tray_handle()
        .get_item("play")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("play")
        .set_title("Play")
        .unwrap();
    app.tray_handle()
        .get_item("previous")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("next")
        .set_enabled(true)
        .unwrap();
    app.tray_handle()
        .get_item("addToLibrary")
        .set_enabled(true)
        .unwrap();
}

pub fn init() -> SystemTray {
    let tray_menu =
        SystemTrayMenu::new() // insert the menu items here
            .add_item(CustomMenuItem::new("songString".to_string(), "Cider").disabled())
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("previous".to_string(), "Previous").disabled())
            .add_item(CustomMenuItem::new("play".to_string(), "Play/Pause").disabled())
            .add_item(CustomMenuItem::new("next".to_string(), "Next").disabled())
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("addToLibrary".to_string(), "Add to Library").disabled())
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("devtools".to_string(), "Open Devtools"))
            .add_item(CustomMenuItem::new("hide".to_string(), "Minimize to Tray"))
            .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    let system_tray: SystemTray = SystemTray::new().with_menu(tray_menu);
    system_tray
}

pub fn system_tray_event_handle(app: &AppHandle, event: SystemTrayEvent) {
    let bridge = Bridge::new(app.clone());
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            let window = app.get_window("cider_main").unwrap();
            let item_handle = app.tray_handle().get_item("hide");
            window.show().unwrap();
            window.unminimize().unwrap();
            window.set_focus().unwrap();
            item_handle.set_title("Minimize to Tray").unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let window = app.get_window("cider_main").unwrap();
            let item_handle = app.tray_handle().get_item(&id);
            match id.as_str() {
                "play" => {
                    bridge.play_pause();
                }
                "previous" => {
                    bridge.previous();
                }
                "next" => {
                    bridge.next();
                }
                "addToLibrary" => {
                    bridge.add_to_library();
                }
                "devtools" => {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                    window.open_devtools()
                }
                "hide" => {
                    if window.is_visible().unwrap() {
                        bridge.hide();
                        item_handle.set_title("Show Window").unwrap();
                    } else {
                        bridge.show();
                        item_handle.set_title("Minimize to Tray").unwrap();
                    }
                }
                "quit" => {
                    window.close().unwrap();
                }
                _ => {}
            }
        }
        _ => {}
    }
}
