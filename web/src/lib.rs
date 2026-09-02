mod network;
mod renderer;
mod save;
mod zip;

use renderer::App;
use save::SaveState;
use std::cell::RefCell;
use wasm_bindgen::prelude::{Closure, JsCast, JsValue, wasm_bindgen};
use web_sys::{window, KeyboardEvent};

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn clog(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
    if let Some(w) = web_sys::window() {
        if let Some(d) = w.document() {
            if let Some(el) = d.get_element_by_id("log") {
                let _ = el.set_text_content(Some(msg));
            }
        }
    }
}

/// Console-only log (does NOT touch the #log HUD element, so it never
/// clobbers the live fps/stats readout).
pub fn cinfo(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

fn show_fallback(message: &str) {
    if let Some(window) = window() {
        if let Some(doc) = window.document() {
            if let Some(el) = doc.get_element_by_id("fallback") {
                el.set_text_content(Some(message));
                let _ = el.set_attribute("style", "display:block");
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    cinfo("[wasm] start() entered");
    let Some(window) = window() else {
        cinfo("[wasm] start() abort: no window");
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(canvas) = document
        .get_element_by_id("game")
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    else {
        show_fallback("Canvas #game not found");
        return;
    };

    wasm_bindgen_futures::spawn_local(async move {
        cinfo("[wasm] start() requesting WebGPU adapter/device…");
        match App::new(canvas).await {
            Ok(app) => {
                clog("[wasm] webgpu initialized");
                cinfo("[wasm] webgpu initialized (App::new ok)");
                APP.with(|cell| *cell.borrow_mut() = Some(app));
                install_input_handlers(window, document);
            }
            Err(msg) => {
                clog(&format!("[wasm] init failed: {msg}"));
                cinfo(&format!("[wasm] init failed: {msg}"));
                show_fallback(&format!("WebGPU unavailable: {}", msg));
            }
        }
    });
}

/// Called by JS every frame. JS owns the loop; wasm just steps.
#[wasm_bindgen]
pub fn step(dt_seconds: f64) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed) + 1;
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            let dt = (dt_seconds as f32).clamp(0.0, 0.1);
            app.update(dt);
            app.render();
        } else if n == 1 {
            cinfo("[wasm] step() called but App not ready yet (WebGPU init still pending)");
        }
    });
}

/// True once the WebGPU App has finished initializing (so calls like
/// `resize()` won't be silently ignored). Used by the page to apply the
/// resolution cap exactly once the render target exists.
#[wasm_bindgen]
pub fn app_ready() -> bool {
    APP.with(|c| c.borrow().is_some())
}

/// Machine-readable stats for the JS HUD / test harness.
#[wasm_bindgen]
pub fn get_stats() -> String {
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => app.stats_line(),
        None => String::new(),
    })
}

/// Headless "visual engine" frame dump (JSON). Lets a non-multimodal agent
/// render the scene as ASCII and assert on layout/animation without a screenshot.
#[wasm_bindgen]
pub fn get_frame_dump() -> String {
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => app.frame_dump(),
        None => String::from("{}"),
    })
}

/// Trigger a melee swing (drives the player attack-lunge animation) for tests.
#[wasm_bindgen]
pub fn do_attack() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.attack();
        }
    });
}

/// JSON UI data for the Inventory & Crafting / Build panel.
#[wasm_bindgen]
pub fn get_ui_data() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.ui_data(),
        None => String::from("{}"),
    })
}

/// Cheap per-frame co-op name-tag positions (id + screen x/y) for the HUD.
#[wasm_bindgen]
pub fn get_coop_tags() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.coop_tags(),
        None => String::from("[]"),
    })
}

/// Cheap per-frame portal-transition status (loading overlay + build state).
#[wasm_bindgen]
pub fn get_town_status() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.town_status(),
        None => String::from("{\"transition\":false,\"build\":false,\"name\":\"\",\"progress\":1.0}"),
    })
}

/// Set the cursor position in internal canvas pixels (drives the build ghost).
#[wasm_bindgen]
pub fn set_mouse(x: f64, y: f64) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.set_mouse(x as f32, y as f32);
        }
    });
}

/// Toggle build mode (true = enter with the first buildable selected).
#[wasm_bindgen]
pub fn set_build_mode(on: bool) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.set_build_mode(on);
        }
    });
}

/// Virtual-joystick input for touch devices. `(x, y)` is the raw drag offset in
/// pixels from the stick origin; `(0, 0)` clears analog control.
#[wasm_bindgen]
pub fn set_analog(x: f64, y: f64) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.set_analog(x as f32, y as f32);
        }
    });
}

/// Select a buildable structure by its index in the build menu (0..7).
#[wasm_bindgen]
pub fn select_build(idx: usize) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.select_build(idx);
        }
    });
}

/// Place the currently-selected build structure at the cursor ghost tile.
#[wasm_bindgen]
pub fn place_selected() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.place_selected();
        }
    });
}

/// Craft recipe `idx` at an Anvil. Returns false if no anvil / unaffordable.
#[wasm_bindgen]
pub fn craft(idx: usize) -> bool {
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => app.craft(idx),
        None => false,
    })
}

/// JSON minimap data centered on the player (terrain grid + markers).
#[wasm_bindgen]
pub fn get_minimap() -> String {
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => app.minimap_data(),
        None => String::from("{}"),
    })
}

