use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::{fs::File, io::BufReader};

use tauri::{AppHandle, Manager, Runtime};

pub struct Plugin {}

#[derive(serde::Serialize, serde::Deserialize)]
struct Metadata {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    frontend_main_script: Option<String>,
    backend_main_script: Option<String>,
}

// TODO(freehelpdesk) - remove the _ when this hashmap is actually used
pub struct Plugins<R: Runtime> {
    handle: AppHandle<R>,
    path: String,
    _hashmap: HashMap<String, Box<Plugin>>,
}

pub fn new<R: Runtime>(path: &str, handle: AppHandle<R>) -> Plugins<R> {
    // Create our hash map
    let map: HashMap<String, Box<Plugin>> = HashMap::new();
    Plugins {
        handle,
        path: path.to_string(),
        _hashmap: map,
    }
}

impl<R: Runtime> Plugins<R> {
    pub async fn load(&self) {
        fs::create_dir_all(&self.path).unwrap();
        for entry in fs::read_dir(&self.path).unwrap() {
            let path = entry.unwrap().path();
            println!("scanning {}", &path.to_str().unwrap());

            // Try and find the metadata in the folder
            let metadata_path = path.join("metadata.json");
            let file = match File::open(&metadata_path) {
                Ok(f) => f,
                Err(_) => {
                    println!(
                        "Unable to load plugin, metadata.json not found at {}",
                        metadata_path.clone().to_str().unwrap()
                    );
                    continue;
                }
            };
            let reader = BufReader::new(&file);
            let metadata: Metadata = match serde_json::from_reader(reader) {
                Ok(m) => m,
                Err(_) => {
                    println!(
                        "Unable to load plugin, was unable to parse metadata for {}",
                        metadata_path.to_str().unwrap()
                    );
                    continue;
                }
            };
            println!(
                "Got plugin: {}:{} by {}",
                metadata.name,
                metadata.version,
                metadata.authors.join(", ")
            );

            if let Some(frontend_path) = metadata.frontend_main_script {
                let frontend_path = path.join(frontend_path);
                let mut frontend = match File::open(&frontend_path) {
                    Ok(f) => f,
                    Err(_) => {
                        println!(
                            "Unable to load {}, {} not found",
                            &metadata.name,
                            &frontend_path.to_str().unwrap()
                        );
                        continue;
                    }
                };
                let mut reader = BufReader::new(&mut frontend);
                let mut contents = String::new();
                reader
                    .read_to_string(&mut contents)
                    .expect("Unable to read config buffer");

                self.handle
                    .get_window("cider_main")
                    .unwrap()
                    .eval(contents.as_str())
                    .unwrap();
            }
        }
    }
}
