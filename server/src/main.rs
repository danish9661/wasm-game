mod protocol;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use game::sim::{
    FULL_SNAPSHOT_INTERVAL, PROTOCOL_VERSION, SaveData, Simulation, SimSnapshot, decode_client_bin,
};
use protocol::{ClientMsg, ServerMsg};
use std::path::PathBuf;

const TICK_RATE: u32 = 30;
const DT: f32 = 1.0 / TICK_RATE as f32;

type Clients = Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<ServerMsg>>>>;

/// Per-connection send state: what we last sent (for delta compression) and
/// when we last sent a full snapshot (for periodic full refreshes).
#[derive(Default)]
struct SenderState {
    last: Option<SimSnapshot>,
    last_full_tick: u32,
}

/// One co-op session. Each room has its own authoritative simulation and its
/// own set of connected clients; snapshots are only broadcast within the room.
struct Room {
    sim: Mutex<Simulation>,
    clients: Clients,
    /// Delta-compression state per player id (M2) — the last (culled)
    /// snapshot we sent them, so the next tick can be a small delta.
    sender: Mutex<HashMap<u32, SenderState>>,
    seed: i32,
}

fn save_path(token: &str) -> PathBuf {
    PathBuf::from("saves").join(format!("{token}.json"))
}

struct Shared {
    rooms: Mutex<HashMap<String, Room>>,
    default_seed: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let default_seed: i32 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337);

    let shared = Arc::new(Shared {
        rooms: Mutex::new(HashMap::new()),
        default_seed,
    });

    // One broadcast task that ticks every room independently. Each tick steps
    // the authoritative sim once, then sends every client its own message:
    // a per-client interest-culled (M4) full snapshot, or a small delta (M2)
    // against what that client last received. Full snapshots refresh
    // periodically so a dropped delta can never desync a client for long.
    let shared_broadcast = shared.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f64(1.0 / TICK_RATE as f64));
        loop {
            interval.tick().await;
            let rooms = shared_broadcast.rooms.lock().await;
            for room in rooms.values() {
                let mut sim = room.sim.lock().await;
                sim.step(DT);
                let clients = room.clients.lock().await;
                let mut sender = room.sender.lock().await;
                for (id, tx) in clients.iter() {
                    // Interest management: only entities near this player.
                    let snap = sim.snapshot_for(*id);
                    let st = sender.entry(*id).or_default();
                    let tick = snap.tick;
                    let need_full = st.last.is_none()
                        || tick.wrapping_sub(st.last_full_tick) >= FULL_SNAPSHOT_INTERVAL;
                    let msg = if need_full {
                        st.last_full_tick = tick;
                        ServerMsg::Snapshot(snap.clone())
                    } else {
                        ServerMsg::Delta(snap.delta_from(st.last.as_ref()))
                    };
                    st.last = Some(snap);
                    let _ = tx.unbounded_send(msg);
                }
            }
        }
    });

    let addr = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!("[server] listening on ws://{addr} (default_seed={default_seed}, tick={TICK_RATE}Hz)");

    let mut conn_id = 0u32;
    while let Ok((stream, _)) = listener.accept().await {
        conn_id += 1;
        tokio::spawn(handle_conn(stream, shared.clone(), conn_id));
    }
    Ok(())
}

