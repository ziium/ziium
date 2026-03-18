use crate::ast::{BinaryOp, UnaryOp};
use crate::error::{RunError, RuntimeError};
use crate::hir::{self, Expr, Program, SendSelector, Stmt};
use crate::message::{
    KeywordMessage, ResultiveMessage, UnaryMessage, keyword_message_for_selector,
    resultive_message_for, unary_message_for_property,
};
use crate::parser::parse_source_with_metadata;
use crate::resolver::ResolverSession;
use crate::token::Span;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: Vec<String>,
    pub canvas_frames: Vec<CanvasFrame>,
    pub events: Vec<ExecutionEvent>,
}

#[derive(Debug)]
pub struct InterpreterSession {
    interpreter: Interpreter,
    resolver: ResolverSession,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    None,
    List(Rc<RefCell<Vec<Value>>>),
    Record(Rc<RefCell<BTreeMap<String, Value>>>),
    Function(FunctionValue),
    Host(HostValue),
}

#[derive(Debug, Clone)]
pub enum FunctionValue {
    User(UserFunction),
    Builtin(BuiltinFunction),
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Rc<Vec<Stmt>>,
    env: EnvRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Length,
    Push,
    PopLast,
    ToString,
    ToInt,
    ToFloat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostValue {
    Canvas,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CanvasFrame {
    pub commands: Vec<CanvasCommand>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ExecutionEvent {
    Output { text: String },
    Sleep { seconds: f64 },
    CanvasFrame { frame: CanvasFrame },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind")]
pub enum CanvasCommand {
    Clear {
        background: String,
    },
    Dot {
        x: f64,
        y: f64,
        color: String,
        size: f64,
    },
    FillRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: String,
    },
    FillText {
        text: String,
        x: f64,
        y: f64,
        color: String,
        size: f64,
    },
}

#[derive(Debug)]
struct Interpreter {
    globals: EnvRef,
    output: Vec<String>,
    current_canvas_commands: Vec<CanvasCommand>,
    canvas_frames: Vec<CanvasFrame>,
    events: Vec<ExecutionEvent>,
}

#[derive(Debug)]
struct Environment {
    values: BTreeMap<String, Value>,
    parent: Option<EnvRef>,
}

#[derive(Debug)]
enum ExecSignal {
    Continue,
    Return(Value),
}

pub fn interpret_program(program: &crate::ast::Program) -> Result<ExecutionResult, RuntimeError> {
    let mut session = InterpreterSession::new();
    session.interpret_program(program)
}

pub fn interpret_hir_program(program: &Program) -> Result<ExecutionResult, RuntimeError> {
    let mut session = InterpreterSession::new();
    session.interpret_hir_program(program)
}

pub fn run_source(source: &str) -> Result<ExecutionResult, RunError> {
    let mut session = InterpreterSession::new();
    session.run_source(source)
}

impl InterpreterSession {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            resolver: ResolverSession::new(),
        }
    }

    pub fn interpret_program(
        &mut self,
        program: &crate::ast::Program,
    ) -> Result<ExecutionResult, RuntimeError> {
        let hir_program = hir::lower_program(program);
        self.resolver
            .resolve_hir_program(&hir_program)
            .map_err(|err| RuntimeError::with_span(err.message.clone(), err.span.clone()))?;
        self.interpreter.run_program(&hir_program)
    }

    pub fn interpret_hir_program(
        &mut self,
        program: &Program,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.interpreter.run_program(program)
    }

    pub fn run_source(&mut self, source: &str) -> Result<ExecutionResult, RunError> {
        let (program, metadata) = parse_source_with_metadata(source).map_err(RunError::from)?;
        let hir_program = hir::lower_program_with_metadata(&program, Some(&metadata));
        self.resolver
            .resolve_hir_program(&hir_program)
            .map_err(crate::error::FrontendError::from)
            .map_err(RunError::from)?;
        self.interpreter
            .run_program(&hir_program)
            .map_err(RunError::from)
    }
}

impl Default for InterpreterSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    fn new() -> Self {
        let globals = Environment::new(None);
        install_builtins(&globals);

        Self {
            globals,
            output: Vec::new(),
            current_canvas_commands: Vec::new(),
            canvas_frames: Vec::new(),
            events: Vec::new(),
        }
    }

