use crate::ast;
use crate::error::FrontendError;
use crate::message::{
    KeywordMessage, ResultiveMessage, WordMessage, keyword_message_for_selector,
    resultive_message_for,
};
use crate::parser::ParseMetadata;
use crate::parser::parse_source_with_metadata;
use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Bind {
        name: String,
        name_span: Option<Span>,
        value: Expr,
        mutable: bool,
        span: Option<Span>,
    },
    Assign {
        name: String,
        target_span: Option<Span>,
        value: Expr,
        span: Option<Span>,
    },
    Print {
        value: Expr,
        span: Option<Span>,
    },
    Sleep {
        duration_seconds: Expr,
        span: Option<Span>,
    },
    Return {
        value: Expr,
        keyword_span: Option<Span>,
        span: Option<Span>,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
        span: Option<Span>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Option<Span>,
    },
    FunctionDef {
        name: String,
        name_span: Option<Span>,
        params: Vec<String>,
        param_spans: Vec<Option<Span>>,
        body: Vec<Stmt>,
        span: Option<Span>,
    },
    Send {
        receiver: Expr,
        selector: SendSelector,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    IndexAssign {
        base: String,
        index: Expr,
        value: Expr,
        span: Option<Span>,
    },
    NamedCall {
        callee: Expr,
        named_args: Expr,
        span: Option<Span>,
    },
    Expr {
        expr: Expr,
        span: Option<Span>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Name {
        name: String,
        span: Option<Span>,
    },
    Int {
        raw: String,
        span: Option<Span>,
    },
    Float {
        raw: String,
        span: Option<Span>,
    },
    String {
        value: String,
        span: Option<Span>,
    },
    Bool {
        value: bool,
        span: Option<Span>,
    },
    None {
        span: Option<Span>,
    },
    List {
        items: Vec<Expr>,
        span: Option<Span>,
    },
    Record {
        entries: Vec<RecordEntry>,
        span: Option<Span>,
    },
    Unary {
        op: ast::UnaryOp,
        expr: Box<Expr>,
        span: Option<Span>,
    },
    Binary {
        left: Box<Expr>,
        op: ast::BinaryOp,
        right: Box<Expr>,
        span: Option<Span>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    Send {
        receiver: Box<Expr>,
        selector: SendSelector,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Option<Span>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEntry {
    pub key: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendSelector {
    Property(String),
    Transform(String),
    Word(WordMessage),
    Keyword(KeywordMessage),
    Resultive(ResultiveMessage),
}

#[derive(Debug, Clone, Default)]
struct LoweringCursor {
    statement_spans: Vec<Span>,
    statement_index: usize,
    expr_spans: Vec<Span>,
    expr_index: usize,
    declaration_spans: Vec<Span>,
    declaration_index: usize,
    assign_target_spans: Vec<Span>,
    assign_target_index: usize,
    return_spans: Vec<Span>,
    return_index: usize,
}

pub fn lower_program(program: &ast::Program) -> Program {
    lower_program_with_metadata(program, None)
}

pub fn parse_source(source: &str) -> Result<Program, FrontendError> {
    let (program, metadata) = parse_source_with_metadata(source)?;
    Ok(lower_program_with_metadata(&program, Some(&metadata)))
}

pub(crate) fn lower_program_with_metadata(
    program: &ast::Program,
    metadata: Option<&ParseMetadata>,
) -> Program {
    let mut cursor = LoweringCursor::from_metadata(metadata);
    Program {
        statements: program
            .statements
            .iter()
            .map(|statement| lower_stmt(statement, &mut cursor))
            .collect(),
    }
}

impl Stmt {
    pub fn span(&self) -> Option<&Span> {
        match self {
            Stmt::Bind { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::IndexAssign { span, .. }
            | Stmt::Print { span, .. }
            | Stmt::Sleep { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::FunctionDef { span, .. }
            | Stmt::Send { span, .. }
            | Stmt::NamedCall { span, .. }
            | Stmt::Expr { span, .. } => span.as_ref(),
        }
    }
}

impl Expr {
    pub fn span(&self) -> Option<&Span> {
        match self {
            Expr::Name { span, .. }
            | Expr::Int { span, .. }
            | Expr::Float { span, .. }
            | Expr::String { span, .. }
            | Expr::Bool { span, .. }
            | Expr::None { span }
            | Expr::List { span, .. }
            | Expr::Record { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Send { span, .. }
            | Expr::Index { span, .. } => span.as_ref(),
        }
    }
}

fn lower_stmt(stmt: &ast::Stmt, cursor: &mut LoweringCursor) -> Stmt {
    match stmt {
        ast::Stmt::Bind { name, value, mutable } => {
            let value = lower_expr(value, cursor);
            Stmt::Bind {
                name: name.clone(),
                name_span: cursor.next_declaration_span(),
                value,
                mutable: *mutable,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Assign { name, value } => {
            let value = lower_expr(value, cursor);
            Stmt::Assign {
                name: name.clone(),
                target_span: cursor.next_assign_target_span(),
                value,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::IndexAssign { base, index, value } => {
            let index = lower_expr(index, cursor);
            let value = lower_expr(value, cursor);
            Stmt::IndexAssign {
                base: base.clone(),
                index,
                value,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Print { value } => {
            let value = lower_expr(value, cursor);
            Stmt::Print {
                value,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Sleep { duration_seconds } => {
            let duration_seconds = lower_expr(duration_seconds, cursor);
            Stmt::Sleep {
                duration_seconds,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Return { value } => {
            let value = lower_expr(value, cursor);
            Stmt::Return {
                value,
                keyword_span: cursor.next_return_span(),
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let condition = lower_expr(condition, cursor);
            let then_block = then_block
                .iter()
                .map(|statement| lower_stmt(statement, cursor))
                .collect();
            let else_block = else_block.as_ref().map(|block| {
                block
                    .iter()
                    .map(|statement| lower_stmt(statement, cursor))
                    .collect()
            });
            Stmt::If {
                condition,
                then_block,
                else_block,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::While { condition, body } => {
            let condition = lower_expr(condition, cursor);
            let body = body
                .iter()
                .map(|statement| lower_stmt(statement, cursor))
                .collect();
            Stmt::While {
                condition,
                body,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::FunctionDef { name, params, body } => {
            let body = body
                .iter()
                .map(|statement| lower_stmt(statement, cursor))
                .collect();
            Stmt::FunctionDef {
                name: name.clone(),
                name_span: cursor.next_declaration_span(),
                params: params.clone(),
                param_spans: (0..params.len())
                    .map(|_| cursor.next_declaration_span())
                    .collect(),
                body,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::KeywordMessage {
            receiver,
            selector,
            arg,
        } => {
            let receiver = lower_expr(receiver, cursor);
            let arg = lower_expr(arg, cursor);
            let selector = keyword_message_for_selector(selector)
                .expect("parser should only lower supported keyword messages");
            Stmt::Send {
                receiver,
                selector: SendSelector::Keyword(selector),
                args: vec![arg],
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Resultive {
            receiver,
            role,
            verb,
        } => {
            let receiver = lower_expr(receiver, cursor);
            Stmt::Send {
                receiver,
                selector: lower_resultive_selector(role, verb),
                args: Vec::new(),
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::NamedCall { callee, named_args } => {
            let callee = lower_expr(callee, cursor);
            let named_args = lower_expr(named_args, cursor);
            Stmt::NamedCall {
                callee,
                named_args,
                span: cursor.next_statement_span(),
            }
        }
        ast::Stmt::Expr(expr) => {
            let expr = lower_expr(expr, cursor);
            Stmt::Expr {
                expr,
                span: cursor.next_statement_span(),
            }
        }
    }
}

fn lower_expr(expr: &ast::Expr, cursor: &mut LoweringCursor) -> Expr {
    match expr {
        ast::Expr::Name(name) => Expr::Name {
            name: name.clone(),
            span: cursor.next_expr_span(),
        },
        ast::Expr::Int(raw) => Expr::Int {
            raw: raw.clone(),
            span: cursor.next_expr_span(),
        },
        ast::Expr::Float(raw) => Expr::Float {
            raw: raw.clone(),
            span: cursor.next_expr_span(),
        },
        ast::Expr::String(value) => Expr::String {
            value: value.clone(),
            span: cursor.next_expr_span(),
        },
        ast::Expr::Bool(value) => Expr::Bool {
            value: *value,
            span: cursor.next_expr_span(),
        },
        ast::Expr::None => Expr::None {
            span: cursor.next_expr_span(),
        },
        ast::Expr::List(items) => {
            let items = items.iter().map(|item| lower_expr(item, cursor)).collect();
            Expr::List {
                items,
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Record(entries) => {
            let entries = entries
                .iter()
                .map(|entry| RecordEntry {
                    key: entry.key.clone(),
                    value: lower_expr(&entry.value, cursor),
                })
                .collect();
            Expr::Record {
                entries,
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Unary { op, expr } => {
            let expr = Box::new(lower_expr(expr, cursor));
            Expr::Unary {
                op: *op,
                expr,
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Binary {
            left,
            op,
            right,
            form,
        } => {
            let left = Box::new(lower_expr(left, cursor));
            let right = Box::new(lower_expr(right, cursor));
            let span = cursor.next_expr_span();
            match form {
                ast::BinarySurface::Word => Expr::Send {
                    receiver: left,
                    selector: SendSelector::Word(word_selector(*op)),
                    args: vec![*right],
                    span,
                },
                ast::BinarySurface::Symbol => Expr::Binary {
                    left,
                    op: *op,
                    right,
                    span,
                },
            }
        }
        ast::Expr::Call { callee, args } => {
            let callee = Box::new(lower_expr(callee, cursor));
            let args = args.iter().map(|arg| lower_expr(arg, cursor)).collect();
            Expr::Call {
                callee,
                args,
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::TransformCall { input, callee } => {
            let receiver = Box::new(lower_expr(input, cursor));
            Expr::Send {
                receiver,
                selector: SendSelector::Transform(callee.clone()),
                args: Vec::new(),
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Resultive {
            receiver,
            role,
            verb,
        } => {
            let receiver = Box::new(lower_expr(receiver, cursor));
            Expr::Send {
                receiver,
                selector: lower_resultive_selector(role, verb),
                args: Vec::new(),
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Index { base, index } => {
            let base = Box::new(lower_expr(base, cursor));
            let index = Box::new(lower_expr(index, cursor));
            Expr::Index {
                base,
                index,
                span: cursor.next_expr_span(),
            }
        }
        ast::Expr::Property { base, name } => {
            let receiver = Box::new(lower_expr(base, cursor));
            Expr::Send {
                receiver,
                selector: SendSelector::Property(name.clone()),
                args: Vec::new(),
                span: cursor.next_expr_span(),
            }
        }
    }
}

fn lower_resultive_selector(role: &str, verb: &str) -> SendSelector {
    // Statement surface `꺼낸다` and expression surface `꺼낸` both lower to
    // the same closed resultive selector.
    let verb = match verb {
        "꺼낸다" => "꺼낸",
        "고른다" => "고른",
        other => other,
    };
    let selector =
        resultive_message_for(role, verb).expect("parser should only lower supported resultives");
    SendSelector::Resultive(selector)
}

impl LoweringCursor {
    fn from_metadata(metadata: Option<&ParseMetadata>) -> Self {
        let Some(metadata) = metadata else {
            return Self::default();
        };

        Self {
            statement_spans: metadata.statement_spans.clone(),
            statement_index: 0,
            expr_spans: metadata.expr_spans.clone(),
            expr_index: 0,
            declaration_spans: metadata.declaration_spans.clone(),
            declaration_index: 0,
            assign_target_spans: metadata.assign_target_spans.clone(),
            assign_target_index: 0,
            return_spans: metadata.return_spans.clone(),
            return_index: 0,
        }
    }

    fn next_statement_span(&mut self) -> Option<Span> {
        let span = self.statement_spans.get(self.statement_index).cloned();
        if span.is_some() {
            self.statement_index += 1;
        }
        span
    }

    fn next_expr_span(&mut self) -> Option<Span> {
        let span = self.expr_spans.get(self.expr_index).cloned();
        if span.is_some() {
            self.expr_index += 1;
        }
        span
    }

    fn next_declaration_span(&mut self) -> Option<Span> {
        let span = self.declaration_spans.get(self.declaration_index).cloned();
        if span.is_some() {
            self.declaration_index += 1;
        }
        span
    }

    fn next_assign_target_span(&mut self) -> Option<Span> {
        let span = self
            .assign_target_spans
            .get(self.assign_target_index)
            .cloned();
        if span.is_some() {
            self.assign_target_index += 1;
        }
        span
    }

    fn next_return_span(&mut self) -> Option<Span> {
        let span = self.return_spans.get(self.return_index).cloned();
        if span.is_some() {
            self.return_index += 1;
        }
        span
    }
}

fn word_selector(op: ast::BinaryOp) -> WordMessage {
    match op {
        ast::BinaryOp::Add => WordMessage::Add,
        ast::BinaryOp::Subtract => WordMessage::Subtract,
        ast::BinaryOp::Multiply => WordMessage::Multiply,
        ast::BinaryOp::Divide => WordMessage::Divide,
        _ => panic!("word selector requested for non-word binary op"),
    }
}