async fn handle_conn(stream: tokio::net::TcpStream, shared: Arc<Shared>, conn_id: u32) {
    let ws = match tokio_tungstenite::accept_async(stream).await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[server] conn {conn_id} ws accept failed: {e}");
            return;
        }
    };
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = mpsc::unbounded::<ServerMsg>();

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            // Binary bincode frames (M2): ~3-5x smaller than JSON text. Old
            // clients that only read Text frames must upgrade.
            let bytes = game::sim::encode_server(&msg);
            if sink.send(Message::Binary(bytes)).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    let mut room_code: Option<String> = None;
    let mut player_id: Option<u32> = None;
    while let Some(Ok(msg)) = source.next().await {
        // Accept both wire formats: Binary (bincode, new clients) and Text
        // (JSON, legacy clients) so a mixed-version room still plays.
        let client: Option<ClientMsg> = match msg {
            Message::Binary(b) => decode_client_bin(&b),
            Message::Text(t) => serde_json::from_str(&t).ok(),
            Message::Close(_) => break,
            _ => continue,
        };
        let client = match client {
            Some(c) => c,
            None => continue,
        };
                match client {
                    ClientMsg::Join { name, token, room } => {
                        let code = if room.trim().is_empty() {
                            format!("R{:06}", conn_id)
                        } else {
                            room.trim().to_string()
                        };
                        // Create the room (with its own sim) on first join.
                        let mut rooms = shared.rooms.lock().await;
                        let entry = rooms.entry(code.clone()).or_insert_with(|| Room {
                            sim: Mutex::new(Simulation::new(shared.default_seed as u32)),
                            clients: Arc::new(Mutex::new(HashMap::new())),
                            sender: Mutex::new(HashMap::new()),
                            seed: shared.default_seed,
                        });
                        let id = entry
                            .sim
                            .lock()
                            .await
                            .add_player(name.clone(), token.clone());
                        entry.clients.lock().await.insert(id, tx.clone());
                        drop(rooms);
                        room_code = Some(code.clone());
                        player_id = Some(id);
                        // Cross-device save: restore any prior progress.
                        if let Some(tok) = &token {
                            let path = save_path(tok);
                            if let Ok(bytes) = std::fs::read(&path) {
                                if let Ok(save) = serde_json::from_slice::<SaveData>(&bytes) {
                                    shared
                                        .rooms
                                        .lock()
                                        .await
                                        .get(&code)
                                        .unwrap()
                                        .sim
                                        .lock()
                                        .await
                                        .restore_player(id, &save);
                                    println!("[server] player {id} restored save '{tok}'");
                                }
                            }
                        }
                        let welcome = ServerMsg::Welcome {
                            player_id: id,
                            tick_rate: TICK_RATE,
                            seed: shared.default_seed as u32,
                            protocol: PROTOCOL_VERSION,
                        };
                        let _ = tx.unbounded_send(welcome);
                        println!("[server] player {id} joined room '{code}'");
                    }
                    ClientMsg::Input(input) => {
                        if let (Some(code), Some(id)) = (&room_code, player_id) {
                            if let Some(room) = shared.rooms.lock().await.get(&code.clone()) {
                                room.sim.lock().await.set_input(id, input);
                            }
                        }
                    }
                    ClientMsg::Leave => break,
                }
    }

    // Persist + clean up on disconnect.
    if let (Some(code), Some(id)) = (&room_code, player_id) {
        let mut sim_guard = shared.rooms.lock().await;
        if let Some(room) = sim_guard.get(&code.clone()) {
            let mut sim = room.sim.lock().await;
            if let Some(tok) = sim.token_of(id) {
                if let Some(save) = sim.save_player(id) {
                    let _ = std::fs::create_dir_all("saves");
                    let _ = std::fs::write(
                        save_path(&tok),
                        serde_json::to_vec(&save).unwrap_or_default(),
                    );
                }
            }
            sim.remove_player(id);
            // Prune the connection (previously leaked: the id stayed in the
            // map forever, so rooms never emptied and never closed).
            room.clients.lock().await.remove(&id);
            room.sender.lock().await.remove(&id);
        }
        // Drop the room entirely once empty so it can be recreated fresh later.
        if let Some(room) = sim_guard.get(&code.clone()) {
            if room.clients.lock().await.is_empty() {
                sim_guard.remove(&code.clone());
                println!("[server] room '{code}' closed (empty)");
            }
        }
        println!("[server] player {id} left room '{code}'");
    }
    write_task.abort();
}
