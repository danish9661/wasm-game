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

fn save_path(token: &str) -> PathBuf {
    PathBuf::from("saves").join(format!("{token}.json"))
}

const TICK_RATE: u32 = 30;
const DT: f32 = 1.0 / TICK_RATE as f32;

type Clients = Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<ServerMsg>>>>;

struct Shared {
    sim: Mutex<Simulation>,
    clients: Clients,
    seed: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let seed: i32 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337);

    let shared = Arc::new(Shared {
        sim: Mutex::new(Simulation::new(seed as u32)),
        clients: Arc::new(Mutex::new(HashMap::new())),
        seed,
    });

    let shared_broadcast = shared.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs_f64(1.0 / TICK_RATE as f64));
        loop {
            interval.tick().await;
            let snap: SimSnapshot = {
                let mut sim = shared_broadcast.sim.lock().await;
                sim.step(DT);
                sim.snapshot()
            };
            let clients = shared_broadcast.clients.lock().await;
            for (_, tx) in clients.iter() {
                let _ = tx.unbounded_send(ServerMsg::Snapshot(snap.clone()));
            }
        }
    });

    let addr = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!("[server] listening on ws://{addr} (seed={seed}, tick={TICK_RATE}Hz)");

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

    let mut player_id: Option<u32> = None;
    while let Some(Ok(msg)) = source.next().await {
        match msg {
            Message::Text(t) => {
                let client: ClientMsg = match serde_json::from_str(&t) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                match client {
                    ClientMsg::Join { name, token } => {
                        let id = shared.sim.lock().await.add_player(name, token.clone());
                        shared
                            .clients
                            .lock()
                            .await
                            .insert(id, tx.clone());
                        player_id = Some(id);
                        // Cross-device save: if the client supplied an account
                        // token, restore any previously persisted progress.
                        if let Some(tok) = &token {
                            let path = save_path(tok);
                            if let Ok(bytes) = std::fs::read(&path) {
                                if let Ok(save) = serde_json::from_slice::<SaveData>(&bytes) {
                                    shared.sim.lock().await.restore_player(id, &save);
                                    println!("[server] player {id} restored save '{tok}'");
                                }
                            }
                        }
                        let welcome = ServerMsg::Welcome {
                            player_id: id,
                            tick_rate: TICK_RATE,
                            seed: shared.seed as u32,
                        };
                        let _ = tx.unbounded_send(welcome);
                        println!("[server] player {id} joined");
                    }
                    ClientMsg::Input(input) => {
                        if let Some(id) = player_id {
                            shared.sim.lock().await.set_input(id, input);
                        }
                    }
                    ClientMsg::Leave => break,
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    if let Some(id) = player_id {
        let mut sim = shared.sim.lock().await;
        // Persist progress for accounts that joined with a token.
        if let Some(tok) = sim.token_of(id) {
            if let Some(save) = sim.save_player(id) {
                let _ = std::fs::create_dir_all("saves");
                let _ = std::fs::write(save_path(&tok), serde_json::to_vec(&save).unwrap_or_default());
            }
        }
        sim.remove_player(id);
        drop(sim);
        shared.clients.lock().await.remove(&id);
        println!("[server] player {id} left");
    }
    write_task.abort();
}
