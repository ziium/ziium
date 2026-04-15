use crate::error::RuntimeError;
use crate::interpreter::{InterpreterSession, Value};
use crate::lexer::lex;
use crate::run_source;
use crate::token::TokenKind;
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

#[wasm_bindgen]
pub fn highlight_web(source: &str) -> String {
    let lines: Vec<Vec<char>> = source.lines().map(|l| l.chars().collect()).collect();

    let tokens = match lex(source) {
        Ok(t) => t,
        Err(_) => return html_escape(source),
    };

    let mut out = String::new();
    let mut ln = 1usize;
    let mut col = 1usize;

    for tok in &tokens {
        match tok.kind {
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => continue,
            _ => {}
        }

        let sl = tok.span.start_line;
        let sc = tok.span.start_column;
        let el = tok.span.end_line;
        let ec = tok.span.end_column;

        // gap: newlines between current pos and token start
        while ln < sl {
            let li = ln - 1;
            if li < lines.len() {
                let from = col - 1;
                if from < lines[li].len() {
                    push_escaped(&mut out, &lines[li][from..]);
                }
            }
            out.push('\n');
            ln += 1;
            col = 1;
        }

        // gap: chars on the same line before token
        if col < sc {
            let li = ln - 1;
            if li < lines.len() {
                let from = col - 1;
                let to = (sc - 1).min(lines[li].len());
                if from < to {
                    push_escaped(&mut out, &lines[li][from..to]);
                }
            }
        }

        // extract token text from source (using spans, not lexeme)
        let text: String = if sl == el {
            let li = sl - 1;
            if li < lines.len() {
                let from = (sc - 1).min(lines[li].len());
                let to = (ec - 1).min(lines[li].len());
                lines[li][from..to].iter().collect()
            } else {
                tok.lexeme.clone()
            }
        } else {
            tok.lexeme.clone()
        };

        let cls = token_class(tok.kind);
        if cls.is_empty() {
            push_escaped_str(&mut out, &text);
        } else {
            out.push_str("<span class=\"zh-");
            out.push_str(cls);
            out.push_str("\">");
            push_escaped_str(&mut out, &text);
            out.push_str("</span>");
        }

        ln = el;
        col = ec;
    }

    // remaining source after last token
    while ln <= lines.len() {
        let li = ln - 1;
        let from = col - 1;
        if from < lines[li].len() {
            push_escaped(&mut out, &lines[li][from..]);
        }
        if ln < lines.len() {
            out.push('\n');
        }
        ln += 1;
        col = 1;
    }

    out
}

fn token_class(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Function | TokenKind::FunctionTopic | TokenKind::Copula
        | TokenKind::If | TokenKind::Else | TokenKind::In | TokenKind::During
        | TokenKind::Receive | TokenKind::Nothing | TokenKind::ReceiveNot | TokenKind::ReceiveNeg
        | TokenKind::Print | TokenKind::Return | TokenKind::Change
        | TokenKind::Store | TokenKind::Exist | TokenKind::Each | TokenKind::About
        | TokenKind::ResultMarker => "kw",

        TokenKind::Topic | TokenKind::Subject | TokenKind::Object
        | TokenKind::Gen | TokenKind::Locative | TokenKind::From | TokenKind::Direction
        | TokenKind::Than | TokenKind::With | TokenKind::Amount => "pa",

        TokenKind::String => "str",
        TokenKind::Int | TokenKind::Float => "num",

        TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash | TokenKind::Percent
        | TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge
        | TokenKind::And | TokenKind::Or | TokenKind::Not => "op",

        TokenKind::True | TokenKind::False | TokenKind::None => "lit",

        _ => "",
    }
}

fn push_escaped(out: &mut String, chars: &[char]) {
    for &ch in chars {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

fn push_escaped_str(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

fn html_escape(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    push_escaped_str(&mut out, source);
    out
}
