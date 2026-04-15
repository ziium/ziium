use crate::token::{Span, Token, TokenKind};

pub fn normalize_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut normalized = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        if let Some((base, suffix_kind, suffix)) = split_attached_statement_head(&tokens, index) {
            let token = &tokens[index];
            let base_len = base.chars().count();

            normalized.push(Token::new(
                TokenKind::Ident,
                base,
                Span::new(
                    token.span.start_line,
                    token.span.start_column,
                    token.span.start_line,
                    token.span.start_column + base_len,
                ),
            ));
            normalized.push(Token::new(
                suffix_kind,
                suffix,
                Span::new(
                    token.span.start_line,
                    token.span.start_column + base_len,
                    token.span.end_line,
                    token.span.end_column,
                ),
            ));
            index += 1;
            continue;
        }

        // P-1: Ident("나를") + 받아/돌려준다/출력한다 → 을/를 분리
        if let Some((base, suffix_kind, suffix)) =
            split_object_before_keyword(&tokens, index)
        {
            let token = &tokens[index];
            let base_len = base.chars().count();
            normalized.push(Token::new(
                TokenKind::Ident,
                base,
                Span::new(
                    token.span.start_line,
                    token.span.start_column,
                    token.span.start_line,
                    token.span.start_column + base_len,
                ),
            ));
            normalized.push(Token::new(
                suffix_kind,
                suffix.to_string(),
                Span::new(
                    token.span.start_line,
                    token.span.start_column + base_len,
                    token.span.end_line,
                    token.span.end_column,
                ),
            ));
            index += 1;
            continue;
        }

        // P-3: Ident("X인") + During("동안") → Ident("X") + In("인") 분리
        if let Some((base, base_len)) = split_in_before_during(&tokens, index) {
            let token = &tokens[index];
            normalized.push(Token::new(
                TokenKind::Ident,
                base,
                Span::new(
                    token.span.start_line,
                    token.span.start_column,
                    token.span.start_line,
                    token.span.start_column + base_len,
                ),
            ));
            normalized.push(Token::new(
                TokenKind::In,
                "인",
                Span::new(
                    token.span.start_line,
                    token.span.start_column + base_len,
                    token.span.end_line,
                    token.span.end_column,
                ),
            ));
            index += 1;
            continue;
        }

        // `Ident("X에")` + `Topic("는")` + ... + `Exist("있다")` 또는
        // `Ident("X에")` + `Topic("는")` + ... + `Store("넣는다")` →
        // `Ident("X") + Locative("에")` 분리
        if let Some((base, base_len)) = split_locative_before_topic(&tokens, index) {
            let token = &tokens[index];
            normalized.push(Token::new(
                TokenKind::Ident,
                base,
                Span::new(
                    token.span.start_line,
                    token.span.start_column,
                    token.span.start_line,
                    token.span.start_column + base_len,
                ),
            ));
            normalized.push(Token::new(
                TokenKind::Locative,
                "에",
                Span::new(
                    token.span.start_line,
                    token.span.start_column + base_len,
                    token.span.end_line,
                    token.span.end_column,
                ),
            ));
            index += 1;
            continue;
        }

        // P-1: 독립 Ident("가"/"이")가 비교 꼬리 앞에 오면 Subject로 재분류
        if is_standalone_subject_particle(&tokens, index) {
            let token = &tokens[index];
            normalized.push(Token::new(
                TokenKind::Subject,
                token.lexeme.clone(),
                token.span.clone(),
            ));
            index += 1;
            continue;
        }

        normalized.push(tokens[index].clone());
        index += 1;
    }

    normalized
}

fn split_attached_statement_head(
    tokens: &[Token],
    index: usize,
) -> Option<(String, TokenKind, &'static str)> {
    let token = tokens.get(index)?;
    if token.kind != TokenKind::Ident || !is_statement_start(tokens, index) {
        return None;
    }

    let (base, suffix_kind, suffix) = split_attached_statement_suffix(&token.lexeme)?;

    match suffix_kind {
        TokenKind::Object => match tokens.get(index + 1).map(|token| token.kind) {
            Some(TokenKind::Print) | Some(TokenKind::Return) => Some((base, suffix_kind, suffix)),
            _ if line_has_assignment_tail(tokens, index + 1) => Some((base, suffix_kind, suffix)),
            _ => None,
        },
        TokenKind::Subject if line_has_comparison_tail(tokens, index + 1) => {
            Some((base, suffix_kind, suffix))
        }
        TokenKind::Locative if line_has_keyword_message_tail(tokens, index + 1) => {
            Some((base, suffix_kind, suffix))
        }
        _ => None,
    }
}

fn split_attached_statement_suffix(lexeme: &str) -> Option<(String, TokenKind, &'static str)> {
    for (suffix, kind) in [
        ("을", TokenKind::Object),
        ("를", TokenKind::Object),
        ("이", TokenKind::Subject),
        ("가", TokenKind::Subject),
        ("에", TokenKind::Locative),
    ] {
        if let Some(base) = lexeme.strip_suffix(suffix) {
            if !base.is_empty() {
                return Some((base.to_string(), kind, suffix));
            }
        }
    }

    None
}

fn is_statement_start(tokens: &[Token], index: usize) -> bool {
    if index == 0 {
        return true;
    }

    matches!(
        tokens.get(index - 1).map(|token| token.kind),
        Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent)
    )
}

