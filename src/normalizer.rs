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
