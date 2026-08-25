use std::cell::RefCell;
use std::rc::Rc;

use game::sim::{ClientMsg, PlayerInput, ServerMsg, SimSnapshot};
use js_sys::Function;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

/// Thin WebSocket client for the Starfall multiplayer server. It sends the
/// local player's `PlayerInput` each frame and keeps the latest world
/// `SimSnapshot` received from the server (server is authoritative).
pub struct NetClient {
    ws: WebSocket,
    latest: Rc<RefCell<Option<SimSnapshot>>>,
    id: Rc<RefCell<Option<u32>>>,
}

impl NetClient {
    pub fn connect(url: &str) -> Result<NetClient, JsValue> {
        let ws = WebSocket::new(url)?;
        let latest = Rc::new(RefCell::new(None));
        let id = Rc::new(RefCell::new(None));

        let latest_cb = latest.clone();
        let id_cb = id.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(s) = e.data().as_string() {
                if let Ok(msg) = serde_json::from_str::<ServerMsg>(&s) {
                    match msg {
                        ServerMsg::Welcome { player_id, .. } => {
                            *id_cb.borrow_mut() = Some(player_id);
                        }
                        ServerMsg::Snapshot(snap) => {
                            *latest_cb.borrow_mut() = Some(snap);
                        }
                        _ => {}
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref::<Function>()));
        on_message.forget();

        Ok(NetClient { ws, latest, id })
    }

    pub fn send_input(&self, input: &PlayerInput) {
        if let Ok(t) = serde_json::to_string(&ClientMsg::Input(*input)) {
            let _ = self.ws.send_with_str(&t);
        }
    }

    /// Take the most recent snapshot (consuming it) so we only render latest.
    pub fn take_latest(&self) -> Option<SimSnapshot> {
        self.latest.borrow_mut().take()
    }

    pub fn id(&self) -> Option<u32> {
        *self.id.borrow()
    }

    pub fn connected(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}
