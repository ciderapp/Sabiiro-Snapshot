use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::Serialize;
use std::{collections::LinkedList, convert::Infallible, sync::Arc};
use tauri::{
    async_runtime::{JoinHandle, Mutex, RwLock},
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};
use warp::{
    filters::ws::{Message, WebSocket},
    http::StatusCode,
    Filter,
};

lazy_static::lazy_static! {
    static ref WS_CLIENTS: Arc<RwLock<LinkedList<SplitSink<WebSocket, Message>>>> = Arc::new(RwLock::new(LinkedList::new()));
}

pub struct WebSocketState {
    pub server_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

const PORT_NUMBER: u16 = 10766u16;

// example error response
#[derive(Serialize, Debug)]
struct ApiErrorResult {
    detail: String,
}

async fn store_ws_sender(ws: warp::ws::WebSocket) {
    let (sender, _) = ws.split();
    WS_CLIENTS.write().await.push_front(sender);
}

async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl warp::reply::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not Allowed";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ApiErrorResult {
        detail: message.into(),
    });
    Ok(warp::reply::with_status(json, code))
}

#[tauri::command]
pub fn start_server(ws_state: State<'_, WebSocketState>) -> u16 {
    let health_check = warp::path("health-check").map(|| "OK".to_string());

    let ws = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(store_ws_sender));

    let routes = health_check
        .or(ws)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);

    let thread_ref = ws_state.server_thread.clone();

    tokio::task::block_in_place(move || {
        *thread_ref.blocking_lock() = Some(tauri::async_runtime::spawn(
            warp::serve(routes).run(([127, 0, 0, 1], PORT_NUMBER)),
        ));
    });

    PORT_NUMBER
}

#[tauri::command]
pub fn stop_server(ws_state: State<'_, WebSocketState>) {
    tokio::task::block_in_place(move || {
        let mut ws_state_lock = ws_state.server_thread.blocking_lock();
        if let Some(handle) = ws_state_lock.as_ref() {
            // literally delete the server
            // this is not a clean exit
            // but it's an exit
            handle.abort();
        }
        *ws_state_lock = None;
        WS_CLIENTS.blocking_write().clear();
    });
}

#[tauri::command]
pub async fn send_message(message: String) {
    for i in WS_CLIENTS.write().await.iter_mut() {
        let _ = i.send(Message::text(message.clone())).await;
    }
}

pub fn init<R>() -> TauriPlugin<R>
where
    R: Runtime,
{
    PluginBuilder::new("ws")
        .setup(|app| {
            app.manage(WebSocketState {
                server_thread: Arc::new(Mutex::new(None)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            send_message
        ])
        .build()
}
