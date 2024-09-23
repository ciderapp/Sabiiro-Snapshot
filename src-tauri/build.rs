use cfg_if::cfg_if;
use std::{env, fs, path::PathBuf};
cfg_if! {
    if #[cfg(any(windows))] {
        fn main() {

          #[cfg(debug_assertions)]
          let profile = "debug";
          #[cfg(not(debug_assertions))]
          let profile = "release";


          let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

           cfg_if! {
                if #[cfg(all(any(debug_assertions, feature = "steamworks"), windows))] {
                    let from_path = manifest_dir.join("resource/steam_api64.dll");
                    let to_path = manifest_dir.join(format!("target/{}/steam_api64.dll", profile));

                    println!("cargo:rerun-if-changed=resource/steam_api64.dll");
                    println!("cargo:rerun-if-changed=target/{}/steam_api_64.dll", profile);

                    fs::copy(from_path, to_path).expect("FAILED TO COPY STEAM_API DLL");
                }
            }

            cfg_if! {
              if #[cfg(all(not(debug_assertions), windows))] {
                // create a symlink for ../../Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64/ to ./target/debug/Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64/ folder to debug and release
                let from_path = manifest_dir.join("Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64");
                let to_path_debug = manifest_dir.join(format!("target/debug/Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64"));
                let to_path_release = manifest_dir.join(format!("target/release/Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64"));

                println!("cargo:rerun-if-changed=resource/Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64");
                println!("cargo:rerun-if-changed=target/{}/Microsoft.WebView2.FixedVersionRuntime.123.0.2420.81.x64", profile);

                if !to_path_debug.exists() {
                    std::os::windows::fs::symlink_dir(&from_path, to_path_debug).expect("FAILED TO CREATE SYMLINK FOR WEBVIEW2 RUNTIME");
                }

                if !to_path_release.exists() {
                  std::os::windows::fs::symlink_dir(&from_path, to_path_release).expect("FAILED TO CREATE SYMLINK FOR WEBVIEW2 RUNTIME");
              }
              }
            }

            tauri_build::build()
        }
    }
    else {
        fn main() {
            compile_error!("UNSUPPORTED PLATFORM AT THIS TIME");
        }
    }
}
