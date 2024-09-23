// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Arc;

use plugin::Plugins;
use tauri::{
    async_runtime::RwLock,
    utils::{assets::EmbeddedAssets, config::AppUrl},
    Context, Manager, Theme, WindowBuilder, WindowUrl, Wry,
};

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[macro_use]
extern crate obfstr;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "windows")]
#[path = "./platform/windows.rs"]
mod platform;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea, Win32::UI::Controls::MARGINS,
    Win32::UI::WindowsAndMessaging::GetWindowLongA, Win32::UI::WindowsAndMessaging::SetWindowLongA,
    Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
};
use zipdist::{startup_zip_check, PROTOCOL_VERSION};

mod zipdist;

mod additional;
mod airplay;
mod bridge;
mod config;
mod discord;
mod lastfm;
mod musickit;
mod plugin;
mod rpc;
#[cfg(feature = "steamworks")]
mod steam;
mod updater;
mod ws;
mod ziphttp;
#[cfg(not(feature = "steamworks"))]
mod steam {
    use tauri::{
        plugin::{Builder, TauriPlugin},
        Runtime,
    };

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("steamworks_dud_to_make_steam_stfu").build()
    }
}

mod systemtray;
mod vibrancy;

lazy_static::lazy_static! {
  static ref ITSPOD: RwLock<Option<String>> = RwLock::new(None);
  static ref PLUGINS: Arc<RwLock<Option<Plugins<Wry>>>> = Arc::new(RwLock::new(None));
}

#[tauri::command]
async fn set_itspod(itspod: Option<String>) {
    *ITSPOD.write().await = itspod;
}

