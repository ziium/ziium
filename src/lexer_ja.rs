use crate::error::LexError;
use crate::token::{Span, Token, TokenKind};

pub fn lex_ja(source: &str) -> Result<Vec<Token>, LexError> {
    LexerJa::new(source).lex()
}

// ---------------------------------------------------------------------------
// Script classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptClass {
    Kanji,
    Hiragana,
    Katakana,
    Latin,
    Other,
}

fn script_class(ch: char) -> ScriptClass {
    match ch as u32 {
        0x3040..=0x309F => ScriptClass::Hiragana,
        0x30A0..=0x30FF => ScriptClass::Katakana,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF => ScriptClass::Kanji,
        _ if ch == '_' || ch.is_ascii_alphabetic() => ScriptClass::Latin,
        _ => ScriptClass::Other,
    }
}

fn is_japanese_char(ch: char) -> bool {
    matches!(
        script_class(ch),
        ScriptClass::Kanji | ScriptClass::Hiragana | ScriptClass::Katakana
    )
}

fn is_ja_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || is_japanese_char(ch)
}

// ---------------------------------------------------------------------------
// Keyword and particle tables (sorted by length descending for longest match)
// ---------------------------------------------------------------------------

const KEYWORDS: &[(&str, TokenKind)] = &[
    ("でなければ", TokenKind::Else),
    ("ではない", TokenKind::Not),
    ("出力する", TokenKind::Print),
    ("である", TokenKind::Copula),
    ("受けて", TokenKind::Receive),
    ("または", TokenKind::Or),
    ("変える", TokenKind::Change),
    ("返す", TokenKind::Return),
    ("の間", TokenKind::During),
    ("なら", TokenKind::If),
    ("かつ", TokenKind::And),
    ("なし", TokenKind::None),
    ("真", TokenKind::True),
    ("偽", TokenKind::False),
    ("だ", TokenKind::Copula),
];

const PARTICLES: &[(&str, TokenKind)] = &[
    ("から", TokenKind::From),
    ("より", TokenKind::Than),
    ("だけ", TokenKind::Amount),
    ("は", TokenKind::Topic),
    ("が", TokenKind::Subject),
    ("を", TokenKind::Object),
    ("の", TokenKind::Gen),
    ("に", TokenKind::Locative),
    ("で", TokenKind::Direction),
];

// ---------------------------------------------------------------------------
// Matching helpers
// ---------------------------------------------------------------------------

fn chars_match_str(chars: &[char], index: usize, s: &str) -> bool {
    let s_chars: Vec<char> = s.chars().collect();
    if index + s_chars.len() > chars.len() {
        return false;
    }
    s_chars
        .iter()
        .enumerate()
        .all(|(i, &c)| chars[index + i] == c)
}

/// Try longest-match keyword at the given position.
/// A keyword match is valid only when followed by a character of a different
/// script class than the keyword's last character (or end of input).
fn try_keyword_match(chars: &[char], index: usize) -> Option<(TokenKind, usize)> {
    for &(keyword, kind) in KEYWORDS {
        if !chars_match_str(chars, index, keyword) {
            continue;
        }
        let kw_len = keyword.chars().count();
        let end = index + kw_len;
        let last_char = keyword.chars().last().unwrap();
        if end < chars.len() && script_class(chars[end]) == script_class(last_char) {
            continue;
        }
        return Some((kind, end));
    }
    None
}

/// Try longest-match particle at the given position.
/// Particles are hiragana-only. A match is valid when followed by a character
/// of a different script, end of input, or the start of a known keyword.
fn try_particle_match(chars: &[char], index: usize) -> Option<(TokenKind, usize)> {
    if index >= chars.len() || script_class(chars[index]) != ScriptClass::Hiragana {
        return None;
    }
    for &(particle, kind) in PARTICLES {
        if !chars_match_str(chars, index, particle) {
            continue;
        }
        let p_len = particle.chars().count();
        let end = index + p_len;
        if end < chars.len() && script_class(chars[end]) == ScriptClass::Hiragana {
            // Allow if what follows is a keyword (e.g. は + なし)
            if try_keyword_match(chars, end).is_none() {
                continue;
            }
        }
        return Some((kind, end));
    }
    None
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

struct LexerJa<'a> {
    lines: Vec<&'a str>,
    tokens: Vec<Token>,
    indent_stack: Vec<usize>,
    bracket_depth: usize,
}

