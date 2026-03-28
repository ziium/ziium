use crate::error::RuntimeError;
use crate::interpreter::{InterpreterSession, Value};
use crate::run_source;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const NEED_CHOICE_MARKER: &str = "__NEED_CHOICE__";

#[wasm_bindgen]
pub struct WebRunResult {
    ok: bool,
    output: String,
    error: String,
    canvas_frames_json: String,
    execution_events_json: String,
    choices_json: String,
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

    #[wasm_bindgen(getter)]
    pub fn choices_json(&self) -> String {
        self.choices_json.clone()
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
            choices_json: String::new(),
        },
        Err(err) => WebRunResult {
            ok: false,
            output: String::new(),
            error: err.to_string(),
            canvas_frames_json: "[]".to_string(),
            execution_events_json: "[]".to_string(),
            choices_json: String::new(),
        },
    }
}

#[wasm_bindgen]
pub fn run_source_web_with_choices(source: &str, choices_json: &str) -> WebRunResult {
    let choices: Vec<String> = serde_json::from_str(choices_json).unwrap_or_default();
    let queue = Rc::new(RefCell::new(choices));
    let pending: Rc<RefCell<Option<Vec<String>>>> = Rc::new(RefCell::new(None));

    let mut session = InterpreterSession::new();
    let q = queue.clone();
    let p = pending.clone();
    session.set_choose_fn(move |options: &[Value]| {
        let mut q = q.borrow_mut();
        if let Some(choice_str) = q.first().cloned() {
            q.remove(0);
            for opt in options {
                if opt.render() == choice_str {
                    return Ok(opt.clone());
                }
            }
            Err(RuntimeError::new("선택지를 찾을 수 없습니다."))
        } else {
            let opts: Vec<String> = options.iter().map(|v| v.render()).collect();
            *p.borrow_mut() = Some(opts);
            Err(RuntimeError::new(NEED_CHOICE_MARKER))
        }
    });

    match session.run_source(source) {
        Ok(result) => WebRunResult {
            ok: true,
            output: result.output.join("\n"),
            error: String::new(),
            canvas_frames_json: serde_json::to_string(&result.canvas_frames)
                .unwrap_or_else(|_| "[]".to_string()),
            execution_events_json: serde_json::to_string(&result.events)
                .unwrap_or_else(|_| "[]".to_string()),
            choices_json: String::new(),
        },
        Err(_) => {
            let events = session.drain_events();
            let events_json =
                serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());

            if let Some(opts) = pending.borrow().as_ref() {
                let choices =
                    serde_json::to_string(opts).unwrap_or_else(|_| "[]".to_string());
                WebRunResult {
                    ok: true,
                    output: String::new(),
                    error: String::new(),
                    canvas_frames_json: "[]".to_string(),
                    execution_events_json: events_json,
                    choices_json: choices,
                }
            } else {
                WebRunResult {
                    ok: false,
                    output: String::new(),
                    error: "실행 중 오류가 발생했습니다.".to_string(),
                    canvas_frames_json: "[]".to_string(),
                    execution_events_json: events_json,
                    choices_json: String::new(),
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn ziium_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
