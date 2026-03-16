use crate::ast::{BinaryOp, Expr, Program, RecordEntry, Stmt, UnaryOp};
use crate::error::{FrontendError, ParseError};
use crate::lexer::lex;
use crate::normalizer::normalize_tokens;
use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone, Default)]
pub(crate) struct ParseMetadata {
    pub declaration_spans: Vec<Span>,
    pub assign_target_spans: Vec<Span>,
    pub name_expr_spans: Vec<Span>,
    pub return_spans: Vec<Span>,
    pub statement_spans: Vec<Span>,
    pub expr_spans: Vec<Span>,
}

pub fn parse_source(source: &str) -> Result<Program, FrontendError> {
    let tokens = lex(source)?;
    parse_tokens(tokens).map_err(FrontendError::from)
}

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Program, ParseError> {
    parse_tokens_with_metadata(tokens).map(|(program, _)| program)
}

pub(crate) fn parse_source_with_metadata(
    source: &str,
) -> Result<(Program, ParseMetadata), FrontendError> {
    let tokens = lex(source)?;
    parse_tokens_with_metadata(tokens).map_err(FrontendError::from)
}

pub(crate) fn parse_tokens_with_metadata(
    tokens: Vec<Token>,
) -> Result<(Program, ParseMetadata), ParseError> {
    Parser::new(normalize_tokens(tokens)).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    metadata: ParseMetadata,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            metadata: ParseMetadata::default(),
        }
    }

    fn parse_program(&mut self) -> Result<(Program, ParseMetadata), ParseError> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.at(TokenKind::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok((Program { statements }, std::mem::take(&mut self.metadata)))
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let statement_span = self.current_span();
        let stmt = if self.is_function_def_start() {
            self.parse_function_def()?
        } else if self.is_bind_start() {
            self.parse_bind()?
        } else if self.is_assign_start() {
            self.parse_assign()?
        } else {
            let expr = self.parse_expression(0)?;

            if self.match_kind(TokenKind::If) {
                let then_block =
                    self.parse_indented_block("`이면` 뒤에는 들여쓴 블록이 와야 합니다.")?;
                let else_block = if self.match_kind(TokenKind::Else) {
                    Some(self.parse_indented_block("`아니면` 뒤에는 들여쓴 블록이 와야 합니다.")?)
                } else {
                    None
                };

                Stmt::If {
                    condition: expr,
                    then_block,
                    else_block,
                }
            } else if self.match_kind(TokenKind::In) {
                self.expect(
                    TokenKind::During,
                    "`인` 뒤에는 `동안`이 와야 반복문이 됩니다.",
                )?;
                let body =
                    self.parse_indented_block("`인 동안` 뒤에는 들여쓴 블록이 와야 합니다.")?;
                Stmt::While {
                    condition: expr,
                    body,
                }
            } else if self.match_kind(TokenKind::Locative) {
                self.parse_keyword_message(expr)?
            } else if self.match_kind(TokenKind::Object) {
                if self.match_kind(TokenKind::Print) {
                    let stmt = Stmt::Print { value: expr };
                    self.consume_optional_period();
                    stmt
                } else if let Some(token) = self.match_token(TokenKind::Return) {
                    self.metadata.return_spans.push(token.span);
                    let stmt = Stmt::Return { value: expr };
                    self.consume_optional_period();
                    stmt
                } else {
                    return Err(self.error_here(
                        "`을` 또는 `를` 뒤에는 `출력한다` 또는 `돌려준다`가 와야 합니다.",
                    ));
                }
            } else if matches!(expr, Expr::Call { .. }) {
                let stmt = Stmt::Expr(expr);
                self.consume_optional_period();
                stmt
            } else {
                return Err(self.error_here("문장을 해석할 수 없습니다. 바인딩, 출력, 조건문, 반복문, 함수 정의, 키워드 메시지 중 하나를 기대했습니다."));
            }
        };

        if let Some(span) = statement_span {
            self.metadata.statement_spans.push(span);
        }

        Ok(stmt)
    }

    fn parse_keyword_message(&mut self, receiver: Expr) -> Result<Stmt, ParseError> {
        let arg = self.parse_expression(0)?;
        let selector = self.expect_ident_token("`에` 뒤 키워드 메시지 이름이 필요합니다.")?;

        if selector.lexeme != "추가" {
            return Err(ParseError::new(
                "`에` 뒤 키워드 메시지로는 현재 `추가`만 지원합니다.",
                Some(selector.span),
            ));
        }

        self.consume_optional_period();
        Ok(Stmt::KeywordMessage {
            receiver,
            selector: selector.lexeme,
            arg,
        })
    }

    fn parse_bind(&mut self) -> Result<Stmt, ParseError> {
        let name_token = self.expect_ident_token("바인딩 이름이 필요합니다.")?;
        self.metadata
            .declaration_spans
            .push(name_token.span.clone());
        let name = name_token.lexeme;
        self.expect(
            TokenKind::Topic,
            "바인딩 이름 뒤에는 `은` 또는 `는`이 와야 합니다.",
        )?;
        let value = self.parse_expression(0)?;
        self.expect(TokenKind::Copula, "바인딩 문장은 `이다`로 끝나야 합니다.")?;
        self.consume_optional_period();
        Ok(Stmt::Bind { name, value })
    }

    fn parse_assign(&mut self) -> Result<Stmt, ParseError> {
        let name_token = self.expect_ident_token("재대입 대상 이름이 필요합니다.")?;
        self.metadata
            .assign_target_spans
            .push(name_token.span.clone());
        let name = name_token.lexeme;
        self.expect(
            TokenKind::Object,
            "재대입 대상 뒤에는 `을` 또는 `를`이 와야 합니다.",
        )?;
        let value = self.parse_expression(0)?;
        self.expect(
            TokenKind::Direction,
            "재대입 값 뒤에는 `로` 또는 `으로`가 와야 합니다.",
        )?;
        self.expect(TokenKind::Change, "재대입 문장은 `바꾼다`로 끝나야 합니다.")?;
        self.consume_optional_period();
        Ok(Stmt::Assign { name, value })
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        let name_token = self.expect_ident_token("함수 이름이 필요합니다.")?;
        self.metadata
            .declaration_spans
            .push(name_token.span.clone());
        let name = name_token.lexeme;
        self.expect(TokenKind::Function, "함수 정의에는 `함수`가 필요합니다.")?;
        self.expect(
            TokenKind::FunctionTopic,
            "함수 정의는 `<이름> 함수는` 형식을 따라야 합니다.",
        )?;

        let params = if self.match_kind(TokenKind::Nothing) {
            self.expect(
                TokenKind::ReceiveNot,
                "`아무것도` 뒤에는 `받지`가 와야 합니다.",
            )?;
            self.expect(TokenKind::ReceiveNeg, "`받지` 뒤에는 `않아`가 와야 합니다.")?;
            Vec::new()
        } else {
            let first_param = self.expect_ident_token("첫 번째 매개변수 이름이 필요합니다.")?;
            self.metadata
                .declaration_spans
                .push(first_param.span.clone());
            let mut params = vec![first_param.lexeme];
            while self.match_kind(TokenKind::Comma) {
                let param_token = self.expect_ident_token("쉼표 뒤 매개변수 이름이 필요합니다.")?;
                self.metadata
                    .declaration_spans
                    .push(param_token.span.clone());
                params.push(param_token.lexeme);
            }
            self.expect(
                TokenKind::Object,
                "매개변수 목록 뒤에는 `을` 또는 `를`이 와야 합니다.",
            )?;
            self.expect(TokenKind::Receive, "함수 헤더는 `받아`로 끝나야 합니다.")?;
            params
        };

        let body = self.parse_indented_block("함수 본문은 들여쓴 블록이어야 합니다.")?;
        Ok(Stmt::FunctionDef { name, params, body })
    }

    fn parse_indented_block(&mut self, context: &str) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenKind::Newline, context)?;
        self.expect(TokenKind::Indent, context)?;

        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(TokenKind::Dedent, "블록이 올바르게 닫혀야 합니다.")?;

        if statements.is_empty() {
            return Err(self.error_here("빈 블록은 아직 허용하지 않습니다."));
        }

        Ok(statements)
    }

    fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let Some((op, precedence)) = self.current_binary_op() else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            let operator_span = self
                .advance()
                .expect("current_binary_op should only be present when a token exists")
                .span;
            let right = self.parse_expression(precedence + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
            self.metadata.expr_spans.push(operator_span);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.match_token(TokenKind::Minus) {
            let expr = self.parse_unary()?;
            let expr = Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
            };
            self.metadata.expr_spans.push(token.span);
            return Ok(expr);
        }

        if let Some(token) = self.match_token(TokenKind::Not) {
            let expr = self.parse_unary()?;
            let expr = Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            };
            self.metadata.expr_spans.push(token.span);
            return Ok(expr);
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if let Some(token) = self.match_token(TokenKind::LParen) {
                let args = self.parse_argument_list()?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
                self.metadata.expr_spans.push(token.span);
                continue;
            }

            if let Some(token) = self.match_token(TokenKind::LBracket) {
                let index = self.parse_expression(0)?;
                self.expect(TokenKind::RBracket, "인덱싱은 `]`로 닫혀야 합니다.")?;
                expr = Expr::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                };
                self.metadata.expr_spans.push(token.span);
                continue;
            }

            if let Some(token) = self.match_token(TokenKind::Gen) {
                let name = self
                    .expect_ident_token("`의` 뒤에는 속성 이름이 필요합니다.")?
                    .lexeme;
                expr = Expr::Property {
                    base: Box::new(expr),
                    name,
                };
                self.metadata.expr_spans.push(token.span);
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self
            .advance()
            .ok_or_else(|| ParseError::new("표현식이 끝나지 않았습니다.", None))?;

        match token.kind {
            TokenKind::Ident => {
                self.metadata.name_expr_spans.push(token.span.clone());
                self.metadata.expr_spans.push(token.span.clone());
                Ok(Expr::Name(token.lexeme))
            }
            TokenKind::Int => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::Int(token.lexeme))
            }
            TokenKind::Float => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::Float(token.lexeme))
            }
            TokenKind::String => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::String(token.lexeme))
            }
            TokenKind::True => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::Bool(false))
            }
            TokenKind::None => {
                self.metadata.expr_spans.push(token.span);
                Ok(Expr::None)
            }
            TokenKind::LParen => {
                let expr = self.parse_expression(0)?;
                self.expect(TokenKind::RParen, "괄호 표현식은 `)`로 닫혀야 합니다.")?;
                Ok(expr)
            }
            TokenKind::LBracket => self.parse_list_literal(token.span),
            TokenKind::LBrace => self.parse_record_literal(token.span),
            _ => Err(ParseError::new(
                "표현식이 와야 할 위치입니다.",
                Some(token.span),
            )),
        }
    }

    fn parse_list_literal(&mut self, span: Span) -> Result<Expr, ParseError> {
        let mut items = Vec::new();

        if !self.at(TokenKind::RBracket) {
            loop {
                items.push(self.parse_expression(0)?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RBracket, "목록 리터럴은 `]`로 닫혀야 합니다.")?;
        self.metadata.expr_spans.push(span);
        Ok(Expr::List(items))
    }

    fn parse_record_literal(&mut self, span: Span) -> Result<Expr, ParseError> {
        let mut entries = Vec::new();

        if !self.at(TokenKind::RBrace) {
            loop {
                let key_token = self.advance().ok_or_else(|| {
                    ParseError::new("레코드 키가 와야 할 위치입니다.", self.current_span())
                })?;

                let key = match key_token.kind {
                    TokenKind::Ident | TokenKind::String => key_token.lexeme,
                    _ => {
                        return Err(ParseError::new(
                            "레코드 키는 식별자 또는 문자열이어야 합니다.",
                            Some(key_token.span),
                        ));
                    }
                };

                self.expect(TokenKind::Colon, "레코드 키 뒤에는 `:`가 와야 합니다.")?;
                let value = self.parse_expression(0)?;
                entries.push(RecordEntry { key, value });

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RBrace, "레코드 리터럴은 `}`로 닫혀야 합니다.")?;
        self.metadata.expr_spans.push(span);
        Ok(Expr::Record(entries))
    }

    fn parse_argument_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        if !self.at(TokenKind::RParen) {
            loop {
                args.push(self.parse_expression(0)?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen, "함수 호출은 `)`로 닫혀야 합니다.")?;
        Ok(args)
    }

    fn expect_ident_token(&mut self, message: &str) -> Result<Token, ParseError> {
        self.expect(TokenKind::Ident, message)
    }

    fn consume_optional_period(&mut self) {
        self.match_kind(TokenKind::Period);
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        let token = self
            .advance()
            .ok_or_else(|| ParseError::new(message, None))?;
        if token.kind == kind {
            Ok(token)
        } else {
            Err(ParseError::new(message, Some(token.span)))
        }
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8)> {
        let token = self.tokens.get(self.pos)?;
        match token.kind {
            TokenKind::Or => Some((BinaryOp::Or, 1)),
            TokenKind::And => Some((BinaryOp::And, 2)),
            TokenKind::Eq => Some((BinaryOp::Equal, 3)),
            TokenKind::Ne => Some((BinaryOp::NotEqual, 3)),
            TokenKind::Lt => Some((BinaryOp::Less, 4)),
            TokenKind::Le => Some((BinaryOp::LessEqual, 4)),
            TokenKind::Gt => Some((BinaryOp::Greater, 4)),
            TokenKind::Ge => Some((BinaryOp::GreaterEqual, 4)),
            TokenKind::Plus => Some((BinaryOp::Add, 5)),
            TokenKind::Minus => Some((BinaryOp::Subtract, 5)),
            TokenKind::Ident if token.lexeme == "더하기" => Some((BinaryOp::Add, 5)),
            TokenKind::Ident if token.lexeme == "빼기" => Some((BinaryOp::Subtract, 5)),
            TokenKind::Star => Some((BinaryOp::Multiply, 6)),
            TokenKind::Slash => Some((BinaryOp::Divide, 6)),
            TokenKind::Percent => Some((BinaryOp::Modulo, 6)),
            TokenKind::Ident if token.lexeme == "곱하기" => Some((BinaryOp::Multiply, 6)),
            TokenKind::Ident if token.lexeme == "나누기" => Some((BinaryOp::Divide, 6)),
            _ => None,
        }
    }

    fn is_function_def_start(&self) -> bool {
        self.nth_kind(0) == Some(TokenKind::Ident)
            && self.nth_kind(1) == Some(TokenKind::Function)
            && self.nth_kind(2) == Some(TokenKind::FunctionTopic)
    }

    fn is_bind_start(&self) -> bool {
        self.nth_kind(0) == Some(TokenKind::Ident) && self.nth_kind(1) == Some(TokenKind::Topic)
    }

    fn is_assign_start(&self) -> bool {
        self.nth_kind(0) == Some(TokenKind::Ident)
            && self.nth_kind(1) == Some(TokenKind::Object)
            && self
                .nth_kind(2)
                .is_some_and(|kind| kind != TokenKind::Print && kind != TokenKind::Return)
    }

    fn skip_newlines(&mut self) {
        while self.match_kind(TokenKind::Newline) {}
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        self.match_token(kind).is_some()
    }

    fn match_token(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) { self.advance() } else { None }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == Some(kind)
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|token| token.kind)
    }

    fn nth_kind(&self, offset: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| token.kind)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn current_span(&self) -> Option<Span> {
        self.tokens.get(self.pos).map(|token| token.span.clone())
    }

    fn error_here(&self, message: impl Into<String>) -> ParseError {
        ParseError::new(message, self.current_span())
    }
}