    fn run_program(&mut self, program: &Program) -> Result<ExecutionResult, RuntimeError> {
        let output_start = self.output.len();
        self.current_canvas_commands.clear();
        self.canvas_frames.clear();
        self.events.clear();
        match self.execute_block(&program.statements, self.globals.clone())? {
            ExecSignal::Continue => {
                self.finish_canvas_frame();
                Ok(ExecutionResult {
                    output: self.output[output_start..].to_vec(),
                    canvas_frames: self.canvas_frames.clone(),
                    events: self.events.clone(),
                })
            }
            ExecSignal::Return(_) => Err(RuntimeError::new(
                "`돌려준다`는 함수 본문 안에서만 사용할 수 있습니다.",
            )),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: EnvRef,
    ) -> Result<ExecSignal, RuntimeError> {
        for statement in statements {
            match self.execute_stmt(statement, env.clone())? {
                ExecSignal::Continue => {}
                signal @ ExecSignal::Return(_) => return Ok(signal),
            }
        }

        Ok(ExecSignal::Continue)
    }

    fn execute_stmt(&mut self, stmt: &Stmt, env: EnvRef) -> Result<ExecSignal, RuntimeError> {
        let stmt_span = stmt.span().cloned();
        match stmt {
            Stmt::Bind { name, value, .. } => {
                if env.borrow().values.contains_key(name) {
                    return Err(RuntimeError::with_span(
                        format!("`{}`은(는) 현재 스코프에 이미 정의되어 있습니다.", name),
                        stmt_span,
                    ));
                }
                let value = self
                    .eval_expr(value, env.clone())
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                env.borrow_mut().values.insert(name.clone(), value);
                Ok(ExecSignal::Continue)
            }
            Stmt::Assign { name, value, .. } => {
                let value = self
                    .eval_expr(value, env.clone())
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                assign_value(&env, name, value).map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Continue)
            }
            Stmt::Print { value, .. } => {
                let value = self
                    .eval_expr(value, env)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                let rendered = value.render();
                self.output.push(rendered.clone());
                self.events.push(ExecutionEvent::Output { text: rendered });
                Ok(ExecSignal::Continue)
            }
            Stmt::Sleep {
                duration_seconds, ..
            } => {
                let duration_value = self
                    .eval_expr(duration_seconds, env)
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                let seconds = expect_sleep_seconds(duration_value)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                self.finish_canvas_frame();
                self.events.push(ExecutionEvent::Sleep { seconds });
                Ok(ExecSignal::Continue)
            }
            Stmt::Return { value, .. } => {
                let value = self
                    .eval_expr(value, env)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Return(value))
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let condition_span = condition.span().cloned().or_else(|| stmt_span.clone());
                let condition = self
                    .eval_expr(condition, env.clone())
                    .map_err(|err| err.with_fallback_span(condition_span.clone()))?;
                match condition {
                    Value::Bool(true) => self.execute_block(then_block, env),
                    Value::Bool(false) => {
                        if let Some(else_block) = else_block {
                            self.execute_block(else_block, env)
                        } else {
                            Ok(ExecSignal::Continue)
                        }
                    }
                    _ => Err(RuntimeError::with_span(
                        "조건식은 `참` 또는 `거짓`이어야 합니다.",
                        condition_span,
                    )),
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                let condition_span = condition.span().cloned().or_else(|| stmt_span.clone());
                loop {
                    let condition_value = self
                        .eval_expr(condition, env.clone())
                        .map_err(|err| err.with_fallback_span(condition_span.clone()))?;
                    match condition_value {
                        Value::Bool(true) => match self.execute_block(body, env.clone())? {
                            ExecSignal::Continue => {}
                            signal @ ExecSignal::Return(_) => return Ok(signal),
                        },
                        Value::Bool(false) => break,
                        _ => {
                            return Err(RuntimeError::with_span(
                                "반복 조건식은 `참` 또는 `거짓`이어야 합니다.",
                                condition_span,
                            ));
                        }
                    }
                }
                Ok(ExecSignal::Continue)
            }
            Stmt::FunctionDef {
                name, params, body, ..
            } => {
                if env.borrow().values.contains_key(name) {
                    return Err(RuntimeError::with_span(
                        format!("`{}`은(는) 현재 스코프에 이미 정의되어 있습니다.", name),
                        stmt_span,
                    ));
                }

                let cloned_body = Rc::new(body.clone());
                let function = Value::Function(FunctionValue::User(UserFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: cloned_body.clone(),
                    env: env.clone(),
                }));
                env.borrow_mut().values.insert(name.clone(), function);
                Ok(ExecSignal::Continue)
            }
            Stmt::Send {
                receiver,
                selector,
                args,
                ..
            } => {
                let receiver = self
                    .eval_expr(receiver, env.clone())
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(
                        self.eval_expr(arg, env.clone())
                            .map_err(|err| err.with_fallback_span(stmt_span.clone()))?,
                    );
                }
                self.execute_send_stmt(receiver, selector, arg_values)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Continue)
            }
            Stmt::NamedCall {
                callee, named_args, ..
            } => {
                let callee = self
                    .eval_expr(callee, env.clone())
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                let named_args = self
                    .eval_expr(named_args, env)
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                self.call_named_value(callee, named_args, stmt_span)?;
                Ok(ExecSignal::Continue)
            }
            Stmt::Expr { expr, .. } => {
                self.eval_expr(expr, env)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Continue)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr, env: EnvRef) -> Result<Value, RuntimeError> {
        let expr_span = expr.span().cloned();
        match expr {
            Expr::Name { name, .. } => {
                lookup_value(&env, name).map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Int { raw, .. } => raw.parse::<i64>().map(Value::Int).map_err(|_| {
                RuntimeError::with_span(
                    format!("정수 리터럴 `{}`를 해석할 수 없습니다.", raw),
                    expr_span,
                )
            }),
            Expr::Float { raw, .. } => raw.parse::<f64>().map(Value::Float).map_err(|_| {
                RuntimeError::with_span(
                    format!("실수 리터럴 `{}`를 해석할 수 없습니다.", raw),
                    expr_span,
                )
            }),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::None { .. } => Ok(Value::None),
            Expr::List { items, .. } => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(
                        self.eval_expr(item, env.clone())
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                Ok(Value::List(Rc::new(RefCell::new(values))))
            }
            Expr::Record { entries, .. } => {
                let mut map = BTreeMap::new();
                for entry in entries {
                    map.insert(
                        entry.key.clone(),
                        self.eval_expr(&entry.value, env.clone())
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                Ok(Value::Record(Rc::new(RefCell::new(map))))
            }
            Expr::Unary { op, expr, .. } => {
                let value = self
                    .eval_expr(expr, env)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_unary(*op, value)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                let left = self
                    .eval_expr(left, env.clone())
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let right = self
                    .eval_expr(right, env)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_binary(left, *op, right)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Call { callee, args, .. } => {
                let callee = self
                    .eval_expr(callee, env.clone())
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(
                        self.eval_expr(arg, env.clone())
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                self.call_value(callee, arg_values, expr_span.clone())
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Send {
                receiver,
                selector,
                args,
                ..
            } => {
                let receiver = self
                    .eval_expr(receiver, env.clone())
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(
                        self.eval_expr(arg, env.clone())
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                self.eval_send_expr(receiver, selector, arg_values, env, expr_span.clone())
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Index { base, index, .. } => {
                let base = self
                    .eval_expr(base, env.clone())
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let index = self
                    .eval_expr(index, env)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_index(base, index)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
        }
    }

    fn eval_unary(&self, op: UnaryOp, value: Value) -> Result<Value, RuntimeError> {
        match op {
            UnaryOp::Negate => match value {
                Value::Int(value) => Ok(Value::Int(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(RuntimeError::new("단항 `-`는 숫자에만 사용할 수 있습니다.")),
            },
            UnaryOp::Not => match value {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                _ => Err(RuntimeError::new(
                    "`아니다`는 불리언 값에만 사용할 수 있습니다.",
                )),
            },
        }
    }

    fn eval_binary(&self, left: Value, op: BinaryOp, right: Value) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
                _ => Err(RuntimeError::new(
                    "`+`는 숫자끼리 또는 문자열끼리만 사용할 수 있습니다.",
                )),
            },
            BinaryOp::Subtract => numeric_binary(left, right, |a, b| a - b, |a, b| a - b),
            BinaryOp::Multiply => numeric_binary(left, right, |a, b| a * b, |a, b| a * b),
            BinaryOp::Divide => numeric_binary(left, right, |a, b| a / b, |a, b| a / b),
            BinaryOp::Modulo => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(RuntimeError::new("`%`는 정수끼리만 사용할 수 있습니다.")),
            },
            BinaryOp::Equal => Ok(Value::Bool(values_equal(&left, &right))),
            BinaryOp::NotEqual => Ok(Value::Bool(!values_equal(&left, &right))),
            BinaryOp::Less => comparison_binary(left, right, |a, b| a < b),
            BinaryOp::LessEqual => comparison_binary(left, right, |a, b| a <= b),
            BinaryOp::Greater => comparison_binary(left, right, |a, b| a > b),
            BinaryOp::GreaterEqual => comparison_binary(left, right, |a, b| a >= b),
            BinaryOp::And => match (left, right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                _ => Err(RuntimeError::new(
                    "`그리고`는 불리언 값에만 사용할 수 있습니다.",
                )),
            },
            BinaryOp::Or => match (left, right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                _ => Err(RuntimeError::new(
                    "`또는`은 불리언 값에만 사용할 수 있습니다.",
                )),
            },
        }
    }

    fn call_value(
        &mut self,
        callee: Value,
        args: Vec<Value>,
        call_span: Option<Span>,
    ) -> Result<Value, RuntimeError> {
        match callee {
            Value::Function(FunctionValue::Builtin(function)) => self
                .call_builtin(function, args)
                .map_err(|err| err.with_call_frame(function.name(), call_span)),
            Value::Function(FunctionValue::User(function)) => {
                if function.params.len() != args.len() {
                    return Err(RuntimeError::new(format!(
                        "함수 인수 개수가 맞지 않습니다. 기대: {}, 실제: {}",
                        function.params.len(),
                        args.len()
                    )));
                }

                let frame = Environment::new(Some(function.env.clone()));
                for (param, arg) in function.params.iter().zip(args) {
                    frame.borrow_mut().values.insert(param.clone(), arg);
                }

                match self
                    .execute_block(function.body.as_ref(), frame)
                    .map_err(|err| err.with_call_frame(function.name.clone(), call_span))?
                {
                    ExecSignal::Continue => Ok(Value::None),
                    ExecSignal::Return(value) => Ok(value),
                }
            }
            _ => Err(RuntimeError::new("호출할 수 없는 값을 호출했습니다.")),
        }
    }

    fn call_named_value(
        &mut self,
        callee: Value,
        named_args: Value,
        call_span: Option<Span>,
    ) -> Result<Value, RuntimeError> {
        match callee {
            Value::Function(FunctionValue::User(function)) => {
                let named_args = expect_record("호출한다", named_args)?;

                for key in named_args.keys() {
                    if !function.params.iter().any(|param| param == key) {
                        return Err(RuntimeError::new(format!(
                            "`{}` 함수에는 `{}` 인수가 없습니다.",
                            function.name, key
                        )));
                    }
                }

                let mut ordered_args = Vec::with_capacity(function.params.len());
                for param in &function.params {
                    let value = named_args.get(param).cloned().ok_or_else(|| {
                        RuntimeError::new(format!(
                            "`{}` 함수 호출에 `{}` 인수가 필요합니다.",
                            function.name, param
                        ))
                    })?;
                    ordered_args.push(value);
                }

                self.call_value(
                    Value::Function(FunctionValue::User(function)),
                    ordered_args,
                    call_span,
                )
            }
            Value::Function(FunctionValue::Builtin(function)) => Err(RuntimeError::new(format!(
                "`{}`에는 아직 이름 붙은 호출을 사용할 수 없습니다.",
                function.name()
            ))),
            _ => Err(RuntimeError::new("호출할 수 없는 값을 호출했습니다.")),
        }
    }

    fn call_builtin(
        &mut self,
        function: BuiltinFunction,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match function {
            BuiltinFunction::Length => {
                let [value] = expect_arity::<1>("길이", args)?;
                match value {
                    Value::List(items) => Ok(Value::Int(items.borrow().len() as i64)),
                    Value::String(text) => Ok(Value::Int(text.chars().count() as i64)),
                    Value::Record(map) => Ok(Value::Int(map.borrow().len() as i64)),
                    _ => Err(RuntimeError::new(
                        "`길이`는 목록, 문자열, 레코드에만 사용할 수 있습니다.",
                    )),
                }
            }
            BuiltinFunction::Push => {
                let [list, value] = expect_arity::<2>("추가", args)?;
                match list {
                    Value::List(items) => {
                        items.borrow_mut().push(value);
                        Ok(Value::None)
                    }
                    _ => Err(RuntimeError::new(
                        "`추가`의 첫 번째 인수는 목록이어야 합니다.",
                    )),
                }
            }
            BuiltinFunction::PopLast => {
                let [list] = expect_arity::<1>("마지막꺼내기", args)?;
                match list {
                    Value::List(items) => items.borrow_mut().pop().ok_or_else(|| {
                        RuntimeError::new("빈 목록에서는 마지막 값을 꺼낼 수 없습니다.")
                    }),
                    _ => Err(RuntimeError::new(
                        "`마지막꺼내기`는 목록에만 사용할 수 있습니다.",
                    )),
                }
            }
            BuiltinFunction::ToString => {
                let [value] = expect_arity::<1>("문자열로", args)?;
                Ok(Value::String(value.render()))
            }
            BuiltinFunction::ToInt => {
                let [value] = expect_arity::<1>("정수로", args)?;
                match value {
                    Value::Int(value) => Ok(Value::Int(value)),
                    Value::Float(value) => Ok(Value::Int(value as i64)),
                    Value::String(value) => value
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| RuntimeError::new("문자열을 정수로 바꿀 수 없습니다.")),
                    _ => Err(RuntimeError::new(
                        "`정수로`는 숫자 또는 문자열에만 사용할 수 있습니다.",
                    )),
                }
            }
            BuiltinFunction::ToFloat => {
                let [value] = expect_arity::<1>("실수로", args)?;
                match value {
                    Value::Int(value) => Ok(Value::Float(value as f64)),
                    Value::Float(value) => Ok(Value::Float(value)),
                    Value::String(value) => value
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| RuntimeError::new("문자열을 실수로 바꿀 수 없습니다.")),
                    _ => Err(RuntimeError::new(
                        "`실수로`는 숫자 또는 문자열에만 사용할 수 있습니다.",
                    )),
                }
            }
        }
    }

    fn eval_index(&self, base: Value, index: Value) -> Result<Value, RuntimeError> {
        let index = match index {
            Value::Int(value) if value >= 0 => value as usize,
            _ => return Err(RuntimeError::new("인덱스는 0 이상의 정수여야 합니다.")),
        };

        match base {
            Value::List(items) => items
                .borrow()
                .get(index)
                .cloned()
                .ok_or_else(|| RuntimeError::new("목록 인덱스가 범위를 벗어났습니다.")),
            Value::String(text) => text
                .chars()
                .nth(index)
                .map(|ch| Value::String(ch.to_string()))
                .ok_or_else(|| RuntimeError::new("문자열 인덱스가 범위를 벗어났습니다.")),
            _ => Err(RuntimeError::new(
                "인덱싱은 목록 또는 문자열에만 사용할 수 있습니다.",
            )),
        }
    }

    fn eval_word_message(
        &self,
        receiver: Value,
        selector: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let [arg] = expect_arity::<1>(selector, args)?;
        match selector {
            "더하기" => self.eval_binary(receiver, BinaryOp::Add, arg),
            "빼기" => self.eval_binary(receiver, BinaryOp::Subtract, arg),
            "곱하기" => self.eval_binary(receiver, BinaryOp::Multiply, arg),
            "나누기" => self.eval_binary(receiver, BinaryOp::Divide, arg),
            _ => Err(RuntimeError::new(format!(
                "`{}` 단어 메시지는 아직 지원하지 않습니다.",
                selector
            ))),
        }
    }

    fn eval_send_expr(
        &mut self,
        receiver: Value,
        selector: &SendSelector,
        args: Vec<Value>,
        env: EnvRef,
        expr_span: Option<Span>,
    ) -> Result<Value, RuntimeError> {
        match selector {
            SendSelector::Property(name) => {
                if !args.is_empty() {
                    return Err(RuntimeError::new("속성 메시지는 인수를 받을 수 없습니다."));
                }
                self.eval_property(receiver, name)
            }
            SendSelector::Transform(callee_name) => {
                if !args.is_empty() {
                    return Err(RuntimeError::new(
                        "변환 호출은 추가 인수를 받을 수 없습니다.",
                    ));
                }
                let callee = lookup_value(&env, callee_name)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.call_value(callee, vec![receiver], expr_span)
            }
            SendSelector::Word(selector) => self.eval_word_message(receiver, selector, args),
            SendSelector::Resultive { role, verb } => {
                if !args.is_empty() {
                    return Err(RuntimeError::new(
                        "결과 서술 메시지는 추가 인수를 받을 수 없습니다.",
                    ));
                }
                self.eval_resultive(receiver, role, verb)
            }
            SendSelector::Keyword(_) => Err(RuntimeError::new(
                "키워드 메시지는 표현식 자리에서 사용할 수 없습니다.",
            )),
        }
    }

    fn execute_send_stmt(
        &mut self,
        receiver: Value,
        selector: &SendSelector,
        args: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        match selector {
            SendSelector::Keyword(selector) => {
                self.execute_keyword_message(receiver, selector, args)
            }
            SendSelector::Resultive { role, verb } => {
                if !args.is_empty() {
                    return Err(RuntimeError::new(
                        "결과 서술 메시지는 추가 인수를 받을 수 없습니다.",
                    ));
                }
                self.eval_resultive(receiver, role, verb).map(|_| ())
            }
            _ => Err(RuntimeError::new(
                "이 메시지는 문장 자리에서 사용할 수 없습니다.",
            )),
        }
    }

    fn eval_property(&self, base: Value, name: &str) -> Result<Value, RuntimeError> {
        match base {
            Value::Record(map) => {
                if let Some(value) = map.borrow().get(name).cloned() {
                    return Ok(value);
                }

                if let Some(selector) = unary_message_for_property(name) {
                    return self.send_unary_message(Value::Record(map), selector);
                }

                Err(RuntimeError::new(format!(
                    "이 값에는 `{}` 속성이 없습니다.",
                    name
                )))
            }
            other => {
                if let Some(selector) = unary_message_for_property(name) {
                    self.send_unary_message(other, selector)
                } else {
                    Err(RuntimeError::new(format!(
                        "이 값에는 `{}` 속성이 없습니다.",
                        name
                    )))
                }
            }
        }
    }

    fn eval_resultive(
        &self,
        receiver: Value,
        role: &str,
        verb: &str,
    ) -> Result<Value, RuntimeError> {
        let selector = resultive_message_for(role, verb).ok_or_else(|| {
            RuntimeError::new(
                "현재 결과 서술 문법은 `맨위 요소를 꺼낸 것이다` 또는 `맨위 요소를 꺼낸다`만 지원합니다.",
            )
        })?;
        self.send_resultive_message(receiver, selector)
    }

    fn execute_keyword_message(
        &mut self,
        receiver: Value,
        selector: &str,
        args: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        let selector = keyword_message_for_selector(selector).ok_or_else(|| {
            RuntimeError::new(format!(
                "`{}` 키워드 메시지는 아직 지원하지 않습니다.",
                selector
            ))
        })?;
        self.send_keyword_message(receiver, selector, args)
    }

    fn send_unary_message(
        &self,
        receiver: Value,
        selector: UnaryMessage,
    ) -> Result<Value, RuntimeError> {
        match (receiver, selector) {
            (Value::List(items), UnaryMessage::Length) => {
                Ok(Value::Int(items.borrow().len() as i64))
            }
            (Value::String(text), UnaryMessage::Length) => {
                Ok(Value::Int(text.chars().count() as i64))
            }
            (Value::Record(map), UnaryMessage::Length) => Ok(Value::Int(map.borrow().len() as i64)),
            (Value::Int(value), UnaryMessage::Square) => Ok(Value::Int(value * value)),
            (Value::Float(value), UnaryMessage::Square) => Ok(Value::Float(value * value)),
            (Value::List(_), UnaryMessage::Square) => {
                Err(RuntimeError::new("목록에는 `제곱` 속성이 없습니다."))
            }
            (Value::String(_), UnaryMessage::Square) => {
                Err(RuntimeError::new("문자열에는 `제곱` 속성이 없습니다."))
            }
            (Value::Record(_), UnaryMessage::Square) => {
                Err(RuntimeError::new("이 값에는 `제곱` 속성이 없습니다."))
            }
            (Value::Int(_), UnaryMessage::Length) => {
                Err(RuntimeError::new("정수에는 `길이` 속성이 없습니다."))
            }
            (Value::Float(_), UnaryMessage::Length) => {
                Err(RuntimeError::new("실수에는 `길이` 속성이 없습니다."))
            }
            (_, UnaryMessage::Length) => {
                Err(RuntimeError::new("이 값에는 `길이` 속성이 없습니다."))
            }
            (_, UnaryMessage::Square) => {
                Err(RuntimeError::new("이 값에는 `제곱` 속성이 없습니다."))
            }
        }
    }

    fn send_resultive_message(
        &self,
        receiver: Value,
        selector: ResultiveMessage,
    ) -> Result<Value, RuntimeError> {
        match (receiver, selector) {
            (Value::List(items), ResultiveMessage::PopTopElement) => items
                .borrow_mut()
                .pop()
                .ok_or_else(|| RuntimeError::new("빈 목록에서는 맨위 요소를 꺼낼 수 없습니다.")),
            (_, ResultiveMessage::PopTopElement) => Err(RuntimeError::new(
                "`맨위 요소를 꺼낸` 결과 서술은 목록에만 사용할 수 있습니다.",
            )),
        }
    }

    fn send_keyword_message(
        &mut self,
        receiver: Value,
        selector: KeywordMessage,
        args: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        match (receiver, selector) {
            (Value::List(items), KeywordMessage::Push) => {
                let [arg] = expect_arity::<1>("추가", args)?;
                items.borrow_mut().push(arg);
                Ok(())
            }
            (Value::Host(HostValue::Canvas), selector) => {
                let [arg] = expect_arity::<1>("그림판 메시지", args)?;
                self.execute_canvas_message(selector, arg)
            }
            (_, KeywordMessage::Push) => Err(RuntimeError::new(
                "`추가` 메시지는 목록에만 보낼 수 있습니다.",
            )),
            (_, KeywordMessage::CanvasClear) => Err(RuntimeError::new(
                "`지우기` 메시지는 그림판에만 보낼 수 있습니다.",
            )),
            (_, KeywordMessage::CanvasFillRect) => Err(RuntimeError::new(
                "`사각형채우기` 메시지는 그림판에만 보낼 수 있습니다.",
            )),
            (_, KeywordMessage::CanvasFillText) => Err(RuntimeError::new(
                "`글자쓰기` 메시지는 그림판에만 보낼 수 있습니다.",
            )),
            (_, KeywordMessage::CanvasDot) => Err(RuntimeError::new(
                "`점찍기` 메시지는 그림판에만 보낼 수 있습니다.",
            )),
        }
    }

    fn execute_canvas_message(
        &mut self,
        selector: KeywordMessage,
        arg: Value,
    ) -> Result<(), RuntimeError> {
        let selector_name = match selector {
            KeywordMessage::CanvasClear => "지우기",
            KeywordMessage::CanvasDot => "점찍기",
            KeywordMessage::CanvasFillRect => "사각형채우기",
            KeywordMessage::CanvasFillText => "글자쓰기",
            KeywordMessage::Push => "추가",
        };
        let record = expect_record(selector_name, arg)?;
        match selector {
            KeywordMessage::CanvasClear => {
                let background = expect_string_field(&record, &["배경색", "색"])?;
                self.begin_canvas_frame();
                self.current_canvas_commands
                    .push(CanvasCommand::Clear { background });
                Ok(())
            }
            KeywordMessage::CanvasDot => {
                let x = expect_number_field(&record, "x")?;
                let y = expect_number_field(&record, "y")?;
                let color = expect_string_field(&record, &["색"])?;
                let size = expect_number_field_or(record.get("크기"), 8.0, "크기")?;
                self.current_canvas_commands
                    .push(CanvasCommand::Dot { x, y, color, size });
                Ok(())
            }
            KeywordMessage::CanvasFillRect => {
                let x = expect_number_field(&record, "x")?;
                let y = expect_number_field(&record, "y")?;
                let width = expect_number_field(&record, "너비")?;
                let height = expect_number_field(&record, "높이")?;
                let color = expect_string_field(&record, &["색"])?;
                self.current_canvas_commands.push(CanvasCommand::FillRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                });
                Ok(())
            }
            KeywordMessage::CanvasFillText => {
                let text = expect_string_field(&record, &["글"])?;
                let x = expect_number_field(&record, "x")?;
                let y = expect_number_field(&record, "y")?;
                let color = expect_string_field(&record, &["색"])?;
                let size = expect_number_field(&record, "크기")?;
                self.current_canvas_commands.push(CanvasCommand::FillText {
                    text,
                    x,
                    y,
                    color,
                    size,
                });
                Ok(())
            }
            KeywordMessage::Push => Err(RuntimeError::new(
                "`그림판`은 `추가` 동작을 지원하지 않습니다.",
            )),
        }
    }

    fn begin_canvas_frame(&mut self) {
        if !self.current_canvas_commands.is_empty() {
            self.finish_canvas_frame();
        }
    }

    fn finish_canvas_frame(&mut self) {
        if self.current_canvas_commands.is_empty() {
            return;
        }

        let frame = CanvasFrame {
            commands: std::mem::take(&mut self.current_canvas_commands),
        };
        self.canvas_frames.push(frame.clone());
        self.events.push(ExecutionEvent::CanvasFrame { frame });
    }
}

impl Environment {
    fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Self {
            values: BTreeMap::new(),
            parent,
        }))
    }
}

