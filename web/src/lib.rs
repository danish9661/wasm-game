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
    let Some(window) = window() else {
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
        match App::new(canvas).await {
            Ok(app) => {
                clog("[wasm] webgpu initialized");
                APP.with(|cell| *cell.borrow_mut() = Some(app));
                install_input_handlers(window, document);
            }
            Err(msg) => {
                clog(&format!("[wasm] init failed: {msg}"));
                show_fallback(&format!("WebGPU unavailable: {}", msg));
            }
        }
    });
}

/// Called by JS every frame. JS owns the loop; wasm just steps.
#[wasm_bindgen]
pub fn step(dt_seconds: f64) {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            let dt = (dt_seconds as f32).clamp(0.0, 0.1);
            app.update(dt);
            app.render();
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

/// Recompute canvas size on window resize.
#[wasm_bindgen]
pub fn resize() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.resize();
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

/// Queue a GPU→CPU readback of the next rendered frame.
#[wasm_bindgen]
pub fn trigger_readback() {
    APP.with(|cell| {
        if let Some(app) = cell.borrow_mut().as_mut() {
            app.trigger_readback();
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