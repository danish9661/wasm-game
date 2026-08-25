use std::cell::RefCell;
use std::rc::Rc;

use game::sim::{ClientMsg, PlayerInput, ServerMsg, SimSnapshot};
use js_sys::Function;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

/// Render delay (ms) applied when interpolating between snapshots. ~3 ticks at
/// 30 Hz, so we always have a previous frame to blend from and remote motion
/// looks smooth instead of snapping every 33 ms.
const INTERP_DELAY_MS: f64 = 100.0;

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

fn lerp(a: f32, b: f32, t: f64) -> f32 {
    (a as f64 + (b as f64 - a as f64) * t) as f32
}

fn lerp_facing(a: (f32, f32), b: (f32, f32), t: f64) -> (f32, f32) {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t))
}

/// Thin WebSocket client for the Starfall multiplayer server. It sends the
/// local player's `PlayerInput` each frame and keeps the latest world
/// `SimSnapshot` received from the server (server is authoritative). Remote
/// entity positions are interpolated between the previous and current
/// snapshots in `sample()` so movement looks continuous.
pub struct NetClient {
    ws: WebSocket,
    /// Most recent snapshot + the wall-clock time it arrived.
    curr: Rc<RefCell<Option<(SimSnapshot, f64)>>>,
    /// The snapshot before `curr`, for interpolation.
    prev: Rc<RefCell<Option<(SimSnapshot, f64)>>>,
    id: Rc<RefCell<Option<u32>>>,
}

impl NetClient {
    pub fn connect(url: &str, name: &str, token: Option<String>) -> Result<NetClient, JsValue> {
        let ws = WebSocket::new(url)?;
        let curr = Rc::new(RefCell::new(None));
        let prev = Rc::new(RefCell::new(None));
        let id = Rc::new(RefCell::new(None));

        let curr_cb = curr.clone();
        let prev_cb = prev.clone();
        let id_cb = id.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(s) = e.data().as_string() {
                if let Ok(msg) = serde_json::from_str::<ServerMsg>(&s) {
                    match msg {
                        ServerMsg::Welcome { player_id, .. } => {
                            *id_cb.borrow_mut() = Some(player_id);
                        }
                        ServerMsg::Snapshot(snap) => {
                            *prev_cb.borrow_mut() = curr_cb.borrow_mut().take();
                            *curr_cb.borrow_mut() = Some((snap, now_ms()));
                        }
                        _ => {}
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref::<Function>()));
        on_message.forget();

        if let Ok(t) = serde_json::to_string(&ClientMsg::Join {
            name: name.to_string(),
            token,
        }) {
            let _ = ws.send_with_str(&t);
        }

        Ok(NetClient { ws, curr, prev, id })
    }

    pub fn send_input(&self, input: &PlayerInput) {
        if let Ok(t) = serde_json::to_string(&ClientMsg::Input(*input)) {
            let _ = self.ws.send_with_str(&t);
        }
    }

    /// Return an interpolated snapshot for rendering. `None` until we've
    /// received at least one snapshot. Players are blended between the previous
    /// and current snapshots by id; the world (enemies/arrows/structures) is
    /// taken from the latest snapshot (no interpolation yet).
    pub fn sample(&self) -> Option<SimSnapshot> {
        let curr = self.curr.borrow();
        let prev = self.prev.borrow();
        let (c_snap, c_t) = curr.as_ref()?;
        let (p_snap, p_t) = prev.as_ref()?;
        let span = c_t - p_t;
        if span.abs() < 1e-3 {
            return Some(c_snap.clone());
        }
        let now = now_ms();
        let mut t = ((now - c_t) + INTERP_DELAY_MS) / span;
        t = t.clamp(0.0, 1.0);

        let mut snap = c_snap.clone();
        let mut players = Vec::with_capacity(c_snap.players.len());
        for cp in &c_snap.players {
            if let Some(pp) = p_snap.players.iter().find(|pp| pp.id == cp.id) {
                players.push(game::sim::PlayerSnapshot {
                    x: lerp(pp.x, cp.x, t),
                    y: lerp(pp.y, cp.y, t),
                    facing: lerp_facing(pp.facing, cp.facing, t),
                    ..cp.clone()
                });
            } else {
                players.push(cp.clone());
            }
        }
        snap.players = players;
        snap.time_of_day = lerp(p_snap.time_of_day, c_snap.time_of_day, t);
        Some(snap)
    }

    pub fn id(&self) -> Option<u32> {
        *self.id.borrow()
    }

    pub fn connected(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}
