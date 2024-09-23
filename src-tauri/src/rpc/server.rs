use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use crate::{
    bridge::Bridge,
    lastfm::{AuthState, LastFm},
};

use serde_json::Value;

use tauri::{AppHandle, Manager, Runtime};

use rouille::{Response, Server};

type ChannelChan = (Arc<Mutex<Sender<Value>>>, Arc<Mutex<Receiver<Value>>>);

lazy_static::lazy_static! {
  pub static ref CHANNELS: ChannelChan = {
      let (sender, receiver): (Sender<Value>, Receiver<Value>) = channel();
      (Arc::new(Mutex::new(sender)), Arc::new(Mutex::new(receiver)))
  };
}

pub fn create_rpc_server<R>(
    handle: AppHandle<R>,
    port: u16,
) -> Server<impl Fn(&rouille::Request) -> Response>
where
    R: Runtime,
{
    let bridge = Bridge::new(handle.clone());

    let server = Server::new(format!("localhost:{port}"), move |req| {
      rouille::router!(req,

          (GET) (/handleCallbackUrl) => {
              Response::empty_204()
          },

          (GET) (/last_fm_auth_callback) => {
              let state = handle.state::<LastFm>();

              let mut url = req.raw_url().to_string();

              let offset = url.rfind("?token=").expect("INVALID URL");

              url.replace_range(..offset+7, "");

              let s = state
              .inner_client
              .blocking_write()
              .authenticate_with_token(&url);

              match s {
                  Ok(_) => {
                      state.set_auth_state(AuthState::Authorised);
                  }
                  Err(e) => {
                      eprintln!("{:?}", e);
                      return Response::empty_400();
                  }
              }

              Response::empty_204()
          },

          (GET) (/active) => {
              Response::empty_204()
          },

          (GET) (/currentPlayingSong) => {
              #[derive(serde::Deserialize, serde::Serialize)]
              struct Info {
                  info: serde_json::Value
              }

              Response::json(&Info {
                  info: bridge.get_playing_song(),
              }).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (GET) (/addToLibrary) => {
              bridge.add_to_library();
              Response::empty_204()
          },

          (GET) (/isPlaying) => {
              #[derive(serde::Deserialize, serde::Serialize)]
              struct IsPlaying {
                  is_playing: Option<bool>
              }

              Response::json(&IsPlaying {
                  is_playing: bridge.is_playing(),
              }).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (GET) (/toggleAutoplay) => {
              #[derive(serde::Deserialize, serde::Serialize)]
              struct IsAutoplay {
                  autoplay: Option<bool>
              }

              Response::json(&IsAutoplay {
                  autoplay: bridge.toggle_autoplay(),
              }).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (POST) (/toggleShuffle) => {
            Response::json(&bridge.toggle_shuffle()).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (POST) (/toggleRepeat) => {
            Response::json(&bridge.toggle_repeat()).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (GET) (/playPause) => {
              bridge.play_pause();
              Response::empty_204()
          },

          (GET) (/play/{kind: String}/{ids: String}) => {
              let ids: Vec<&str> = ids.split(',').collect();
              bridge.play(Some(kind), Some(&ids));
              Response::empty_204()
          },

          (GET) (/play) => {
              bridge.play(None, None);
              Response::empty_204()
          },

          (GET) (/pause) => {
              bridge.pause();
              Response::empty_204()
          },

          (GET) (/stop) => {
              bridge.stop();
              Response::empty_204()
          },

          (GET) (/next) => {
              bridge.next();
              Response::empty_204()
          },

          (GET) (/previous) => {
              bridge.previous();
              Response::empty_204()
          },

          (GET) (/seekto/{t: u32}) => {
              bridge.seekto(t);
              Response::empty_204()
          },

          (GET) (/show) => {
              bridge.show();
              Response::empty_204()
          },

          (GET) (/hide) => {
              bridge.hide();
              Response::empty_204()
          },

          (GET) (/album/{id: String}) => {
              Response::json(&bridge.album(&id)).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (GET) (/song/{id: String}) => {
              Response::json(&bridge.song(&id)).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (GET) (/audio/{volume: f32}) => {
              bridge.set_audio_volume(volume);
              Response::empty_204()
          },

          (GET) (/audio) => {
              Response::json(&bridge.get_audio_volume()).with_additional_header("Access-Control-Allow-Origin", "*")
          },

          (PUT) (/setRating/{rating: i8}) => {
              bridge.set_rating(rating);
              Response::empty_204()
          },

          (PUT) (/rating/{content_type: String}/{id: u64}/{rating: i8}) => {
              let a = bridge.set_rating_api(content_type, id.to_string(), rating);

              if let Some(value) = a {
                  let mut status = 200u16;

                  if let Some(code) = value.get("code") {
                      if let Some(actual_code) = code.as_u64() {
                          status = actual_code as u16
                      }
                  }

                  Response::json(&value).with_additional_header("Access-Control-Allow-Origin", "*").with_status_code(status)
              } else {
                  // me when the delete request has no return body
                  // kill me
                  if rating != 0 {
                      Response::text("Unable to Set Rating").with_status_code(500)
                  } else {
                      Response::empty_204()
                  }
              }
          },

          (GET) (/rating/{content_type: String}/{id: u64}) => {
              let a = bridge.get_rating(content_type, id.to_string());

              if let Some(value) = a {
                  let mut status = 200u16;

                  if let Some(code) = value.get("code") {
                      if let Some(actual_code) = code.as_u64() {
                          status = actual_code as u16
                      }
                  }

                  Response::json(&value).with_additional_header("Access-Control-Allow-Origin", "*").with_status_code(status)
              } else {
                  Response::text("Unable to Get Rating").with_status_code(500)
              }
          },

          // Add a song ID to the current queue
          // (GET) (/queue/{id: String}) => {
          //     bridge.add_trackid_to_queue(&id);
          //     Response::empty_404()
          // },

          _ => Response::empty_404()
      )
  })
  .expect("RPC Server FAILED to Start");

    server
}
