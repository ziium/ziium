use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::{RunError, RuntimeError};
use crate::parser::{ParseMetadata, parse_source_with_metadata};
use crate::resolver::ResolverSession;
use crate::token::Span;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::rc::Rc;

type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: Vec<String>,
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
    spans: RuntimeSpanMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Length,
    Push,
    ToString,
    ToInt,
    ToFloat,
}

#[derive(Debug)]
struct Interpreter {
    globals: EnvRef,
    output: Vec<String>,
    runtime_spans: RuntimeSpanMap,
}

#[derive(Debug)]
struct Environment {
    values: BTreeMap<String, Value>,
    parent: Option<EnvRef>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeSpanMap {
    stmt_spans: HashMap<usize, Span>,
    expr_spans: HashMap<usize, Span>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeSpanCursor {
    statement_spans: Vec<Span>,
    statement_index: usize,
    expr_spans: Vec<Span>,
    expr_index: usize,
}

#[derive(Debug)]
enum ExecSignal {
    Continue,
    Return(Value),
}

pub fn interpret_program(program: &Program) -> Result<ExecutionResult, RuntimeError> {
    let mut session = InterpreterSession::new();
    session.interpret_program(program)
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
        program: &Program,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.interpreter.runtime_spans = RuntimeSpanMap::default();
        self.resolver
            .resolve_program(program)
            .map_err(|err| RuntimeError::with_span(err.message.clone(), err.span.clone()))?;
        self.interpreter.run_program(program)
    }

    pub fn run_source(&mut self, source: &str) -> Result<ExecutionResult, RunError> {
        let (program, metadata) = parse_source_with_metadata(source).map_err(RunError::from)?;
        self.interpreter.runtime_spans = RuntimeSpanMap::from_program(&program, &metadata);
        self.resolver
            .resolve_program_with_metadata(&program, &metadata)
            .map_err(crate::error::FrontendError::from)
            .map_err(RunError::from)?;
        self.interpreter
            .run_program(&program)
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
            runtime_spans: RuntimeSpanMap::default(),
        }
    }

