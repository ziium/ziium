pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod resolver;
pub mod token;
#[cfg(target_arch = "wasm32")]
mod web;

pub use ast::{BinaryOp, Expr, Program, RecordEntry, Stmt, UnaryOp};
pub use error::{FrontendError, LexError, ParseError, ResolveError, RunError, RuntimeError};
pub use interpreter::{ExecutionResult, InterpreterSession, Value, interpret_program, run_source};
pub use lexer::lex;
pub use normalizer::normalize_tokens;
pub use parser::{parse_source, parse_tokens};
pub use resolver::{ResolverSession, resolve_program};
pub use token::{Span, Token, TokenKind};
