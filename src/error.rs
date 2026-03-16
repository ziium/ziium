use crate::token::Span;
use std::fmt;

fn write_diagnostic_header(
    f: &mut fmt::Formatter<'_>,
    kind: &str,
    line: Option<usize>,
    column: Option<usize>,
    message: &str,
) -> fmt::Result {
    write!(f, "[{kind}]")?;
    if let (Some(line), Some(column)) = (line, column) {
        write!(f, "\n위치: {line}번째 줄 {column}번째 열")?;
    }
    write!(f, "\n메시지: {message}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    UnexpectedCharacter { ch: char, span: Span },
    UnterminatedString { span: Span },
    TabIndentation { span: Span },
    InconsistentDedent { line: usize, column: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { ch, span } => write_diagnostic_header(
                f,
                "어휘 오류",
                Some(span.start_line),
                Some(span.start_column),
                &format!("예상하지 못한 문자 `{}`를 만났습니다.", ch),
            ),
            Self::UnterminatedString { span } => write_diagnostic_header(
                f,
                "어휘 오류",
                Some(span.start_line),
                Some(span.start_column),
                "문자열이 닫히지 않았습니다.",
            ),
            Self::TabIndentation { span } => write_diagnostic_header(
                f,
                "어휘 오류",
                Some(span.start_line),
                Some(span.start_column),
                "탭을 발견했습니다. v0.1은 탭 들여쓰기를 허용하지 않습니다.",
            ),
            Self::InconsistentDedent { line, column } => write_diagnostic_header(
                f,
                "어휘 오류",
                Some(*line),
                Some(*column),
                "들여쓰기 깊이가 이전 블록과 맞지 않습니다.",
            ),
        }
    }
}

impl std::error::Error for LexError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Option<Span>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_diagnostic_header(
            f,
            "구문 오류",
            self.span.as_ref().map(|span| span.start_line),
            self.span.as_ref().map(|span| span.start_column),
            &self.message,
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub message: String,
    pub span: Option<Span>,
}

impl ResolveError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_diagnostic_header(
            f,
            "이름 해석 오류",
            self.span.as_ref().map(|span| span.start_line),
            self.span.as_ref().map(|span| span.start_column),
            &self.message,
        )
    }
}

impl std::error::Error for ResolveError {}

#[derive(Debug)]
pub enum FrontendError {
    Lex(LexError),
    Parse(ParseError),
    Resolve(ResolveError),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(err) => err.fmt(f),
            Self::Parse(err) => err.fmt(f),
            Self::Resolve(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for FrontendError {}

impl From<LexError> for FrontendError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<ParseError> for FrontendError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<ResolveError> for FrontendError {
    fn from(value: ResolveError) -> Self {
        Self::Resolve(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
    pub span: Option<Span>,
    pub call_stack: Vec<CallFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    pub function_name: String,
    pub span: Option<Span>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            call_stack: Vec::new(),
        }
    }

    pub fn with_span(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
            call_stack: Vec::new(),
        }
    }

    pub fn with_fallback_span(mut self, span: Option<Span>) -> Self {
        if self.span.is_none() {
            self.span = span;
        }
        self
    }

    pub fn with_call_frame(mut self, function_name: impl Into<String>, span: Option<Span>) -> Self {
        self.call_stack.push(CallFrame {
            function_name: function_name.into(),
            span,
        });
        self
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_diagnostic_header(
            f,
            "실행 오류",
            self.span.as_ref().map(|span| span.start_line),
            self.span.as_ref().map(|span| span.start_column),
            &self.message,
        )?;

        if !self.call_stack.is_empty() {
            write!(f, "\n호출 경로:")?;
            for frame in &self.call_stack {
                match &frame.span {
                    Some(span) => write!(
                        f,
                        "\n  `{}` 호출: {}번째 줄 {}번째 열",
                        frame.function_name, span.start_line, span.start_column
                    )?,
                    None => write!(f, "\n  `{}` 호출", frame.function_name)?,
                }
            }
        }

        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug)]
pub enum RunError {
    Frontend(FrontendError),
    Runtime(RuntimeError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frontend(err) => err.fmt(f),
            Self::Runtime(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for RunError {}

impl From<FrontendError> for RunError {
    fn from(value: FrontendError) -> Self {
        Self::Frontend(value)
    }
}

impl From<RuntimeError> for RunError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}