    fn run_program(&mut self, program: &Program) -> Result<ExecutionResult, RuntimeError> {
        let output_start = self.output.len();
        let runtime_spans = self.runtime_spans.clone();
        match self.execute_block(&program.statements, self.globals.clone(), &runtime_spans)? {
            ExecSignal::Continue => Ok(ExecutionResult {
                output: self.output[output_start..].to_vec(),
            }),
            ExecSignal::Return(_) => Err(RuntimeError::new(
                "`돌려준다`는 함수 본문 안에서만 사용할 수 있습니다.",
            )),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: EnvRef,
        spans: &RuntimeSpanMap,
    ) -> Result<ExecSignal, RuntimeError> {
        for statement in statements {
            match self.execute_stmt(statement, env.clone(), spans)? {
                ExecSignal::Continue => {}
                signal @ ExecSignal::Return(_) => return Ok(signal),
            }
        }

        Ok(ExecSignal::Continue)
    }

    fn execute_stmt(
        &mut self,
        stmt: &Stmt,
        env: EnvRef,
        spans: &RuntimeSpanMap,
    ) -> Result<ExecSignal, RuntimeError> {
        let stmt_span = spans.stmt_span(stmt);
        match stmt {
            Stmt::Bind { name, value } => {
                if env.borrow().values.contains_key(name) {
                    return Err(RuntimeError::with_span(
                        format!("`{}`은(는) 현재 스코프에 이미 정의되어 있습니다.", name),
                        stmt_span,
                    ));
                }
                let value = self
                    .eval_expr(value, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                env.borrow_mut().values.insert(name.clone(), value);
                Ok(ExecSignal::Continue)
            }
            Stmt::Assign { name, value } => {
                let value = self
                    .eval_expr(value, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(stmt_span.clone()))?;
                assign_value(&env, name, value).map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Continue)
            }
            Stmt::Print { value } => {
                let value = self
                    .eval_expr(value, env, spans)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                self.output.push(value.render());
                Ok(ExecSignal::Continue)
            }
            Stmt::Return { value } => {
                let value = self
                    .eval_expr(value, env, spans)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Return(value))
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition_span = spans.expr_span(condition).or_else(|| stmt_span.clone());
                let condition = self
                    .eval_expr(condition, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(condition_span.clone()))?;
                match condition {
                    Value::Bool(true) => self.execute_block(then_block, env, spans),
                    Value::Bool(false) => {
                        if let Some(else_block) = else_block {
                            self.execute_block(else_block, env, spans)
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
            Stmt::While { condition, body } => {
                let condition_span = spans.expr_span(condition).or_else(|| stmt_span.clone());
                loop {
                    let condition_value = self
                        .eval_expr(condition, env.clone(), spans)
                        .map_err(|err| err.with_fallback_span(condition_span.clone()))?;
                    match condition_value {
                        Value::Bool(true) => match self.execute_block(body, env.clone(), spans)? {
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
            Stmt::FunctionDef { name, params, body } => {
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
                    spans: RuntimeSpanMap::from_cloned_statements(
                        body,
                        cloned_body.as_ref(),
                        spans,
                    ),
                }));
                env.borrow_mut().values.insert(name.clone(), function);
                Ok(ExecSignal::Continue)
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr, env, spans)
                    .map_err(|err| err.with_fallback_span(stmt_span))?;
                Ok(ExecSignal::Continue)
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: EnvRef,
        spans: &RuntimeSpanMap,
    ) -> Result<Value, RuntimeError> {
        let expr_span = spans.expr_span(expr);
        match expr {
            Expr::Name(name) => {
                lookup_value(&env, name).map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Int(raw) => raw.parse::<i64>().map(Value::Int).map_err(|_| {
                RuntimeError::with_span(
                    format!("정수 리터럴 `{}`를 해석할 수 없습니다.", raw),
                    expr_span,
                )
            }),
            Expr::Float(raw) => raw.parse::<f64>().map(Value::Float).map_err(|_| {
                RuntimeError::with_span(
                    format!("실수 리터럴 `{}`를 해석할 수 없습니다.", raw),
                    expr_span,
                )
            }),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::None => Ok(Value::None),
            Expr::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(
                        self.eval_expr(item, env.clone(), spans)
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                Ok(Value::List(Rc::new(RefCell::new(values))))
            }
            Expr::Record(entries) => {
                let mut map = BTreeMap::new();
                for entry in entries {
                    map.insert(
                        entry.key.clone(),
                        self.eval_expr(&entry.value, env.clone(), spans)
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                Ok(Value::Record(Rc::new(RefCell::new(map))))
            }
            Expr::Unary { op, expr } => {
                let value = self
                    .eval_expr(expr, env, spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_unary(*op, value)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Binary { left, op, right } => {
                let left = self
                    .eval_expr(left, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let right = self
                    .eval_expr(right, env, spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_binary(left, *op, right)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Call { callee, args } => {
                let callee = self
                    .eval_expr(callee, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(
                        self.eval_expr(arg, env.clone(), spans)
                            .map_err(|err| err.with_fallback_span(expr_span.clone()))?,
                    );
                }
                self.call_value(callee, arg_values, expr_span.clone())
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Index { base, index } => {
                let base = self
                    .eval_expr(base, env.clone(), spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                let index = self
                    .eval_expr(index, env, spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_index(base, index)
                    .map_err(|err| err.with_fallback_span(expr_span))
            }
            Expr::Property { base, name } => {
                let base = self
                    .eval_expr(base, env, spans)
                    .map_err(|err| err.with_fallback_span(expr_span.clone()))?;
                self.eval_property(base, name)
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
                    .execute_block(function.body.as_ref(), frame, &function.spans)
                    .map_err(|err| err.with_call_frame(function.name.clone(), call_span))?
                {
                    ExecSignal::Continue => Ok(Value::None),
                    ExecSignal::Return(value) => Ok(value),
                }
            }
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

    fn eval_property(&self, base: Value, name: &str) -> Result<Value, RuntimeError> {
        match base {
            Value::Record(map) => {
                map.borrow().get(name).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("이 값에는 `{}` 속성이 없습니다.", name))
                })
            }
            _ => Err(RuntimeError::new(
                "`의` 속성 접근은 레코드에만 사용할 수 있습니다.",
            )),
        }
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
        }
    }
}

impl RuntimeSpanMap {
    fn from_program(program: &Program, metadata: &ParseMetadata) -> Self {
        let mut map = Self::default();
        let mut cursor = RuntimeSpanCursor::from_metadata(metadata);
        for statement in &program.statements {
            map.collect_stmt(statement, &mut cursor);
        }
        map
    }

    fn from_cloned_statements(original: &[Stmt], cloned: &[Stmt], source: &RuntimeSpanMap) -> Self {
        let mut map = Self::default();
        for (original_stmt, cloned_stmt) in original.iter().zip(cloned.iter()) {
            map.copy_stmt_spans(original_stmt, cloned_stmt, source);
        }
        map
    }

    fn stmt_span(&self, stmt: &Stmt) -> Option<Span> {
        self.stmt_spans
            .get(&(stmt as *const Stmt as usize))
            .cloned()
    }

    fn expr_span(&self, expr: &Expr) -> Option<Span> {
        self.expr_spans
            .get(&(expr as *const Expr as usize))
            .cloned()
    }

    fn collect_stmt(&mut self, stmt: &Stmt, cursor: &mut RuntimeSpanCursor) {
        match stmt {
            Stmt::Bind { value, .. }
            | Stmt::Assign { value, .. }
            | Stmt::Print { value }
            | Stmt::Return { value }
            | Stmt::Expr(value) => self.collect_expr(value, cursor),
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.collect_expr(condition, cursor);
                for statement in then_block {
                    self.collect_stmt(statement, cursor);
                }
                if let Some(else_block) = else_block {
                    for statement in else_block {
                        self.collect_stmt(statement, cursor);
                    }
                }
            }
            Stmt::While { condition, body } => {
                self.collect_expr(condition, cursor);
                for statement in body {
                    self.collect_stmt(statement, cursor);
                }
            }
            Stmt::FunctionDef { body, .. } => {
                for statement in body {
                    self.collect_stmt(statement, cursor);
                }
            }
        }

        if let Some(span) = cursor.next_statement_span() {
            self.stmt_spans.insert(stmt as *const Stmt as usize, span);
        }
    }

    fn collect_expr(&mut self, expr: &Expr, cursor: &mut RuntimeSpanCursor) {
        match expr {
            Expr::Name(_)
            | Expr::Int(_)
            | Expr::Float(_)
            | Expr::String(_)
            | Expr::Bool(_)
            | Expr::None => {}
            Expr::List(items) => {
                for item in items {
                    self.collect_expr(item, cursor);
                }
            }
            Expr::Record(entries) => {
                for entry in entries {
                    self.collect_expr(&entry.value, cursor);
                }
            }
            Expr::Unary { expr, .. } => self.collect_expr(expr, cursor),
            Expr::Binary { left, right, .. } => {
                self.collect_expr(left, cursor);
                self.collect_expr(right, cursor);
            }
            Expr::Call { callee, args } => {
                self.collect_expr(callee, cursor);
                for arg in args {
                    self.collect_expr(arg, cursor);
                }
            }
            Expr::Index { base, index } => {
                self.collect_expr(base, cursor);
                self.collect_expr(index, cursor);
            }
            Expr::Property { base, .. } => self.collect_expr(base, cursor),
        }

        if let Some(span) = cursor.next_expr_span() {
            self.expr_spans.insert(expr as *const Expr as usize, span);
        }
    }

    fn copy_stmt_spans(&mut self, original: &Stmt, cloned: &Stmt, source: &RuntimeSpanMap) {
        match (original, cloned) {
            (
                Stmt::Bind {
                    value: original_value,
                    ..
                },
                Stmt::Bind {
                    value: cloned_value,
                    ..
                },
            )
            | (
                Stmt::Assign {
                    value: original_value,
                    ..
                },
                Stmt::Assign {
                    value: cloned_value,
                    ..
                },
            )
            | (
                Stmt::Print {
                    value: original_value,
                },
                Stmt::Print {
                    value: cloned_value,
                },
            )
            | (
                Stmt::Return {
                    value: original_value,
                },
                Stmt::Return {
                    value: cloned_value,
                },
            )
            | (Stmt::Expr(original_value), Stmt::Expr(cloned_value)) => {
                self.copy_expr_spans(original_value, cloned_value, source)
            }
            (
                Stmt::If {
                    condition: original_condition,
                    then_block: original_then,
                    else_block: original_else,
                },
                Stmt::If {
                    condition: cloned_condition,
                    then_block: cloned_then,
                    else_block: cloned_else,
                },
            ) => {
                self.copy_expr_spans(original_condition, cloned_condition, source);
                for (original_stmt, cloned_stmt) in original_then.iter().zip(cloned_then.iter()) {
                    self.copy_stmt_spans(original_stmt, cloned_stmt, source);
                }
                if let (Some(original_else), Some(cloned_else)) = (original_else, cloned_else) {
                    for (original_stmt, cloned_stmt) in original_else.iter().zip(cloned_else.iter())
                    {
                        self.copy_stmt_spans(original_stmt, cloned_stmt, source);
                    }
                }
            }
            (
                Stmt::While {
                    condition: original_condition,
                    body: original_body,
                },
                Stmt::While {
                    condition: cloned_condition,
                    body: cloned_body,
                },
            ) => {
                self.copy_expr_spans(original_condition, cloned_condition, source);
                for (original_stmt, cloned_stmt) in original_body.iter().zip(cloned_body.iter()) {
                    self.copy_stmt_spans(original_stmt, cloned_stmt, source);
                }
            }
            (
                Stmt::FunctionDef {
                    body: original_body,
                    ..
                },
                Stmt::FunctionDef {
                    body: cloned_body, ..
                },
            ) => {
                for (original_stmt, cloned_stmt) in original_body.iter().zip(cloned_body.iter()) {
                    self.copy_stmt_spans(original_stmt, cloned_stmt, source);
                }
            }
            _ => {}
        }

        if let Some(span) = source.stmt_span(original) {
            self.stmt_spans.insert(cloned as *const Stmt as usize, span);
        }
    }

    fn copy_expr_spans(&mut self, original: &Expr, cloned: &Expr, source: &RuntimeSpanMap) {
        match (original, cloned) {
            (Expr::List(original_items), Expr::List(cloned_items)) => {
                for (original_item, cloned_item) in original_items.iter().zip(cloned_items.iter()) {
                    self.copy_expr_spans(original_item, cloned_item, source);
                }
            }
            (Expr::Record(original_entries), Expr::Record(cloned_entries)) => {
                for (original_entry, cloned_entry) in
                    original_entries.iter().zip(cloned_entries.iter())
                {
                    self.copy_expr_spans(&original_entry.value, &cloned_entry.value, source);
                }
            }
            (
                Expr::Unary {
                    expr: original_expr,
                    ..
                },
                Expr::Unary {
                    expr: cloned_expr, ..
                },
            ) => self.copy_expr_spans(original_expr, cloned_expr, source),
            (
                Expr::Binary {
                    left: original_left,
                    right: original_right,
                    ..
                },
                Expr::Binary {
                    left: cloned_left,
                    right: cloned_right,
                    ..
                },
            ) => {
                self.copy_expr_spans(original_left, cloned_left, source);
                self.copy_expr_spans(original_right, cloned_right, source);
            }
            (
                Expr::Call {
                    callee: original_callee,
                    args: original_args,
                },
                Expr::Call {
                    callee: cloned_callee,
                    args: cloned_args,
                },
            ) => {
                self.copy_expr_spans(original_callee, cloned_callee, source);
                for (original_arg, cloned_arg) in original_args.iter().zip(cloned_args.iter()) {
                    self.copy_expr_spans(original_arg, cloned_arg, source);
                }
            }
            (
                Expr::Index {
                    base: original_base,
                    index: original_index,
                },
                Expr::Index {
                    base: cloned_base,
                    index: cloned_index,
                },
            ) => {
                self.copy_expr_spans(original_base, cloned_base, source);
                self.copy_expr_spans(original_index, cloned_index, source);
            }
            (
                Expr::Property {
                    base: original_base,
                    ..
                },
                Expr::Property {
                    base: cloned_base, ..
                },
            ) => self.copy_expr_spans(original_base, cloned_base, source),
            _ => {}
        }

        if let Some(span) = source.expr_span(original) {
            self.expr_spans.insert(cloned as *const Expr as usize, span);
        }
    }
}

impl RuntimeSpanCursor {
    fn from_metadata(metadata: &ParseMetadata) -> Self {
        Self {
            statement_spans: metadata.statement_spans.clone(),
            statement_index: 0,
            expr_spans: metadata.expr_spans.clone(),
            expr_index: 0,
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
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinFunction::Length => write!(f, "<내장 함수 길이>"),
            BuiltinFunction::Push => write!(f, "<내장 함수 추가>"),
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
