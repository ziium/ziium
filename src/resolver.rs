use crate::ast::{Expr, Program, RecordEntry, Stmt};
use crate::error::ResolveError;
use crate::parser::ParseMetadata;
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
    metadata: ResolveMetadataCursor,
}

#[derive(Debug, Clone, Default)]
struct ResolveMetadataCursor {
    declaration_spans: Vec<Span>,
    declaration_index: usize,
    assign_target_spans: Vec<Span>,
    assign_target_index: usize,
    name_expr_spans: Vec<Span>,
    name_expr_index: usize,
    return_spans: Vec<Span>,
    return_index: usize,
}

pub fn resolve_program(program: &Program) -> Result<(), ResolveError> {
    let mut session = ResolverSession::new();
    session.resolve_program(program)
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

    pub fn resolve_program(&mut self, program: &Program) -> Result<(), ResolveError> {
        self.resolve_program_with_metadata(program, &ParseMetadata::default())
    }

    pub(crate) fn resolve_program_with_metadata(
        &mut self,
        program: &Program,
        metadata: &ParseMetadata,
    ) -> Result<(), ResolveError> {
        let mut globals = self.globals.clone();
        globals
            .defined_eventually
            .extend(collect_unconditional_names(&program.statements));

        let mut resolver = Resolver {
            scopes: vec![globals],
            function_depth: 0,
            metadata: ResolveMetadataCursor::from_metadata(metadata),
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
            Stmt::Bind { name, value } => {
                self.resolve_expr(value)?;
                self.declare(name)
            }
            Stmt::Assign { name, value } => {
                let span = self.metadata.next_assign_target_span();
                self.resolve_name(name, span)?;
                self.resolve_expr(value)
            }
            Stmt::Print { value } | Stmt::Expr(value) => self.resolve_expr(value),
            Stmt::KeywordMessage { receiver, arg, .. } => {
                self.resolve_expr(receiver)?;
                self.resolve_expr(arg)
            }
            Stmt::Return { value } => {
                let span = self.metadata.next_return_span();
                if self.function_depth == 0 {
                    return Err(ResolveError::with_span(
                        "`돌려준다`는 함수 본문 안에서만 사용할 수 있습니다.",
                        span,
                    ));
                }
                self.resolve_expr(value)
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => self.resolve_if(condition, then_block, else_block.as_deref()),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_loop(body)
            }
            Stmt::FunctionDef { name, params, body } => {
                self.declare(name)?;
                self.resolve_function(params, body)
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Name(name) => {
                let span = self.metadata.next_name_expr_span();
                self.resolve_name(name, span)
            }
            Expr::Int(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::None => Ok(()),
            Expr::List(items) => {
                for item in items {
                    self.resolve_expr(item)?;
                }
                Ok(())
            }
            Expr::Record(entries) => {
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
            Expr::Call { callee, args } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expr::Index { base, index } => {
                self.resolve_expr(base)?;
                self.resolve_expr(index)
            }
            Expr::Property { base, .. } => self.resolve_expr(base),
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

    fn resolve_function(&mut self, params: &[String], body: &[Stmt]) -> Result<(), ResolveError> {
        self.function_depth += 1;
        let eventual_names = collect_unconditional_names(body);
        self.scopes.push(Scope {
            defined_now: HashSet::new(),
            defined_eventually: eventual_names,
        });

        let result = (|| {
            for param in params {
                self.declare(param)?;
            }
            self.resolve_statements(body)
        })();

        self.scopes.pop();
        self.function_depth -= 1;
        result
    }

    fn declare(&mut self, name: &str) -> Result<(), ResolveError> {
        let span = self.metadata.next_declaration_span();
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

impl ResolveMetadataCursor {
    fn from_metadata(metadata: &ParseMetadata) -> Self {
        Self {
            declaration_spans: metadata.declaration_spans.clone(),
            declaration_index: 0,
            assign_target_spans: metadata.assign_target_spans.clone(),
            assign_target_index: 0,
            name_expr_spans: metadata.name_expr_spans.clone(),
            name_expr_index: 0,
            return_spans: metadata.return_spans.clone(),
            return_index: 0,
        }
    }

    fn next_declaration_span(&mut self) -> Option<Span> {
        next_span(&self.declaration_spans, &mut self.declaration_index)
    }

    fn next_assign_target_span(&mut self) -> Option<Span> {
        next_span(&self.assign_target_spans, &mut self.assign_target_index)
    }

    fn next_name_expr_span(&mut self) -> Option<Span> {
        next_span(&self.name_expr_spans, &mut self.name_expr_index)
    }

    fn next_return_span(&mut self) -> Option<Span> {
        next_span(&self.return_spans, &mut self.return_index)
    }
}

fn next_span(spans: &[Span], index: &mut usize) -> Option<Span> {
    let span = spans.get(*index).cloned();
    if span.is_some() {
        *index += 1;
    }
    span
}

fn builtin_names() -> HashSet<String> {
    ["길이", "추가", "문자열로", "정수로", "실수로"]
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
            | Stmt::KeywordMessage { .. }
            | Stmt::Return { .. }
            | Stmt::Expr(_)
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