fn line_has_assignment_tail(tokens: &[Token], mut index: usize) -> bool {
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => return false,
            TokenKind::Direction => {
                return tokens
                    .get(index + 1)
                    .is_some_and(|next| next.kind == TokenKind::Change);
            }
            TokenKind::Amount => {
                return tokens.get(index + 1).is_some_and(|next| {
                    next.kind == TokenKind::Ident
                        && matches!(next.lexeme.as_str(), "줄인다" | "늘린다")
                });
            }
            _ => index += 1,
        }
    }

    false
}

fn line_has_comparison_tail(tokens: &[Token], mut index: usize) -> bool {
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => return false,
            TokenKind::Than | TokenKind::With => {
                return tokens.get(index + 1).is_some_and(|next| {
                    next.kind == TokenKind::Ident
                        && matches!(
                            next.lexeme.as_str(),
                            "크면" | "작으면" | "같으면" | "다르면"
                        )
                });
            }
            _ => index += 1,
        }
    }

    false
}

fn split_object_before_keyword(
    tokens: &[Token],
    index: usize,
) -> Option<(String, TokenKind, &'static str)> {
    let token = tokens.get(index)?;
    if token.kind != TokenKind::Ident {
        return None;
    }
    // 다음 토큰이 Receive/Return/Print인 경우에만
    let next_kind = tokens.get(index + 1).map(|t| t.kind);
    if !matches!(
        next_kind,
        Some(TokenKind::Receive | TokenKind::Return | TokenKind::Print)
    ) {
        return None;
    }
    // 을/를 접미사 분리 — 단음절 base만 대상 (P-1).
    // 다음절 base(예: "마을")는 렉서의 split_attached_word가 이미 처리했거나,
    // should_split_word 가드에 의해 의도적으로 보존된 것이므로 건드리지 않는다.
    for (suffix, kind) in [("을", TokenKind::Object), ("를", TokenKind::Object)] {
        if let Some(base) = token.lexeme.strip_suffix(suffix) {
            if !base.is_empty() && base.chars().count() == 1 {
                return Some((base.to_string(), kind, suffix));
            }
        }
    }
    None
}

fn is_standalone_subject_particle(tokens: &[Token], index: usize) -> bool {
    let _token = match tokens.get(index) {
        Some(t) if t.kind == TokenKind::Ident && matches!(t.lexeme.as_str(), "가" | "이") => t,
        _ => return false,
    };

    // 앞 토큰이 Ident가 아닌 경우에만 (Ident 뒤면 이미 split_attached_word가 처리)
    // 또는 문장 시작 위치가 아닌 경우 (문장 시작 Ident는 split_attached_statement_head가 처리)
    let prev = tokens.get(index.wrapping_sub(1)).map(|t| t.kind);
    let after_non_ident = matches!(
        prev,
        Some(
            TokenKind::Int
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::True
                | TokenKind::False
                | TokenKind::RParen
                | TokenKind::RBracket
        )
    );

    if !after_non_ident {
        return false;
    }

    // "가"/"이" 뒤에 비교 꼬리가 있는지, 또는 `있다`가 있는지
    line_has_comparison_tail(tokens, index + 1)
        || tokens
            .get(index + 1)
            .is_some_and(|t| t.kind == TokenKind::Exist)
}

/// P-3: `Ident("X인")` 바로 뒤에 `During("동안")`이 오면 "인"을 분리한다.
/// "확인", "회문확인" 등 "인"으로 끝나는 일반 식별자는 건드리지 않는다.
fn split_in_before_during(tokens: &[Token], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind != TokenKind::Ident {
        return None;
    }
    let base = token.lexeme.strip_suffix("인")?;
    if base.is_empty() {
        return None;
    }
    // 다음 토큰이 During("동안")인 경우에만 분리
    if !tokens
        .get(index + 1)
        .is_some_and(|t| t.kind == TokenKind::During)
    {
        return None;
    }
    let base_len = base.chars().count();
    Some((base.to_string(), base_len))
}

/// `Ident("X에")` + `Topic("는"/"은")` → `Ident("X") + Locative("에")` 분리.
/// 라인에 `Exist("있다")` 또는 `Store("넣는다"/"넣고")` 꼬리가 있는 경우에만 분리한다.
fn split_locative_before_topic(tokens: &[Token], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind != TokenKind::Ident {
        return None;
    }
    let base = token.lexeme.strip_suffix("에")?;
    if base.is_empty() {
        return None;
    }
    if !tokens
        .get(index + 1)
        .is_some_and(|t| t.kind == TokenKind::Topic)
    {
        return None;
    }
    if !line_has_bind_tail(tokens, index + 2) {
        return None;
    }
    let base_len = base.chars().count();
    Some((base.to_string(), base_len))
}

fn line_has_bind_tail(tokens: &[Token], mut index: usize) -> bool {
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => return false,
            TokenKind::Exist | TokenKind::Store => return true,
            _ => index += 1,
        }
    }
    false
}

fn line_has_keyword_message_tail(tokens: &[Token], mut index: usize) -> bool {
    let mut last_ident = None;

    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
            TokenKind::Period => {}
            TokenKind::Ident => last_ident = Some(token.lexeme.as_str()),
            _ => {}
        }
        index += 1;
    }

    matches!(last_ident, Some("추가"))
}
