#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryMessage {
    Length,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordMessage {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordMessage {
    Push,
    CanvasClear,
    CanvasFillRect,
    CanvasFillText,
    CanvasDot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultiveMessage {
    PopTopElement,
    PopBackElement,
    PopFrontElement,
}

impl WordMessage {
    pub fn name(self) -> &'static str {
        match self {
            WordMessage::Add => "더하기",
            WordMessage::Subtract => "빼기",
            WordMessage::Multiply => "곱하기",
            WordMessage::Divide => "나누기",
        }
    }
}

impl KeywordMessage {
    pub fn name(self) -> &'static str {
        match self {
            KeywordMessage::Push => "추가",
            KeywordMessage::CanvasClear => "지우기",
            KeywordMessage::CanvasFillRect => "사각형채우기",
            KeywordMessage::CanvasFillText => "글자쓰기",
            KeywordMessage::CanvasDot => "점찍기",
        }
    }
}

impl ResultiveMessage {
    pub fn role(self) -> &'static str {
        match self {
            ResultiveMessage::PopTopElement => "맨위 요소",
            ResultiveMessage::PopBackElement => "맨뒤 요소",
            ResultiveMessage::PopFrontElement => "맨앞 요소",
        }
    }

    pub fn verb(self) -> &'static str {
        match self {
            ResultiveMessage::PopTopElement
            | ResultiveMessage::PopBackElement
            | ResultiveMessage::PopFrontElement => "꺼낸",
        }
    }
}

pub(crate) fn unary_message_for_property(name: &str) -> Option<UnaryMessage> {
    match name {
        "길이" => Some(UnaryMessage::Length),
        "제곱" => Some(UnaryMessage::Square),
        _ => None,
    }
}

pub(crate) fn keyword_message_for_selector(selector: &str) -> Option<KeywordMessage> {
    match selector {
        "추가" => Some(KeywordMessage::Push),
        "지우기" => Some(KeywordMessage::CanvasClear),
        "사각형채우기" => Some(KeywordMessage::CanvasFillRect),
        "글자쓰기" => Some(KeywordMessage::CanvasFillText),
        "점찍기" => Some(KeywordMessage::CanvasDot),
        _ => None,
    }
}

pub(crate) fn resultive_message_for(role: &str, verb: &str) -> Option<ResultiveMessage> {
    match (role, verb) {
        ("맨위 요소", "꺼낸") => Some(ResultiveMessage::PopTopElement),
        ("맨뒤 요소", "꺼낸") => Some(ResultiveMessage::PopBackElement),
        ("맨앞 요소", "꺼낸") => Some(ResultiveMessage::PopFrontElement),
        _ => None,
    }
}