impl Value {
    pub fn render(&self) -> String {
        match self {
            Value::Int(value) => value.to_string(),
            Value::Float(value) => {
                if value.fract() == 0.0 {
                    format!("{value:.1}")
                } else {
                    value.to_string()
                }
            }
            Value::Bool(true) => "참".to_string(),
            Value::Bool(false) => "거짓".to_string(),
            Value::String(value) => value.clone(),
            Value::None => "없음".to_string(),
            Value::List(values) => {
                let items = values
                    .borrow()
                    .iter()
                    .map(Value::render)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{items}]")
            }
            Value::Record(values) => {
                let items = values
                    .borrow()
                    .iter()
                    .map(|(key, value)| format!("{key}: {}", value.render()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {items} }}")
            }
            Value::Function(FunctionValue::Builtin(function)) => function.to_string(),
            Value::Function(FunctionValue::User(_)) => "<함수>".to_string(),
            Value::Host(HostValue::Canvas) => "<그림판>".to_string(),
        }
    }
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinFunction::Length => write!(f, "<내장 함수 길이>"),
            BuiltinFunction::Push => write!(f, "<내장 함수 추가>"),
            BuiltinFunction::PopLast => write!(f, "<내장 함수 마지막꺼내기>"),
            BuiltinFunction::ToString => write!(f, "<내장 함수 문자열로>"),
            BuiltinFunction::ToInt => write!(f, "<내장 함수 정수로>"),
            BuiltinFunction::ToFloat => write!(f, "<내장 함수 실수로>"),
        }
    }
}

