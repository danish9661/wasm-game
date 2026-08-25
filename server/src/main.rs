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

use game::sim::{SaveData, Simulation, SimSnapshot};
use protocol::{ClientMsg, ServerMsg};
use std::path::PathBuf;

const TICK_RATE: u32 = 30;
const DT: f32 = 1.0 / TICK_RATE as f32;

type Clients = Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<ServerMsg>>>>;

/// One co-op session. Each room has its own authoritative simulation and its
/// own set of connected clients; snapshots are only broadcast within the room.
struct Room {
    sim: Mutex<Simulation>,
    clients: Clients,
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

    // One broadcast task that ticks every room independently.
    let shared_broadcast = shared.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f64(1.0 / TICK_RATE as f64));
        loop {
            interval.tick().await;
            let rooms = shared_broadcast.rooms.lock().await;
            for room in rooms.values() {
                let snap: SimSnapshot = {
                    let mut sim = room.sim.lock().await;
                    sim.step(DT);
                    sim.snapshot()
                };
                let clients = room.clients.lock().await;
                for (_, tx) in clients.iter() {
                    let _ = tx.unbounded_send(ServerMsg::Snapshot(snap.clone()));
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
            let text = match serde_json::to_string(&msg) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if sink.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    let mut room_code: Option<String> = None;
    let mut player_id: Option<u32> = None;
    while let Some(Ok(msg)) = source.next().await {
        match msg {
            Message::Text(t) => {
                let client: ClientMsg = match serde_json::from_str(&t) {
                    Ok(c) => c,
                    Err(_) => continue,
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
            Message::Close(_) => break,
            _ => {}
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
