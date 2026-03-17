use crate::run_source;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebRunResult {
    ok: bool,
    output: String,
    error: String,
    canvas_frames_json: String,
    execution_events_json: String,
}

#[wasm_bindgen]
impl WebRunResult {
    #[wasm_bindgen(getter)]
    pub fn ok(&self) -> bool {
        self.ok
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> String {
        self.error.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn canvas_frames_json(&self) -> String {
        self.canvas_frames_json.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn execution_events_json(&self) -> String {
        self.execution_events_json.clone()
    }
}

#[wasm_bindgen]
pub fn run_source_web(source: &str) -> WebRunResult {
    match run_source(source) {
        Ok(result) => WebRunResult {
            ok: true,
            output: result.output.join("\n"),
            error: String::new(),
            canvas_frames_json: serde_json::to_string(&result.canvas_frames)
                .unwrap_or_else(|_| "[]".to_string()),
            execution_events_json: serde_json::to_string(&result.events)
                .unwrap_or_else(|_| "[]".to_string()),
        },
        Err(err) => WebRunResult {
            ok: false,
            output: String::new(),
            error: err.to_string(),
            canvas_frames_json: "[]".to_string(),
            execution_events_json: "[]".to_string(),
        },
    }
}

#[wasm_bindgen]
pub fn ziium_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
