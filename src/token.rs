#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Newline,
    Indent,
    Dedent,
    Eof,
    Ident,
    Int,
    Float,
    String,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Period,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Function,
    FunctionTopic,
    Copula,
    If,
    Else,
    In,
    During,
    Receive,
    Nothing,
    ReceiveNot,
    ReceiveNeg,
    Print,
    Return,
    Change,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Topic,
    Subject,
    Object,
    Gen,
    Locative,
    From,
    Direction,
    ResultMarker,
    Than,
    With,
    Amount,
    Store,
    Exist,
    Each,
    About,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }
}
