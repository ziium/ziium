use crate::ast;
use crate::error::ResolveError;
use crate::hir::{self, Expr, Program, RecordEntry, SendSelector, Stmt};
use crate::token::Span;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ResolverSession {
    globals: Scope,
}

#[derive(Debug, Clone)]
struct Scope {
    defined_now: HashSet<String>,
    defined_eventually: HashSet<String>,
}

struct Resolver {
    scopes: Vec<Scope>,
    function_depth: usize,
}

pub fn resolve_program(program: &ast::Program) -> Result<(), ResolveError> {
    let mut session = ResolverSession::new();
    session.resolve_program(program)
}

pub fn resolve_hir_program(program: &Program) -> Result<(), ResolveError> {
    let mut session = ResolverSession::new();
    session.resolve_hir_program(program)
}

impl ResolverSession {
    pub fn new() -> Self {
        let builtins = builtin_names();
        Self {
            globals: Scope {
                defined_now: builtins.clone(),
                defined_eventually: builtins,
            },
        }
    }

    pub fn resolve_program(&mut self, program: &ast::Program) -> Result<(), ResolveError> {
        let hir_program = hir::lower_program(program);
        self.resolve_hir_program(&hir_program)
    }

    pub fn resolve_hir_program(&mut self, program: &Program) -> Result<(), ResolveError> {
        let mut globals = self.globals.clone();
        globals
            .defined_eventually
            .extend(collect_unconditional_names(&program.statements));

        let mut resolver = Resolver {
            scopes: vec![globals],
            function_depth: 0,
        };
        resolver.resolve_statements(&program.statements)?;
        self.globals = resolver
            .scopes
            .into_iter()
            .next()
            .map(|mut scope| {
                scope.defined_eventually = scope.defined_now.clone();
                scope
            })
            .expect("resolver should keep global scope");
        Ok(())
    }
}