impl BuiltinFunction {
    fn name(self) -> &'static str {
        match self {
            BuiltinFunction::Length => "길이",
            BuiltinFunction::Push => "추가",
            BuiltinFunction::PopLast => "마지막꺼내기",
            BuiltinFunction::ToString => "문자열로",
            BuiltinFunction::ToInt => "정수로",
            BuiltinFunction::ToFloat => "실수로",
        }
    }
}

fn install_builtins(env: &EnvRef) {
    let mut env = env.borrow_mut();
    env.values.insert(
        "길이".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::Length)),
    );
    env.values.insert(
        "추가".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::Push)),
    );
    env.values.insert(
        "마지막꺼내기".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::PopLast)),
    );
    env.values.insert(
        "문자열로".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::ToString)),
    );
    env.values.insert(
        "정수로".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::ToInt)),
    );
    env.values.insert(
        "실수로".into(),
        Value::Function(FunctionValue::Builtin(BuiltinFunction::ToFloat)),
    );
    env.values
        .insert("그림판".into(), Value::Host(HostValue::Canvas));
}

fn lookup_value(env: &EnvRef, name: &str) -> Result<Value, RuntimeError> {
    let mut current = Some(env.clone());
    while let Some(scope) = current {
        let scope_ref = scope.borrow();
        if let Some(value) = scope_ref.values.get(name) {
            return Ok(value.clone());
        }
        current = scope_ref.parent.clone();
    }

    Err(RuntimeError::new(format!(
        "`{}`은(는) 아직 정의되지 않았습니다.",
        name
    )))
}