#[tauri::command]
async fn init_plugins() {
    let pg = PLUGINS.read();
    tokio::task::spawn(async move {
        let pg = pg.await;
        let pg = pg.as_ref().unwrap();
        pg.load().await;
    });
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

pub const IS_DEV: bool = if cfg!(debug_assertions) { true } else { false };

#[tokio::main]
async fn main() {
    startup_zip_check().await;
    tauri_plugin_deep_link::prepare("Cider");

    let builder = tauri::Builder::default()
      .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
        println!("{}, {argv:?}, {cwd}", app.package_info().name);
        app.emit_all("single-instance", Payload { args: argv, cwd }).unwrap();
        let win = app.get_window("cider_main").expect("NO WINDOW");
        app.tray_handle().get_item("hide").set_title("Minimize to Tray").unwrap();
        win.show().expect("UNABLE TO SHOW WINDOW");
        win.set_focus().expect("UNABLE TO SET FOCUS");
    }))
    .plugin(discord::init())
    .plugin(lastfm::init())
    .plugin(airplay::init())
    .plugin(rpc::init())
    .plugin(config::init())
    .plugin(vibrancy::init())
    .plugin(ws::init())
    .plugin(steam::init())
    .setup(|app| {
      let local_app_data_dir = app.handle()
        .path_resolver()
        .app_config_dir()
        .expect("No Local Data Directory");
      let file = local_app_data_dir.join("plugins");

      let runtime_env = if IS_DEV { "development" } else { "production" };

      // Setup deep link
      let h = app.handle();

      let mut urls = ["cider"/*, "sabiiro", "itms", "itmss", "music", "musics" */];
      for url in urls.iter_mut() {
        println!("Registering {}", url);
        let deeplink_handle = app.handle();
        let copy_dir = local_app_data_dir.clone();
        match tauri_plugin_deep_link::register(
          url,
          move |request| {
            let window = deeplink_handle.get_window("cider_main").unwrap();
            let command = request.split("://").collect::<Vec<&str>>()[1];
            if command.to_ascii_lowercase().contains("openappdata") {
              open::that(copy_dir.to_str().unwrap()).ok();
            } else {
              window.eval(format!("CiderApp.handleProtocolURL('{}')", command).as_str()).unwrap();
            }
          },
        )
        {
          Ok(_) => println!("Registered {}", url),
          Err(err) => println!("Failed to register {}: {}", url, err)
        }
      }

      #[cfg(windows)]
      std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-web-security --disable-site-isolation-trials --allow-file-access-from-files --js-flags=--expose-gc --autoplay-policy=no-user-gesture-required --enable-features=enable_same_site/true,msRefreshRateBoostOnScroll,msEnhancedTextContrast,MsControlsFluentStyle,msEdgeFluentOverlayScrollbar,MsEdgeFluentOverlayScrollbar,EdgeOverlayScrollbarsWinStyle,OverlayScrollbar,msOverlayScrollbarWinStyle:scrollbar_mode/full_mode,msOverlayScrollbarWinStyleAnimation --ignore-gpu-blocklist --enable-gpu-rasterization --disable-http2");


      let mut window_builder = WindowBuilder::new(app, "cider_main", WindowUrl::App("/index.html".into()))
        .inner_size(1380f64, 730f64).min_inner_size(480f64, 365f64).decorations(false)
        .initialization_script("
        window[\"process\"] = [];
        window[\"process\"][\"versions\"] = [];
        window[\"process\"][\"versions\"][\"node\"] = null;
        document.addEventListener(\'DOMContentLoaded\', function() {
          waitForElm(\'#main-page-container\').then((elm) => {
              window.__TAURI__.invoke(\"init_plugins\", {});
          });
        }, false);


        // stole from stack overflow :)
        function waitForElm(selector) {
          return new Promise(resolve => {
            if (document.querySelector(selector)) {
              return resolve(document.querySelector(selector));
            }

          const observer = new MutationObserver(mutations => {
            if (document.querySelector(selector)) {
                observer.disconnect();
                resolve(document.querySelector(selector));
            }
          });

          observer.observe(document.body, {
            childList: true,
            subtree: true
          });
        });
        }
        ")
        .title("Cider").user_agent(ua.as_str()).theme(Some(Theme::Dark))
        .disable_file_drop_handler();

      #[cfg(windows)]
      if vibrancy::windows::is_at_least_build(22000) {
        window_builder = window_builder.transparent(true);
      }

      let window = window_builder.build().unwrap();
      tauri::async_runtime::spawn_blocking(move || ziphttp::start_ziphttp(h));

      // apply DwmExtendFrameIntoClientArea
      #[cfg(windows)]
      unsafe {
        let hwnd = window.hwnd().unwrap();

        let margin_amount = 1i32;
        let margins = MARGINS { cxLeftWidth: margin_amount, cxRightWidth: margin_amount, cyTopHeight: margin_amount, cyBottomHeight: margin_amount };
        DwmExtendFrameIntoClientArea(hwnd, &margins).unwrap();

        // get rid of 0x80000 for WS_SYSMENU
        let mut style = GetWindowLongA(window.hwnd().unwrap(), WINDOW_LONG_PTR_INDEX(-16));
        style &= !0x80000;
        SetWindowLongA(hwnd, WINDOW_LONG_PTR_INDEX(-16), style);

      }

      //window.set_size(window.outer_size().unwrap()).unwrap();

      #[cfg(target_os = "windows")]
      window.with_webview(|wv| unsafe { platform::webview_stuff(wv)}).expect("TODO: panic message");

      #[cfg(target_os = "macos")]
      window.with_webview(|webview| {
      }).expect("TODO: panic message");

      // Load plugins
      let loader = plugin::new(file.to_str().unwrap(), app.handle());
      tokio::task::spawn(async move {
          *PLUGINS.write().await = Some(loader);
      });


      Ok(())
    })
    .system_tray(systemtray::init())
    .on_system_tray_event(systemtray::system_tray_event_handle)
    .invoke_handler(tauri::generate_handler![set_itspod, init_plugins, systemtray::play, systemtray::pause, systemtray::change_song, additional::set_miniplayer_mode, updater::get_zip_update]);

    let mut context: Context<EmbeddedAssets> = tauri::generate_context!();
    let mut should_zip = false;
    let use_zip = zipdist::verify_protocol_version();

    let port = 10768;

    if use_zip && !IS_DEV {
        should_zip = true;
    }

    let url = format!("http://localhost:{}", port).parse().unwrap();
    let window_url = WindowUrl::External(url);

    // rewrite the config so the IPC is enabled on this URL

    if !IS_DEV {
        context.config_mut().build.dist_dir = AppUrl::Url(window_url.clone());
        context.config_mut().build.dev_path = AppUrl::Url(window_url.clone());
    }

    if should_zip {
        println!("Loading from ZIP");
        builder
            .run(context)
            .expect("error while running tauri application");
    } else {
        // we aren't loading from a zip so we use tauri plugin localhost to serve the files so that the users login session is preserved
        builder
            .plugin(tauri_plugin_localhost::Builder::new(port).build())
            .run(context)
            .expect("error while running tauri application");
    }
}
