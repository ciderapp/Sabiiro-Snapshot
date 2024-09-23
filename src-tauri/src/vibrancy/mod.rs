use std::sync::Mutex;

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

mod common;

#[cfg(windows)]
pub mod windows;

struct VibrancyMode(Mutex<&'static str>);

#[tauri::command]
fn set_mode<R>(handle: tauri::AppHandle<R>, window: tauri::Window<R>, variant: String) -> bool
where
    R: Runtime,
{
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let hwnd = window.hwnd().expect("couldn't get HWND").0;

            let current = handle.state::<VibrancyMode>();
            let mut s = current.0.lock().unwrap();

            if *s == variant.as_str() {
                return true;
            }

            match variant.as_str() {
                "light" => {
                    if windows::set_light_mode(hwnd) {
                        *s = "acrylic";
                        true
                    } else {
                        false
                    }
                },

                "dark" => {
                    if windows::set_dark_mode(hwnd) {
                        *s = "acrylic";
                        true
                    } else {
                        false
                    }
                },

                _ => false
            }
        } else {
            false
        }
    }
}

#[tauri::command]
///
/// `variant` should be one of the following
///  - `mica` => to apply mica
///  - `tabbed` => to apply Tabbed
///  - `acrylic` => to apply Acrylic
///
/// any others will be ignored, will return true if the application worked, otherwise false
///
fn set_vibrancy<R>(handle: tauri::AppHandle<R>, window: tauri::Window<R>, variant: String) -> bool
where
    R: Runtime,
{
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let hwnd = window.hwnd().expect("couldn't get HWND").0;

            let current = handle.state::<VibrancyMode>();
            let mut s = current.0.lock().unwrap();

            if *s == variant.as_str() {
                return true;
            }

            match variant.as_str() {
                "mica" => {
                    if windows::apply_mica(hwnd) {
                        *s = "mica";
                        return true;
                    }

                    false
                },

                "acrylic" => {
                    if windows::apply_acrylic(hwnd, None) {
                        *s = "acrylic";
                        return true;
                    }

                    false
                },

                "blur" => {
                    if windows::apply_blur(hwnd, None) {
                        *s = "blur";
                        return true;
                    }

                    false
                },

                "tabbed" => {
                    if windows::apply_tabbed(hwnd) {
                        *s = "tabbed";
                        return true;
                    }

                    false
                }

                _ => false
            }
        } else {
            false
        }
    }
}

#[tauri::command]
fn clear_vibrancy<R>(handle: tauri::AppHandle<R>, window: tauri::Window<R>) -> bool
where
    R: Runtime,
{
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let hwnd = window.hwnd().expect("Unable to get HWND").0;

            let current = handle.state::<VibrancyMode>();
            let mut s = current.0.lock().unwrap();
            let c = *s;

            match c {
                "mica" => {
                    if windows::clear_mica(hwnd) {
                        *s = "none";
                        return true;
                    }

                    false
                },

                "acrylic" => {
                    if windows::clear_acrylic(hwnd) {
                        *s = "none";
                        return true;
                    }

                    false
                },

                "blur" => {
                    if windows::clear_blur(hwnd) {
                        *s = "none";
                        return true;
                    }

                    false
                },

                "tabbed" => {
                    if windows::clear_tabbed(hwnd) {
                        *s = "none";
                        return true;
                    }

                    false
                }

                _ => false
            }
        } else {
            false
        }
    }
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    PluginBuilder::new("vibrancy")
        .setup(|handle| {
            handle.manage(VibrancyMode(Mutex::new("none")));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_vibrancy,
            clear_vibrancy,
            set_mode,
        ])
        .build()
}