fn assign_value(env: &EnvRef, name: &str, value: Value) -> Result<(), RuntimeError> {
    let mut current = Some(env.clone());
    while let Some(scope) = current {
        let parent = {
            let mut scope_ref = scope.borrow_mut();
            if scope_ref.values.contains_key(name) {
                scope_ref.values.insert(name.to_string(), value);
                return Ok(());
            }
            scope_ref.parent.clone()
        };
        current = parent;
    }

    Err(RuntimeError::new(format!(
        "`{}`를 바꿀 수 없습니다. 이 이름이 현재 스코프에 없습니다.",
        name
    )))
}

fn numeric_binary(
    left: Value,
    right: Value,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(a, b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(a, b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(a as f64, b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(a, b as f64))),
        _ => Err(RuntimeError::new(
            "숫자 연산은 정수 또는 실수에만 사용할 수 있습니다.",
        )),
    }
}

fn comparison_binary(
    left: Value,
    right: Value,
    op: impl FnOnce(f64, f64) -> bool,
) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(op(a as f64, b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(op(a, b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(op(a as f64, b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(op(a, b as f64))),
        _ => Err(RuntimeError::new(
            "비교 연산은 숫자에만 사용할 수 있습니다.",
        )),
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::None, Value::None) => true,
        (Value::Host(a), Value::Host(b)) => a == b,
        (Value::List(a), Value::List(b)) => {
            let a = a.borrow();
            let b = b.borrow();
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| values_equal(a, b))
        }
        (Value::Record(a), Value::Record(b)) => {
            let a = a.borrow();
            let b = b.borrow();
            a.len() == b.len()
                && a.iter().all(|(key, a_value)| {
                    b.get(key)
                        .is_some_and(|b_value| values_equal(a_value, b_value))
                })
        }
        _ => false,
    }
}

fn expect_record(name: &str, value: Value) -> Result<BTreeMap<String, Value>, RuntimeError> {
    match value {
        Value::Record(map) => Ok(map.borrow().clone()),
        _ => Err(RuntimeError::new(format!(
            "`{name}` 인수는 레코드여야 합니다."
        ))),
    }
}

fn expect_number_field(record: &BTreeMap<String, Value>, key: &str) -> Result<f64, RuntimeError> {
    match record.get(key) {
        Some(Value::Int(value)) => Ok(*value as f64),
        Some(Value::Float(value)) => Ok(*value),
        Some(_) => Err(RuntimeError::new(format!(
            "`{key}` 필드는 숫자여야 합니다."
        ))),
        None => Err(RuntimeError::new(format!("`{key}` 필드가 필요합니다."))),
    }
}

fn expect_number_field_or(
    value: Option<&Value>,
    default: f64,
    key: &str,
) -> Result<f64, RuntimeError> {
    match value {
        Some(Value::Int(value)) => Ok(*value as f64),
        Some(Value::Float(value)) => Ok(*value),
        Some(_) => Err(RuntimeError::new(format!(
            "`{key}` 필드는 숫자여야 합니다."
        ))),
        None => Ok(default),
    }
}

fn expect_string_field(
    record: &BTreeMap<String, Value>,
    keys: &[&str],
) -> Result<String, RuntimeError> {
    for key in keys {
        if let Some(value) = record.get(*key) {
            return match value {
                Value::String(value) => Ok(value.clone()),
                _ => Err(RuntimeError::new(format!(
                    "`{key}` 필드는 문자열이어야 합니다."
                ))),
            };
        }
    }

    Err(RuntimeError::new(format!(
        "`{}` 필드가 필요합니다.",
        keys.join("` 또는 `")
    )))
}

fn expect_sleep_seconds(value: Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Int(value) if value >= 0 => Ok(value as f64),
        Value::Float(value) if value.is_finite() && value >= 0.0 => Ok(value),
        Value::Int(_) | Value::Float(_) => {
            Err(RuntimeError::new("`쉬기` 시간은 0 이상의 값이어야 합니다."))
        }
        _ => Err(RuntimeError::new("`쉬기` 시간은 숫자여야 합니다.")),
    }
}

fn expect_arity<const N: usize>(name: &str, args: Vec<Value>) -> Result<[Value; N], RuntimeError> {
    args.try_into().map_err(|values: Vec<Value>| {
        RuntimeError::new(format!(
            "`{}` 함수 인수 개수가 맞지 않습니다. 기대: {}, 실제: {}",
            name,
            N,
            values.len()
        ))
    })
}
