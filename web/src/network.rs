use std::cell::RefCell;
use std::rc::Rc;

use game::sim::{ClientMsg, PlayerInput, ServerMsg, SimSnapshot, decode_server_bin, encode_client};
use js_sys::Function;
use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, WebSocket};

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

/// Decode one incoming WebSocket frame in either wire format: Binary (bincode,
/// new servers) or text (JSON, legacy servers / mixed-version rooms).
fn decode_frame(data: &JsValue) -> Option<ServerMsg> {
    if let Some(s) = data.as_string() {
        return serde_json::from_str::<ServerMsg>(&s).ok();
    }
    if let Ok(buf) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
        let bytes = js_sys::Uint8Array::new(&buf).to_vec();
        return decode_server_bin(&bytes);
    }
    // Some browsers deliver binary frames as a Uint8Array view directly.
    if js_sys::ArrayBuffer::is_view(data) {
        let view = js_sys::Uint8Array::new(data);
        return decode_server_bin(&view.to_vec());
    }
    None
}

/// Thin WebSocket client for the Starfall multiplayer server. It sends the
/// local player's `PlayerInput` each frame (bincode `Binary` frames) and keeps
/// the latest world `SimSnapshot` received from the server (server is
/// authoritative). Full snapshots replace the base; `Delta` frames are merged
/// onto the base (see `SimSnapshot::apply_delta`). Remote entity positions are
/// interpolated between the previous and current snapshots in `sample()` so
/// movement looks continuous.
pub struct NetClient {
    ws: WebSocket,
    /// Merge base: last full snapshot with all received deltas applied.
    base: Rc<RefCell<Option<SimSnapshot>>>,
    /// Most recent snapshot + the wall-clock time it arrived.
    curr: Rc<RefCell<Option<(SimSnapshot, f64)>>>,
    /// The snapshot before `curr`, for interpolation.
    prev: Rc<RefCell<Option<(SimSnapshot, f64)>>>,
    id: Rc<RefCell<Option<u32>>>,
}

impl NetClient {
    pub fn connect(
        url: &str,
        name: &str,
        token: Option<String>,
        room: &str,
    ) -> Result<NetClient, JsValue> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);
        let base = Rc::new(RefCell::new(None));
        let curr = Rc::new(RefCell::new(None));
        let prev = Rc::new(RefCell::new(None));
        let id = Rc::new(RefCell::new(None));

        let base_cb = base.clone();
        let curr_cb = curr.clone();
        let prev_cb = prev.clone();
        let id_cb = id.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            let Some(msg) = decode_frame(&e.data()) else {
                return;
            };
            match msg {
                ServerMsg::Welcome { player_id, protocol, .. } => {
                    *id_cb.borrow_mut() = Some(player_id);
                    if protocol != game::sim::PROTOCOL_VERSION {
                        web_sys::console::log_1(&JsValue::from_str(&format!(
                            "[net] server protocol v{protocol} (client v{}) — mixed-version room, JSON fallback active",
                            game::sim::PROTOCOL_VERSION
                        )));
                    }
                }
                ServerMsg::Snapshot(snap) => {
                    *base_cb.borrow_mut() = Some(snap.clone());
                    *prev_cb.borrow_mut() = curr_cb.borrow_mut().take();
                    *curr_cb.borrow_mut() = Some((snap, now_ms()));
                }
                ServerMsg::Delta(d) => {
                    let mut base_ref = base_cb.borrow_mut();
                    if let Some(base_snap) = base_ref.as_mut() {
                        // Stale or out-of-order deltas are dropped inside
                        // `apply_delta` / by the base-tick guard here.
                        if d.base_tick != base_snap.tick && d.tick <= base_snap.tick {
                            return;
                        }
                        base_snap.apply_delta(d);
                        let merged = base_snap.clone();
                        *prev_cb.borrow_mut() = curr_cb.borrow_mut().take();
                        *curr_cb.borrow_mut() = Some((merged, now_ms()));
                    }
                    // No base yet (delta arrived before the first full
                    // snapshot): wait for the periodic full refresh.
                }
                _ => {}
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref::<Function>()));
        on_message.forget();

        // Join is tiny and must be readable even by an old JSON-only server,
        // so it stays a text frame. It MUST go out in `onopen`: sending
        // synchronously here races the CONNECTING state and the Join is
        // silently dropped (the client then sits in local mode forever).
        let join_text = serde_json::to_string(&ClientMsg::Join {
            name: name.to_string(),
            token,
            room: room.to_string(),
        })
        .unwrap_or_default();
        {
            let ws_open = ws.clone();
            let on_open = Closure::wrap(Box::new(move |_: JsValue| {
                if !join_text.is_empty() {
                    if ws_open.send_with_str(&join_text).is_ok() {
                        web_sys::console::log_1(&JsValue::from_str("[net] Join sent"));
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref::<Function>()));
            on_open.forget();
        }

        Ok(NetClient { ws, base, curr, prev, id })
    }

    pub fn send_input(&self, input: &PlayerInput) {
        // Bincode binary at 30 Hz (~1/4 the bytes of JSON); fall back to a
        // JSON text frame if encoding ever fails so input never stalls.
        let bytes = encode_client(&ClientMsg::Input(*input));
        if !bytes.is_empty() {
            if self.ws.send_with_u8_array(&bytes).is_ok() {
                return;
            }
        }
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