impl<'a> LexerJa<'a> {
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
            if is_blank_or_comment_line(&chars[indent..]) {
                return Ok(());
            }
            self.handle_indentation(indent, line_no)?;
            self.lex_content(&chars, indent, line_no)?;
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

    // -----------------------------------------------------------------------
    // Content tokenization — the core difference from the Korean lexer
    // -----------------------------------------------------------------------

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
                    index = self.consume_postfix(chars, index, line_no)?;
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
                    index = self.consume_postfix(chars, index, line_no)?;
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
                    index = self.consume_postfix(chars, index, line_no)?;
                }
                ',' | '\u{3001}' => {
                    self.push_simple(
                        TokenKind::Comma,
                        &ch.to_string(),
                        line_no,
                        index + 1,
                        index + 2,
                    );
                    index += 1;
                }
                ':' => {
                    self.push_simple(TokenKind::Colon, ":", line_no, index + 1, index + 2);
                    index += 1;
                }
                '.' | '\u{3002}' => {
                    self.push_simple(
                        TokenKind::Period,
                        &ch.to_string(),
                        line_no,
                        index + 1,
                        index + 2,
                    );
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
                    index = self.consume_postfix(chars, index, line_no)?;
                }
                c if c.is_ascii_digit() => {
                    index = self.lex_number(chars, index, line_no)?;
                    index = self.consume_postfix(chars, index, line_no)?;
                }
                c if is_ja_identifier_start(c) => {
                    index = self.lex_japanese_token(chars, index, line_no)?;
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

    // -----------------------------------------------------------------------
    // String and number — identical to Korean lexer
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Japanese-specific: two-phase tokenization
    // -----------------------------------------------------------------------

    fn lex_japanese_token(
        &mut self,
        chars: &[char],
        index: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        // Special case: 関数は → Function + FunctionTopic (mirrors Korean 함수는)
        if chars_match_str(chars, index, "関数は") {
            self.push_simple(TokenKind::Function, "関数", line_no, index + 1, index + 3);
            self.push_simple(
                TokenKind::FunctionTopic,
                "は",
                line_no,
                index + 3,
                index + 4,
            );
            return Ok(index + 3);
        }

        // Phase 1: keyword longest match
        if let Some((kind, end)) = try_keyword_match(chars, index) {
            let lexeme: String = chars[index..end].iter().collect();
            self.tokens.push(Token::new(
                kind,
                lexeme,
                Span::new(line_no, index + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        // Phase 2: particle match (hiragana only)
        if let Some((kind, end)) = try_particle_match(chars, index) {
            let lexeme: String = chars[index..end].iter().collect();
            self.tokens.push(Token::new(
                kind,
                lexeme,
                Span::new(line_no, index + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        // Phase 3: collect same-script segment as identifier
        let start_class = script_class(chars[index]);
        let mut end = index + 1;

        match start_class {
            ScriptClass::Latin => {
                while end < chars.len()
                    && (chars[end] == '_' || chars[end].is_ascii_alphanumeric())
                {
                    end += 1;
                }
            }
            ScriptClass::Kanji => {
                while end < chars.len() && script_class(chars[end]) == ScriptClass::Kanji {
                    end += 1;
                }
                // Extend into following hiragana if it is okurigana (送り仮名),
                // i.e. not the start of a keyword or particle.
                // Example: 長さ = 長(kanji) + さ(okurigana) → single Ident("長さ")
                while end < chars.len() && script_class(chars[end]) == ScriptClass::Hiragana {
                    if try_keyword_match(chars, end).is_some() {
                        break;
                    }
                    if try_particle_match(chars, end).is_some() {
                        break;
                    }
                    end += 1;
                }
            }
            _ => {
                while end < chars.len() && script_class(chars[end]) == start_class {
                    end += 1;
                }
            }
        }

        let segment: String = chars[index..end].iter().collect();

        // For hiragana segments, try to split off a trailing keyword or particle.
        // This handles okurigana (送り仮名): e.g. さを → さ(Ident) + を(Object)
        // Analogous to Korean split_attached_word.
        if start_class == ScriptClass::Hiragana
            && let Some((base_chars, kind)) = split_trailing_suffix(&segment) {
                let split_pos = index + base_chars;
                let base: String = chars[index..split_pos].iter().collect();
                let suffix: String = chars[split_pos..end].iter().collect();
                self.tokens.push(Token::new(
                    TokenKind::Ident,
                    base,
                    Span::new(line_no, index + 1, line_no, split_pos + 1),
                ));
                self.tokens.push(Token::new(
                    kind,
                    suffix,
                    Span::new(line_no, split_pos + 1, line_no, end + 1),
                ));
                return Ok(end);
            }

        self.tokens.push(Token::new(
            TokenKind::Ident,
            segment,
            Span::new(line_no, index + 1, line_no, end + 1),
        ));

        Ok(end)
    }

    /// After a number, string, or close bracket, try to consume an immediately
    /// following keyword or particle. Mirrors Korean `consume_attached_word` but
    /// also recognises particles (Korean handles particles via suffix splitting).
    fn consume_postfix(
        &mut self,
        chars: &[char],
        index: usize,
        line_no: usize,
    ) -> Result<usize, LexError> {
        if index >= chars.len() {
            return Ok(index);
        }
        let ch = chars[index];
        if !is_ja_identifier_start(ch) {
            return Ok(index);
        }

        // Try keyword first (e.g. だ after a number → Copula)
        if let Some((kind, end)) = try_keyword_match(chars, index) {
            let lexeme: String = chars[index..end].iter().collect();
            self.tokens.push(Token::new(
                kind,
                lexeme,
                Span::new(line_no, index + 1, line_no, end + 1),
            ));
            return Ok(end);
        }

        // Try particle (e.g. を after a string → Object)
        if script_class(ch) == ScriptClass::Hiragana {
            // Collect the hiragana run and check the particle table
            let mut end = index + 1;
            while end < chars.len() && script_class(chars[end]) == ScriptClass::Hiragana {
                end += 1;
            }
            let segment: String = chars[index..end].iter().collect();
            for &(particle, kind) in PARTICLES {
                if segment == particle {
                    self.tokens.push(Token::new(
                        kind,
                        segment,
                        Span::new(line_no, index + 1, line_no, end + 1),
                    ));
                    return Ok(end);
                }
            }
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

// ---------------------------------------------------------------------------
// Utilities (duplicated from lexer.rs to avoid modifying existing code)
// ---------------------------------------------------------------------------

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

/// Split a trailing keyword or particle from a hiragana segment.
/// Returns the char count of the base (before the suffix) and the suffix's TokenKind.
/// Analogous to Korean `split_attached_word`.
fn split_trailing_suffix(segment: &str) -> Option<(usize, TokenKind)> {
    let seg_len = segment.chars().count();

    for &(keyword, kind) in KEYWORDS {
        if segment.ends_with(keyword) {
            let kw_len = keyword.chars().count();
            if seg_len > kw_len {
                return Some((seg_len - kw_len, kind));
            }
        }
    }

    for &(particle, kind) in PARTICLES {
        if segment.ends_with(particle) {
            let p_len = particle.chars().count();
            if seg_len > p_len {
                return Some((seg_len - p_len, kind));
            }
        }
    }

    None
}
