pub mod ast;
pub mod error;
pub mod hir;
pub mod interpreter;
pub mod lexer;
mod message;
pub mod normalizer;
pub mod parser;
pub mod resolver;
pub mod token;
#[cfg(target_arch = "wasm32")]
mod web;

pub use ast::{BinaryOp, BinarySurface, Expr, Program, RecordEntry, Stmt, UnaryOp};
pub use error::{FrontendError, LexError, ParseError, ResolveError, RunError, RuntimeError};
pub use hir::{
    Expr as HirExpr, Program as HirProgram, RecordEntry as HirRecordEntry,
    SendSelector as HirSendSelector, Stmt as HirStmt, lower_program as lower_to_hir,
    parse_source as parse_source_to_hir,
};
pub use interpreter::{
    CanvasCommand, CanvasFrame, ExecutionEvent, ExecutionResult, InterpreterSession, Value,
    interpret_hir_program, interpret_program, run_source,
};
pub use lexer::lex;
pub use normalizer::normalize_tokens;
pub use parser::{parse_source, parse_tokens};
pub use resolver::{ResolverSession, resolve_hir_program, resolve_program};
pub use token::{Span, Token, TokenKind};