/// Current biome under the player (e.g. "Forest"). Used by the page to pick a
/// gentle ambient sound bed.
#[wasm_bindgen]
pub fn current_biome() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.biome_name(),
        None => String::from("Grass"),
    })
}

/// Bestiary / Codex JSON: discovered enemies with stats + behaviour.
#[wasm_bindgen]
pub fn get_codex() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.codex(),
        None => String::from("{}"),
    })
}

/// Ask the renderer to grab a screenshot on its next frame (readback + #blit).
#[wasm_bindgen]
pub fn capture() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.request_capture();
        }
    });
}

/// Recompute canvas size on window resize.
#[wasm_bindgen]
pub fn resize() {
    cinfo("[wasm] resize() called");
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.resize();
        } else {
            cinfo("[wasm] resize() ignored: App not ready");
        }
    });
}

/// Reforge the Crown. choice: 0 = Reign (victory), 1 = Shatter (New Game+).
#[wasm_bindgen]
pub fn reforge(choice: u8) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.reforge(choice);
        }
    });
}

/// Serialize the current run to a JSON string (caller persists it).
#[wasm_bindgen]
pub fn serialize_save() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => serde_json::to_string(&app.to_save()).unwrap_or_else(|_| String::from("{}")),
        None => String::from("{}"),
    })
}

/// Restore a run from a JSON string produced by `serialize_save`.
#[wasm_bindgen]
pub fn deserialize_save(json: &str) -> bool {
    match serde_json::from_str::<SaveState>(json) {
        Ok(s) => {
            // Reject saves from a different format version so an old save
            // can never silently corrupt a new build.
            if s.version != crate::save::CURRENT_SAVE_VERSION {
                return false;
            }
            APP.with(|cell| {
                if let Some(app) = cell.borrow_mut().as_mut() {
                    app.apply_save(&s);
                }
            });
            true
        }
        Err(_) => false,
    }
}

/// Start a fresh run at the base seed (Save/Load "New Game").
#[wasm_bindgen]
pub fn new_game() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.new_game();
        }
    });
}

/// Bundle the current run into a downloadable `.zip` containing `save.json`.
/// Returns the raw zip bytes (JS turns them into a Blob download).
#[wasm_bindgen]
pub fn download_save_zip() -> Vec<u8> {
    let json = APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => serde_json::to_string(&app.to_save()).unwrap_or_else(|_| String::from("{}")),
        None => String::from("{}"),
    });
    zip::make_zip(&[("save.json", json.as_bytes())])
}

/// Load a `.zip` produced by `download_save_zip`. Extracts `save.json` and
/// applies it. Returns false if the archive/file is invalid or the version
/// mismatches.
#[wasm_bindgen]
pub fn upload_save_zip(bytes: &[u8]) -> bool {
    let s = match zip::read_zip_file(bytes, "save.json") {
        Some(b) => match String::from_utf8(b) {
            Ok(s) => s,
            Err(_) => return false,
        },
        None => return false,
    };
    match serde_json::from_str::<SaveState>(&s) {
        Ok(save) => {
            if save.version != crate::save::CURRENT_SAVE_VERSION {
                return false;
            }
            APP.with(|cell| {
                if let Some(app) = cell.borrow_mut().as_mut() {
                    app.apply_save(&save);
                }
            });
            true
        }
        Err(_) => false,
    }
}


/// Start a fresh run at a specific world seed (player-entered).
#[wasm_bindgen]
pub fn new_game_with_seed(seed: u32) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.new_game_with_seed(seed);
        }
    });
}

/// Queue a GPU→CPU readback of the next rendered frame.
#[wasm_bindgen]
pub fn trigger_readback() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.trigger_readback();
        }
    });
}

/// Set the internal render/readback resolution cap. (w, h) = (0, 0) means native
/// (no cap). Used by the settings menu. Call `resize()` afterwards to apply.
#[wasm_bindgen]
pub fn set_render_cap(w: u32, h: u32) {
    crate::renderer::set_render_cap(w, h);
}

/// Enable/disable fps-driven adaptive resolution. Used by the settings menu.
#[wasm_bindgen]
pub fn set_adaptive_res(v: bool) {
    crate::renderer::set_adaptive_res(v);
}

#[wasm_bindgen]
pub fn zoom_step(delta: f32) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.zoom_step(delta);
        }
    });
}

/// Result of the last completed readback ("pending" until the map callback runs).
#[wasm_bindgen]
pub fn readback_stats() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => {
            let s = app.readback_stats();
            if s.is_empty() {
                String::from("pending")
            } else {
                s
            }
        }
        None => String::from("no app"),
    })
}

fn install_input_handlers(window: web_sys::Window, document: web_sys::Document) {
    let on_key = {
        Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(|e: KeyboardEvent| {
            APP.with(|cell| {
                if let Some(app) = cell.borrow_mut().as_mut() {
                    app.set_key(e.code().as_str(), true);
                }
            });
            e.prevent_default();
        }))
    };
    document
        .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())
        .ok();
    on_key.forget();

    let off_key = {
        Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(|e: KeyboardEvent| {
            APP.with(|cell| {
                if let Some(app) = cell.borrow_mut().as_mut() {
                    app.set_key(e.code().as_str(), false);
                }
            });
        }))
    };
    document
        .add_event_listener_with_callback("keyup", off_key.as_ref().unchecked_ref())
        .ok();
    off_key.forget();

    let on_resize = {
        Closure::<dyn FnMut()>::wrap(Box::new(|| {
            resize();
        }))
    };
    window
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .ok();
    on_resize.forget();
}