impl Default for ResolverSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    fn resolve_statements(&mut self, statements: &[Stmt]) -> Result<(), ResolveError> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Bind {
                name,
                name_span,
                value,
                ..
            } => {
                self.resolve_expr(value)?;
                self.declare(name, name_span.clone())
            }
            Stmt::Assign {
                name,
                target_span,
                value,
                ..
            } => {
                self.resolve_name(name, target_span.clone())?;
                self.resolve_expr(value)
            }
            Stmt::Print { value, .. }
            | Stmt::Sleep {
                duration_seconds: value,
                ..
            }
            | Stmt::Expr { expr: value, .. } => self.resolve_expr(value),
            Stmt::Send {
                receiver, args, ..
            } => {
                self.resolve_expr(receiver)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Stmt::Return {
                value, keyword_span, ..
            } => {
                if self.function_depth == 0 {
                    return Err(ResolveError::with_span(
                        "`돌려준다`는 함수 본문 안에서만 사용할 수 있습니다.",
                        keyword_span.clone(),
                    ));
                }
                self.resolve_expr(value)
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => self.resolve_if(condition, then_block, else_block.as_deref()),
            Stmt::While {
                condition, body, ..
            } => {
                self.resolve_expr(condition)?;
                self.resolve_loop(body)
            }
            Stmt::FunctionDef {
                name,
                name_span,
                params,
                param_spans,
                body,
                ..
            } => {
                self.declare(name, name_span.clone())?;
                self.resolve_function(params, param_spans, body)
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Name { name, span } => self.resolve_name(name, span.clone()),
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Bool { .. }
            | Expr::None { .. } => Ok(()),
            Expr::List { items, .. } => {
                for item in items {
                    self.resolve_expr(item)?;
                }
                Ok(())
            }
            Expr::Record { entries, .. } => {
                for RecordEntry { value, .. } in entries {
                    self.resolve_expr(value)?;
                }
                Ok(())
            }
            Expr::Unary { expr, .. } => self.resolve_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call { callee, args, .. } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expr::Send {
                receiver,
                selector,
                args,
                span,
            } => {
                self.resolve_expr(receiver)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                if let SendSelector::Transform(callee) = selector {
                    self.resolve_name(callee, span.clone())?;
                }
                Ok(())
            }
            Expr::Index { base, index, .. } => {
                self.resolve_expr(base)?;
                self.resolve_expr(index)
            }
        }
    }

    fn resolve_if(
        &mut self,
        condition: &Expr,
        then_block: &[Stmt],
        else_block: Option<&[Stmt]>,
    ) -> Result<(), ResolveError> {
        self.resolve_expr(condition)?;

        let before_scope = self
            .scopes
            .last()
            .cloned()
            .expect("resolver should have a scope");

        let then_scope = self.resolve_branch_scope(then_block)?;
        let else_scope = if let Some(else_block) = else_block {
            Some(self.resolve_branch_scope(else_block)?)
        } else {
            None
        };

        let merged = merge_if_scope(&before_scope, &then_scope, else_scope.as_ref());
        *self
            .scopes
            .last_mut()
            .expect("resolver should have a scope") = merged;
        Ok(())
    }

    fn resolve_loop(&mut self, body: &[Stmt]) -> Result<(), ResolveError> {
        self.resolve_branch_scope(body).map(|_| ())
    }

    fn resolve_branch_scope(&mut self, statements: &[Stmt]) -> Result<Scope, ResolveError> {
        let saved_scope = self
            .scopes
            .last()
            .cloned()
            .expect("resolver should have a scope");

        self.prepare_current_scope_for_block(statements);
        let result = self.resolve_statements(statements);
        let branch_scope = self
            .scopes
            .last()
            .cloned()
            .ok_or_else(|| ResolveError::new("resolver branch lost its scope"))?;
        *self
            .scopes
            .last_mut()
            .expect("resolver should have a scope") = saved_scope;

        result.map(|_| branch_scope)
    }

    fn prepare_current_scope_for_block(&mut self, statements: &[Stmt]) {
        if let Some(scope) = self.scopes.last_mut() {
            scope
                .defined_eventually
                .extend(collect_unconditional_names(statements));
        }
    }

    fn resolve_function(
        &mut self,
        params: &[String],
        param_spans: &[Option<Span>],
        body: &[Stmt],
    ) -> Result<(), ResolveError> {
        self.function_depth += 1;
        let eventual_names = collect_unconditional_names(body);
        self.scopes.push(Scope {
            defined_now: HashSet::new(),
            defined_eventually: eventual_names,
        });

        let result = (|| {
            for (param, span) in params.iter().zip(param_spans.iter()) {
                self.declare(param, span.clone())?;
            }
            self.resolve_statements(body)
        })();

        self.scopes.pop();
        self.function_depth -= 1;
        result
    }

    fn declare(&mut self, name: &str, span: Option<Span>) -> Result<(), ResolveError> {
        let current_scope = self
            .scopes
            .last_mut()
            .expect("resolver should have a current scope");

        if current_scope.defined_now.contains(name) {
            return Err(ResolveError::with_span(
                format!("`{}`은(는) 현재 스코프에 이미 정의되어 있습니다.", name),
                span,
            ));
        }

        current_scope.defined_now.insert(name.to_string());
        current_scope.defined_eventually.insert(name.to_string());
        Ok(())
    }

    fn resolve_name(&self, name: &str, span: Option<Span>) -> Result<(), ResolveError> {
        let mut scopes = self.scopes.iter().rev();
        if scopes
            .next()
            .is_some_and(|scope| scope.defined_now.contains(name))
        {
            return Ok(());
        }

        if scopes.any(|scope| {
            scope.defined_now.contains(name) || scope.defined_eventually.contains(name)
        }) {
            return Ok(());
        }

        Err(ResolveError::with_span(
            format!("`{}`은(는) 아직 정의되지 않았습니다.", name),
            span,
        ))
    }
}

fn builtin_names() -> HashSet<String> {
    [
        "길이",
        "추가",
        "마지막꺼내기",
        "문자열로",
        "정수로",
        "실수로",
        "그림판",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn collect_unconditional_names(statements: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();

    for statement in statements {
        match statement {
            Stmt::Bind { name, .. } | Stmt::FunctionDef { name, .. } => {
                names.insert(name.clone());
            }
            Stmt::Assign { .. }
            | Stmt::Print { .. }
            | Stmt::Sleep { .. }
            | Stmt::Send { .. }
            | Stmt::Return { .. }
            | Stmt::Expr { .. }
            | Stmt::If { .. }
            | Stmt::While { .. } => {}
        }
    }

    names
}

fn merge_if_scope(before: &Scope, then_scope: &Scope, else_scope: Option<&Scope>) -> Scope {
    let mut defined_now = before.defined_now.clone();

    if let Some(else_scope) = else_scope {
        for name in then_scope
            .defined_now
            .intersection(&else_scope.defined_now)
            .filter(|name| !before.defined_now.contains(*name))
        {
            defined_now.insert(name.clone());
        }
    }

    Scope {
        defined_eventually: defined_now.clone(),
        defined_now,
    }
}
