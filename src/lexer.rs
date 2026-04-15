use crate::error::LexError;
use crate::normalizer::normalize_tokens;
use crate::token::{Span, Token, TokenKind};

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let tokens = Lexer::new(source).lex()?;
    Ok(normalize_tokens(tokens))
}

struct Lexer<'a> {
    lines: Vec<&'a str>,
    tokens: Vec<Token>,
    indent_stack: Vec<usize>,
    bracket_depth: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: split_lines(source),
            tokens: Vec::new(),
            indent_stack: vec![0],
            bracket_depth: 0,
        }
    }

    fn lex(mut self) -> Result<Vec<Token>, LexError> {
        for line_index in 0..self.lines.len() {
            let raw_line = self.lines[line_index];
            self.lex_line(raw_line, line_index + 1)?;
        }

        while self.indent_stack.len() > 1 {
            self.tokens.push(Token::new(
                TokenKind::Dedent,
                "",
                Span::new(self.lines.len().max(1), 1, self.lines.len().max(1), 1),
            ));
            self.indent_stack.pop();
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            "",
            Span::new(self.lines.len().max(1), 1, self.lines.len().max(1), 1),
        ));

        Ok(self.tokens)
    }

    fn lex_line(&mut self, raw_line: &str, line_no: usize) -> Result<(), LexError> {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let chars: Vec<char> = line.chars().collect();

        if self.bracket_depth == 0 {
            let indent = leading_spaces(&chars, line_no)?;
            let first_non_space = indent;

            if is_blank_or_comment_line(&chars[first_non_space..]) {
                return Ok(());
            }

            self.handle_indentation(indent, line_no)?;
            self.lex_content(&chars, first_non_space, line_no)?;
            self.tokens.push(Token::new(
                TokenKind::Newline,
                "",
                Span::new(line_no, chars.len() + 1, line_no, chars.len() + 1),
            ));
        } else {
            if is_blank_or_comment_line(&chars) {
                return Ok(());
            }
            self.lex_content(&chars, 0, line_no)?;
        }

        Ok(())
    }

    fn handle_indentation(&mut self, indent: usize, line_no: usize) -> Result<(), LexError> {
        let current = *self.indent_stack.last().unwrap_or(&0);
        if indent > current {
            self.indent_stack.push(indent);
            self.tokens.push(Token::new(
                TokenKind::Indent,
                "",
                Span::new(line_no, 1, line_no, indent + 1),
            ));
            return Ok(());
        }

        if indent < current {
            while indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
                self.tokens.push(Token::new(
                    TokenKind::Dedent,
                    "",
                    Span::new(line_no, 1, line_no, indent + 1),
                ));
            }

            if indent != *self.indent_stack.last().unwrap_or(&0) {
                return Err(LexError::InconsistentDedent {
                    line: line_no,
                    column: indent + 1,
                });
            }
        }

        Ok(())
    }

    fn lex_content(
        &mut self,
        chars: &[char],
        mut index: usize,
        line_no: usize,
    ) -> Result<(), LexError> {
        while index < chars.len() {
            let ch = chars[index];

            if ch == '#' {
                break;
            }

            if ch == '\t' {
                return Err(LexError::TabIndentation {
                    span: Span::new(line_no, index + 1, line_no, index + 2),
                });
            }

            if ch == ' ' {
                index += 1;
                continue;
            }

            match ch {
                '(' => {
                    self.push_simple(TokenKind::LParen, "(", line_no, index + 1, index + 2);
                    self.bracket_depth += 1;
                    index += 1;
                }
                ')' => {
                    self.push_simple(TokenKind::RParen, ")", line_no, index + 1, index + 2);
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    index += 1;
                    index = self.consume_attached_word(chars, index, line_no)?;
                }
                '[' => {
                    self.push_simple(TokenKind::LBracket, "[", line_no, index + 1, index + 2);
                    self.bracket_depth += 1;
                    index += 1;
                }
                ']' => {
                    self.push_simple(TokenKind::RBracket, "]", line_no, index + 1, index + 2);
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    index += 1;
                    index = self.consume_attached_word(chars, index, line_no)?;
                }
                '{' => {
                    self.push_simple(TokenKind::LBrace, "{", line_no, index + 1, index + 2);
                    self.bracket_depth += 1;
                    index += 1;
                }
                '}' => {
                    self.push_simple(TokenKind::RBrace, "}", line_no, index + 1, index + 2);
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    index += 1;
                    index = self.consume_attached_word(chars, index, line_no)?;
                }
                ',' => {
                    self.push_simple(TokenKind::Comma, ",", line_no, index + 1, index + 2);
                    index += 1;
                }
                ':' => {
                    self.push_simple(TokenKind::Colon, ":", line_no, index + 1, index + 2);
                    index += 1;
                }
                '.' => {
                    self.push_simple(TokenKind::Period, ".", line_no, index + 1, index + 2);
                    index += 1;
                }
                '+' => {
                    self.push_simple(TokenKind::Plus, "+", line_no, index + 1, index + 2);
                    index += 1;
                }
                '-' => {
                    self.push_simple(TokenKind::Minus, "-", line_no, index + 1, index + 2);
                    index += 1;
                }
                '*' => {
                    self.push_simple(TokenKind::Star, "*", line_no, index + 1, index + 2);
                    index += 1;
                }
                '/' => {
                    self.push_simple(TokenKind::Slash, "/", line_no, index + 1, index + 2);
                    index += 1;
                }
                '%' => {
                    self.push_simple(TokenKind::Percent, "%", line_no, index + 1, index + 2);
                    index += 1;
                }
                '=' => {
                    if peek_char(chars, index + 1) == Some('=') {
                        self.push_simple(TokenKind::Eq, "==", line_no, index + 1, index + 3);
                        index += 2;
                    } else {
                        return Err(LexError::UnexpectedCharacter {
                            ch,
                            span: Span::new(line_no, index + 1, line_no, index + 2),
                        });
                    }
                }
                '!' => {
                    if peek_char(chars, index + 1) == Some('=') {
                        self.push_simple(TokenKind::Ne, "!=", line_no, index + 1, index + 3);
                        index += 2;
                    } else {
                        return Err(LexError::UnexpectedCharacter {
                            ch,
                            span: Span::new(line_no, index + 1, line_no, index + 2),
                        });
                    }
                }
                '<' => {
                    if peek_char(chars, index + 1) == Some('=') {
                        self.push_simple(TokenKind::Le, "<=", line_no, index + 1, index + 3);
                        index += 2;
                    } else {
                        self.push_simple(TokenKind::Lt, "<", line_no, index + 1, index + 2);
                        index += 1;
                    }
                }
                '>' => {
                    if peek_char(chars, index + 1) == Some('=') {
                        self.push_simple(TokenKind::Ge, ">=", line_no, index + 1, index + 3);
                        index += 2;
                    } else {
                        self.push_simple(TokenKind::Gt, ">", line_no, index + 1, index + 2);
                        index += 1;
                    }
                }
                '"' => {
                    index = self.lex_string(chars, index, line_no)?;
                    index = self.consume_attached_word(chars, index, line_no)?;
                }
                c if c.is_ascii_digit() => {
                    index = self.lex_number(chars, index, line_no)?;
                    index = self.consume_attached_word(chars, index, line_no)?;
                }
                c if is_identifier_start(c) => {
                    index = self.lex_word(chars, index, line_no)?;
                }
                _ => {
                    return Err(LexError::UnexpectedCharacter {
                        ch,
                        span: Span::new(line_no, index + 1, line_no, index + 2),
                    });
                }
            }
        }

        Ok(())
    }

    fn lex_string(
        &mut self,
        chars: &[char],
        start: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        let mut end = start + 1;
        while end < chars.len() && chars[end] != '"' {
            if chars[end] == '\t' {
                return Err(LexError::TabIndentation {
                    span: Span::new(line_no, end + 1, line_no, end + 2),
                });
            }
            end += 1;
        }

        if end >= chars.len() {
            return Err(LexError::UnterminatedString {
                span: Span::new(line_no, start + 1, line_no, start + 2),
            });
        }

        let value: String = chars[start + 1..end].iter().collect();
        self.tokens.push(Token::new(
            TokenKind::String,
            value,
            Span::new(line_no, start + 1, line_no, end + 2),
        ));

        Ok(end + 1)
    }

    fn lex_number(
        &mut self,
        chars: &[char],
        start: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        let mut end = start;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }

        let kind = if peek_char(chars, end) == Some('.')
            && peek_char(chars, end + 1).is_some_and(|c| c.is_ascii_digit())
        {
            end += 1;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            TokenKind::Float
        } else {
            TokenKind::Int
        };

        let lexeme: String = chars[start..end].iter().collect();
        self.tokens.push(Token::new(
            kind,
            lexeme,
            Span::new(line_no, start + 1, line_no, end + 1),
        ));

        Ok(end)
    }

    fn lex_word(
        &mut self,
        chars: &[char],
        start: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        let mut end = start + 1;
        while end < chars.len() && is_identifier_continue(chars[end]) {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();

        if word == "함수는" {
            self.push_simple(TokenKind::Function, "함수", line_no, start + 1, start + 3);
            self.push_simple(
                TokenKind::FunctionTopic,
                "는",
                line_no,
                start + 3,
                start + 4,
            );
            return Ok(end);
        }

        if let Some(kind) = exact_word_kind(&word) {
            self.tokens.push(Token::new(
                kind,
                word,
                Span::new(line_no, start + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        if let Some((base, suffix_kind, suffix)) = split_attached_word(&word) {
            let base_len = base.chars().count();
            let base_kind = exact_word_kind(&base).unwrap_or(TokenKind::Ident);
            self.tokens.push(Token::new(
                base_kind,
                base,
                Span::new(line_no, start + 1, line_no, start + base_len + 1),
            ));
            self.tokens.push(Token::new(
                suffix_kind,
                suffix.clone(),
                Span::new(line_no, start + base_len + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        self.tokens.push(Token::new(
            TokenKind::Ident,
            word,
            Span::new(line_no, start + 1, line_no, end + 1),
        ));

        Ok(end)
    }

    fn consume_attached_word(
        &mut self,
        chars: &[char],
        index: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        if !peek_char(chars, index).is_some_and(is_identifier_start) {
            return Ok(index);
        }

        let mut end = index + 1;
        while end < chars.len() && is_identifier_continue(chars[end]) {
            end += 1;
        }

        let word: String = chars[index..end].iter().collect();
        if let Some(kind) = exact_word_kind(&word) {
            self.tokens.push(Token::new(
                kind,
                word,
                Span::new(line_no, index + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        Ok(index)
    }

    fn push_simple(
        &mut self,
        kind: TokenKind,
        lexeme: &str,
        line_no: usize,
        start_column: usize,
        end_column: usize,
    ) {
        self.tokens.push(Token::new(
            kind,
            lexeme,
            Span::new(line_no, start_column, line_no, end_column),
        ));
    }
}

fn split_lines(source: &str) -> Vec<&str> {
    if source.is_empty() {
        return vec![""];
    }

    let mut lines = Vec::new();
    let mut start = 0;
    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            lines.push(&source[start..index]);
            start = index + 1;
        }
    }

    if start <= source.len() {
        lines.push(&source[start..]);
    }

    lines
}

fn leading_spaces(chars: &[char], line_no: usize) -> Result<usize, LexError> {
    let mut count = 0;
    while count < chars.len() {
        match chars[count] {
            ' ' => count += 1,
            '\t' => {
                return Err(LexError::TabIndentation {
                    span: Span::new(line_no, count + 1, line_no, count + 2),
                });
            }
            _ => break,
        }
    }
    Ok(count)
}

fn is_blank_or_comment_line(chars: &[char]) -> bool {
    let mut index = 0;
    while index < chars.len() && chars[index] == ' ' {
        index += 1;
    }

    index >= chars.len() || chars[index] == '#'
}

fn peek_char(chars: &[char], index: usize) -> Option<char> {
    chars.get(index).copied()
}

fn exact_word_kind(word: &str) -> Option<TokenKind> {
    match word {
        "함수" => Some(TokenKind::Function),
        "이다" => Some(TokenKind::Copula),
        "것이다" => Some(TokenKind::ResultMarker),
        "이면" => Some(TokenKind::If),
        "아니면" => Some(TokenKind::Else),
        "인" => Some(TokenKind::In),
        "동안" => Some(TokenKind::During),
        "받아" => Some(TokenKind::Receive),
        "아무것도" => Some(TokenKind::Nothing),
        "받지" => Some(TokenKind::ReceiveNot),
        "않아" => Some(TokenKind::ReceiveNeg),
        "출력한다" | "출력하고" => Some(TokenKind::Print),
        "돌려준다" | "돌려주고" => Some(TokenKind::Return),
        "바꾼다" | "바꾸고" => Some(TokenKind::Change),
        "참" => Some(TokenKind::True),
        "거짓" => Some(TokenKind::False),
        "없음" => Some(TokenKind::None),
        "그리고" => Some(TokenKind::And),
        "또는" => Some(TokenKind::Or),
        "아니다" => Some(TokenKind::Not),
        "은" | "는" => Some(TokenKind::Topic),
        "을" | "를" => Some(TokenKind::Object),
        "의" => Some(TokenKind::Gen),
        "에" => Some(TokenKind::Locative),
        "에서" => Some(TokenKind::From),
        "로" | "으로" => Some(TokenKind::Direction),
        "보다" => Some(TokenKind::Than),
        "과" | "와" | "이랑" | "랑" => Some(TokenKind::With),
        "만큼" => Some(TokenKind::Amount),
        _ => None,
    }
}

fn split_attached_word(word: &str) -> Option<(String, TokenKind, String)> {
    // "인"은 여기서 제거 — normalizer가 "X인 동안" 문맥에서만 분리 (P-3)
    const KEYWORD_SUFFIXES: [(&str, TokenKind); 2] = [
        ("이면", TokenKind::If),
        ("이다", TokenKind::Copula),
    ];
    // "이"/"가" 주격 조사는 여기서 제거 — normalizer가 문맥 기반으로 처리 (P-2)
    const PARTICLES: [(&str, TokenKind); 15] = [
        ("만큼", TokenKind::Amount),
        ("이랑", TokenKind::With),
        ("보다", TokenKind::Than),
        ("으로", TokenKind::Direction),
        ("에서", TokenKind::From),
        ("은", TokenKind::Topic),
        ("는", TokenKind::Topic),
        ("을", TokenKind::Object),
        ("를", TokenKind::Object),
        ("과", TokenKind::With),
        ("와", TokenKind::With),
        ("랑", TokenKind::With),
        ("의", TokenKind::Gen),
        ("에", TokenKind::Locative),
        ("로", TokenKind::Direction),
    ];

    for (suffix, kind) in KEYWORD_SUFFIXES {
        if let Some(base) = word.strip_suffix(suffix) {
            if base.is_empty() || !should_split_word(base, suffix) {
                continue;
            }
            return Some((base.to_string(), kind, suffix.to_string()));
        }
    }

    for (suffix, kind) in PARTICLES {
        if let Some(base) = word.strip_suffix(suffix) {
            if base.is_empty() || !should_split_word(base, suffix) {
                continue;
            }
            return Some((base.to_string(), kind, suffix.to_string()));
        }
    }

    None
}

// Korean identifiers that end with particle-like syllables are ambiguous.
// This lexer stays conservative for short Hangul bases so words like `나이`
// are not incorrectly normalized into `나` + `이`. Parser-side helpers handle
// a few high-value attached forms such as one-syllable print/assign targets.
fn should_split_word(base: &str, suffix: &str) -> bool {
    // "이다"/"이면"은 키워드 접미사이므로 base 길이와 무관하게 항상 분리한다.
    // "인"은 normalizer가 "X인 동안" 문맥에서만 분리하므로 여기선 제외.
    if matches!(suffix, "으로" | "은" | "는" | "보다" | "만큼" | "이면" | "이다") {
        return true;
    }

    let base_len = base.chars().count();
    if base_len >= 2 {
        return true;
    }

    if exact_word_kind(base).is_some() {
        return true;
    }

    base.chars().any(|ch| !is_hangul_syllable(ch))
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || is_hangul_syllable(ch)
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

fn is_hangul_syllable(ch: char) -> bool {
    matches!(ch as u32, 0xAC00..=0xD7A3)
}
