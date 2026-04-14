use crate::ast::{BinaryOp, BinarySurface, Expr, Program, RecordEntry, Stmt, UnaryOp};
use crate::error::{FrontendError, ParseError};
use crate::lexer::lex;
use crate::message::{KeywordMessage, keyword_message_for_selector};
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

            if self.at_ident_lexeme("초") && self.nth_ident_lexeme(1, "쉬기") {
                self.parse_sleep_statement(expr)?
            } else if self.match_kind(TokenKind::From) {
                self.parse_resultive_statement(expr)?
            } else if self.match_kind(TokenKind::If) {
                self.parse_if_tail(expr, "`이면` 뒤에는 들여쓴 블록이 와야 합니다.")?
            } else if self.at(TokenKind::Subject) && self.is_korean_comparison() {
                self.parse_korean_comparison(expr)?
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
                    self.parse_named_call_statement(expr)?
                }
            } else if matches!(expr, Expr::Call { .. } | Expr::TransformCall { .. }) {
                let stmt = Stmt::Expr(expr);
                self.consume_optional_period();
                stmt
            } else {
                return Err(self.error_here("문장을 해석할 수 없습니다. 바인딩, 출력, 쉬기, 결과 서술, 조건문, 반복문, 함수 정의, 키워드 메시지, 호출문 중 하나를 기대했습니다."));
            }
        };

        if let Some(span) = statement_span {
            self.metadata.statement_spans.push(span);
        }

        Ok(stmt)
    }

    fn parse_named_call_statement(&mut self, callee: Expr) -> Result<Stmt, ParseError> {
        let named_args = self.parse_expression_without_transform(0)?;
        self.expect(
            TokenKind::Direction,
            "넘긴 값 뒤에는 `로` 또는 `으로`가 와야 합니다.",
        )?;
        self.expect_ident_lexeme(
            "호출한다",
            "`로` 또는 `으로` 뒤에는 `호출한다`가 와야 합니다.",
        )?;
        if !matches!(named_args, Expr::Record(_)) {
            return Err(self.error_here("이름 붙은 호출에 넘긴 값은 레코드여야 합니다."));
        }
        self.consume_optional_period();
        Ok(Stmt::NamedCall { callee, named_args })
    }

    fn parse_keyword_message(&mut self, receiver: Expr) -> Result<Stmt, ParseError> {
        let arg = self.parse_expression_without_transform(0)?;
        let has_direction = self.match_kind(TokenKind::Direction);
        let selector = self.expect_ident_token("`에` 뒤 키워드 메시지 이름이 필요합니다.")?;
        let keyword = keyword_message_for_selector(&selector.lexeme).ok_or_else(|| {
            ParseError::new(
                "현재 키워드 메시지는 `추가`, `지우기`, `점찍기`, `사각형채우기`, `글자쓰기`만 지원합니다.",
                Some(selector.span.clone()),
            )
        })?;

        match keyword {
            KeywordMessage::Push => {
                if has_direction {
                    return Err(ParseError::new(
                        "`추가`는 `<목록>에 <값> 추가` 형식으로만 쓸 수 있습니다.",
                        Some(selector.span),
                    ));
                }
            }
            KeywordMessage::CanvasClear
            | KeywordMessage::CanvasFillRect
            | KeywordMessage::CanvasFillText
            | KeywordMessage::CanvasDot => {
                if !has_direction {
                    return Err(ParseError::new(
                        "`그림판` 동작은 `그림판에 <레코드>로/으로 <동작>` 형식으로만 쓸 수 있습니다.",
                        Some(selector.span.clone()),
                    ));
                }
                if !matches!(&receiver, Expr::Name(name) if name == "그림판") {
                    return Err(ParseError::new(
                        "현재 `지우기`, `점찍기`, `사각형채우기`, `글자쓰기`는 `그림판`에만 사용할 수 있습니다.",
                        Some(selector.span.clone()),
                    ));
                }
                if !matches!(&arg, Expr::Record(_)) {
                    return Err(ParseError::new(
                        "`그림판` 동작에 넘긴 값은 레코드여야 합니다.",
                        Some(selector.span.clone()),
                    ));
                }
            }
        }

        self.consume_optional_period();
        Ok(Stmt::KeywordMessage {
            receiver,
            selector: selector.lexeme,
            arg,
        })
    }

    fn parse_sleep_statement(&mut self, duration_seconds: Expr) -> Result<Stmt, ParseError> {
        self.expect_ident_lexeme("초", "시간 값 뒤에는 `초`가 와야 합니다.")?;
        self.expect_ident_lexeme("쉬기", "`초` 뒤에는 `쉬기`가 와야 합니다.")?;
        self.consume_optional_period();
        Ok(Stmt::Sleep { duration_seconds })
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
        let receiver = self.parse_expression(0)?;
        let value = if self.match_kind(TokenKind::From) {
            self.parse_resultive_expression(receiver)?
        } else {
            if self.at(TokenKind::Direction) {
                return Err(self.error_here("`로` 또는 `으로` 뒤에는 함수 이름이 필요합니다."));
            }
            self.expect(TokenKind::Copula, "바인딩 문장은 `이다`로 끝나야 합니다.")?;
            receiver
        };
        self.consume_optional_period();
        Ok(Stmt::Bind { name, value })
    }

    fn parse_resultive_expression(&mut self, receiver: Expr) -> Result<Expr, ParseError> {
        // 선택 프레임: `에서 고른 것이다` (role 없음)
        if self.at_ident_lexeme("고른") {
            let verb = self.advance().unwrap();
            let result_marker = self.expect(
                TokenKind::ResultMarker,
                "`고른` 뒤에는 `것이다`가 와야 합니다.",
            )?;
            let expr = Expr::Resultive {
                receiver: Box::new(receiver),
                role: String::new(),
                verb: verb.lexeme,
            };
            self.metadata.expr_spans.push(result_marker.span);
            return Ok(expr);
        }

        let role = self.parse_resultive_role(
            "`에서` 뒤에는 현재 `맨위 요소를`, `맨뒤 요소를`, `맨앞 요소를 꺼낸 것이다` 또는 `고른 것이다`만 지원합니다.",
        )?;
        let object_label = format!("{}를", role);
        let verb_error = format!("`{}` 뒤에는 현재 `꺼낸`만 지원합니다.", object_label);
        let verb = self.expect_ident_lexeme("꺼낸", &verb_error)?;
        let result_marker = self.expect(
            TokenKind::ResultMarker,
            "`꺼낸` 뒤에는 `것이다`가 와야 합니다.",
        )?;

        let expr = Expr::Resultive {
            receiver: Box::new(receiver),
            role,
            verb: verb.lexeme,
        };
        self.metadata.expr_spans.push(result_marker.span);
        Ok(expr)
    }

    fn parse_resultive_statement(&mut self, receiver: Expr) -> Result<Stmt, ParseError> {
        // 선택 프레임 (문장형): `에서 고른다`
        if self.at_ident_lexeme("고른다") {
            let verb = self.advance().unwrap();
            self.consume_optional_period();
            return Ok(Stmt::Resultive {
                receiver,
                role: String::new(),
                verb: verb.lexeme,
            });
        }

        let role = self.parse_resultive_role(
            "`에서` 뒤에는 현재 `맨위 요소를`, `맨뒤 요소를`, `맨앞 요소를 꺼낸다` 또는 `고른다`만 지원합니다.",
        )?;
        let object_label = format!("{}를", role);
        let verb_error = format!("`{}` 뒤에는 현재 `꺼낸다`만 지원합니다.", object_label);
        let verb = self.expect_ident_lexeme("꺼낸다", &verb_error)?;
        self.consume_optional_period();
        Ok(Stmt::Resultive {
            receiver,
            role,
            verb: verb.lexeme,
        })
    }

    fn parse_resultive_role(&mut self, from_error: &str) -> Result<String, ParseError> {
        let position = self.expect_ident_token(from_error)?;
        if !matches!(position.lexeme.as_str(), "맨위" | "맨뒤" | "맨앞") {
            return Err(ParseError::new(from_error, Some(position.span)));
        }
        let noun_error = format!("`{}` 뒤에는 현재 `요소`가 와야 합니다.", position.lexeme);
        let role_noun = self.expect_ident_lexeme("요소", &noun_error)?;
        let role = format!("{} {}", position.lexeme, role_noun.lexeme);
        let object_error = format!("`{}` 뒤에는 `을` 또는 `를`이 와야 합니다.", role);
        self.expect(TokenKind::Object, &object_error)?;
        Ok(role)
    }

    fn parse_assign(&mut self) -> Result<Stmt, ParseError> {
        let name_token = self.expect_ident_token("재대입 대상 이름이 필요합니다.")?;
        self.metadata
            .assign_target_spans
            .push(name_token.span.clone());
        let name = name_token.lexeme.clone();
        self.expect(
            TokenKind::Object,
            "재대입 대상 뒤에는 `을` 또는 `를`이 와야 합니다.",
        )?;
        let amount_expr = self.parse_expression(0)?;

        if self.match_kind(TokenKind::Amount) {
            // 상대적 변화: `체력을 10만큼 줄인다/늘린다`
            let verb = self.advance().ok_or_else(|| {
                self.error_here("`만큼` 뒤에는 `줄인다` 또는 `늘린다`가 와야 합니다.")
            })?;
            let op = match verb.lexeme.as_str() {
                "줄인다" => BinaryOp::Subtract,
                "늘린다" => BinaryOp::Add,
                _ => {
                    return Err(ParseError::new(
                        "`만큼` 뒤에는 `줄인다` 또는 `늘린다`가 와야 합니다.",
                        Some(verb.span),
                    ));
                }
            };
            self.metadata.name_expr_spans.push(verb.span.clone());
            self.metadata.expr_spans.push(verb.span.clone());
            let value = Expr::Binary {
                left: Box::new(Expr::Name(name.clone())),
                op,
                right: Box::new(amount_expr),
                form: BinarySurface::Symbol,
            };
            self.metadata.expr_spans.push(verb.span);
            self.consume_optional_period();
            Ok(Stmt::Assign { name, value })
        } else {
            // 기존 재대입: `체력을 20으로 바꾼다`
            self.expect(
                TokenKind::Direction,
                "재대입 값 뒤에는 `로` 또는 `으로`가 와야 합니다.",
            )?;
            self.expect(TokenKind::Change, "재대입 문장은 `바꾼다`로 끝나야 합니다.")?;
            self.consume_optional_period();
            Ok(Stmt::Assign {
                name,
                value: amount_expr,
            })
        }
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
        self.parse_expression_with_options(min_precedence, true)
    }

    fn parse_expression_without_transform(
        &mut self,
        min_precedence: u8,
    ) -> Result<Expr, ParseError> {
        self.parse_expression_with_options(min_precedence, false)
    }

    fn parse_expression_with_options(
        &mut self,
        min_precedence: u8,
        allow_transform_call: bool,
    ) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary(allow_transform_call)?;

        loop {
            let Some((op, precedence, form)) = self.current_binary_op() else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            let operator_span = self
                .advance()
                .expect("current_binary_op should only be present when a token exists")
                .span;
            let right = self.parse_expression_with_options(precedence + 1, allow_transform_call)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                form,
            };
            self.metadata.expr_spans.push(operator_span);
        }

        Ok(left)
    }

    fn parse_unary(&mut self, allow_transform_call: bool) -> Result<Expr, ParseError> {
        if let Some(token) = self.match_token(TokenKind::Minus) {
            let expr = self.parse_unary(allow_transform_call)?;
            let expr = Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
            };
            self.metadata.expr_spans.push(token.span);
            return Ok(expr);
        }

        if let Some(token) = self.match_token(TokenKind::Not) {
            let expr = self.parse_unary(allow_transform_call)?;
            let expr = Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            };
            self.metadata.expr_spans.push(token.span);
            return Ok(expr);
        }

        self.parse_postfix(allow_transform_call)
    }

    fn parse_postfix(&mut self, allow_transform_call: bool) -> Result<Expr, ParseError> {
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

            if allow_transform_call
                && self.at(TokenKind::Direction)
                && self.nth_kind(1) == Some(TokenKind::Ident)
            {
                let token = self
                    .advance()
                    .expect("direction token should exist when parsing transform call");
                let callee =
                    self.expect_ident_token("`로` 또는 `으로` 뒤에는 함수 이름이 필요합니다.")?;
                self.metadata.name_expr_spans.push(callee.span.clone());
                expr = Expr::TransformCall {
                    input: Box::new(expr),
                    callee: callee.lexeme,
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

                let value = if self.match_kind(TokenKind::Colon) {
                    self.parse_expression(0)?
                } else if key_token.kind == TokenKind::Ident {
                    self.metadata.name_expr_spans.push(key_token.span.clone());
                    self.metadata.expr_spans.push(key_token.span.clone());
                    Expr::Name(key.clone())
                } else {
                    return Err(ParseError::new(
                        "문자열 레코드 키 뒤에는 `:`가 와야 합니다.",
                        Some(key_token.span),
                    ));
                };
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

    fn expect_ident_lexeme(&mut self, lexeme: &str, message: &str) -> Result<Token, ParseError> {
        let token = self.expect(TokenKind::Ident, message)?;
        if token.lexeme == lexeme {
            Ok(token)
        } else {
            Err(ParseError::new(message, Some(token.span)))
        }
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

    fn current_binary_op(&self) -> Option<(BinaryOp, u8, BinarySurface)> {
        let token = self.tokens.get(self.pos)?;
        match token.kind {
            TokenKind::Or => Some((BinaryOp::Or, 1, BinarySurface::Symbol)),
            TokenKind::And => Some((BinaryOp::And, 2, BinarySurface::Symbol)),
            TokenKind::Eq => Some((BinaryOp::Equal, 3, BinarySurface::Symbol)),
            TokenKind::Ne => Some((BinaryOp::NotEqual, 3, BinarySurface::Symbol)),
            TokenKind::Lt => Some((BinaryOp::Less, 4, BinarySurface::Symbol)),
            TokenKind::Le => Some((BinaryOp::LessEqual, 4, BinarySurface::Symbol)),
            TokenKind::Gt => Some((BinaryOp::Greater, 4, BinarySurface::Symbol)),
            TokenKind::Ge => Some((BinaryOp::GreaterEqual, 4, BinarySurface::Symbol)),
            TokenKind::Plus => Some((BinaryOp::Add, 5, BinarySurface::Symbol)),
            TokenKind::Minus => Some((BinaryOp::Subtract, 5, BinarySurface::Symbol)),
            TokenKind::Ident if token.lexeme == "더하기" => {
                Some((BinaryOp::Add, 5, BinarySurface::Word))
            }
            TokenKind::Ident if token.lexeme == "빼기" => {
                Some((BinaryOp::Subtract, 5, BinarySurface::Word))
            }
            TokenKind::Star => Some((BinaryOp::Multiply, 6, BinarySurface::Symbol)),
            TokenKind::Slash => Some((BinaryOp::Divide, 6, BinarySurface::Symbol)),
            TokenKind::Percent => Some((BinaryOp::Modulo, 6, BinarySurface::Symbol)),
            TokenKind::Ident if token.lexeme == "곱하기" => {
                Some((BinaryOp::Multiply, 6, BinarySurface::Word))
            }
            TokenKind::Ident if token.lexeme == "나누기" => {
                Some((BinaryOp::Divide, 6, BinarySurface::Word))
            }
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
            && !self.is_named_call_start()
            && self
                .nth_kind(2)
                .is_some_and(|kind| kind != TokenKind::Print && kind != TokenKind::Return)
    }

    fn is_named_call_start(&self) -> bool {
        let mut index = 2;
        while let Some(token) = self.tokens.get(self.pos + index) {
            match token.kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => return false,
                TokenKind::Direction => {
                    let Some(next) = self.tokens.get(self.pos + index + 1) else {
                        return false;
                    };
                    return next.kind == TokenKind::Ident && next.lexeme == "호출한다";
                }
                _ => index += 1,
            }
        }
        false
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

    fn at_ident_lexeme(&self, lexeme: &str) -> bool {
        self.tokens
            .get(self.pos)
            .is_some_and(|token| token.kind == TokenKind::Ident && token.lexeme == lexeme)
    }

    fn nth_ident_lexeme(&self, offset: usize, lexeme: &str) -> bool {
        self.tokens
            .get(self.pos + offset)
            .is_some_and(|token| token.kind == TokenKind::Ident && token.lexeme == lexeme)
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

    fn parse_if_tail(&mut self, condition: Expr, block_msg: &str) -> Result<Stmt, ParseError> {
        let then_block = self.parse_indented_block(block_msg)?;
        let else_block = if self.match_kind(TokenKind::Else) {
            Some(self.parse_indented_block("`아니면` 뒤에는 들여쓴 블록이 와야 합니다.")?)
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    /// Look ahead to check if this is a Korean comparison pattern:
    /// `<expr> 가/이 <tokens...> 보다 크면/작으면/같으면/다르면`
    /// `<expr> 가/이 <tokens...> 과/와/이랑/랑 같으면/다르면`
    fn is_korean_comparison(&self) -> bool {
        // Current position is at Subject (가/이).
        // Scan forward to find Than (보다) or With (과/와/이랑/랑) followed by a comparison word.
        let mut i = self.pos + 1; // skip Subject
        while let Some(token) = self.tokens.get(i) {
            match token.kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => return false,
                TokenKind::Than | TokenKind::With => {
                    // Check if next token is a comparison word
                    return self.tokens.get(i + 1).is_some_and(|next| {
                        next.kind == TokenKind::Ident
                            && matches!(
                                next.lexeme.as_str(),
                                "크면" | "작으면" | "같으면" | "다르면"
                            )
                    });
                }
                _ => i += 1,
            }
        }
        false
    }

    /// Parse Korean comparison conditional:
    /// `<left>가/이 <right>보다 크면/작으면/같으면/다르면`
    /// `<left>가/이 <right>과/와/이랑/랑 같으면/다르면`
    /// Produces Stmt::If with a Binary comparison condition.
    fn parse_korean_comparison(&mut self, left: Expr) -> Result<Stmt, ParseError> {
        self.advance(); // consume Subject (가/이)

        let right = self.parse_expression(0)?;

        let particle = self.advance().ok_or_else(|| {
            self.error_here("한국어 비교문에서 `보다` 또는 `과/와/이랑/랑`이 필요합니다.")
        })?;
        if !matches!(particle.kind, TokenKind::Than | TokenKind::With) {
            return Err(ParseError::new(
                "한국어 비교문에서 `보다` 또는 `과/와/이랑/랑`이 필요합니다.",
                Some(particle.span),
            ));
        }

        let cmp_token = self.advance().ok_or_else(|| {
            self.error_here("비교 서술어(`크면`, `작으면`, `같으면`, `다르면`)가 필요합니다.")
        })?;
        let cmp_span = cmp_token.span.clone();

        let op = match cmp_token.lexeme.as_str() {
            "크면" => BinaryOp::Greater,
            "작으면" => BinaryOp::Less,
            "같으면" => BinaryOp::Equal,
            "다르면" => BinaryOp::NotEqual,
            _ => {
                return Err(ParseError::new(
                    "비교 서술어는 `크면`, `작으면`, `같으면`, `다르면` 중 하나여야 합니다.",
                    Some(cmp_token.span),
                ));
            }
        };

        let condition = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
            form: BinarySurface::Symbol,
        };
        self.metadata.expr_spans.push(cmp_span);

        self.parse_if_tail(condition, "한국어 비교문 뒤에는 들여쓴 블록이 와야 합니다.")
    }
}
