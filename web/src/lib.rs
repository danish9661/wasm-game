mod renderer;
mod save;

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

/// Machine-readable stats for the JS HUD / test harness.
#[wasm_bindgen]
pub fn get_stats() -> String {
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => app.stats_line(),
        None => String::new(),
    })
}

/// JSON UI data for the Inventory & Crafting / Build panel.
#[wasm_bindgen]
pub fn get_ui_data() -> String {
    APP.with(|cell| match cell.borrow().as_ref() {
        Some(app) => app.ui_data(),
        None => String::from("{}"),
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