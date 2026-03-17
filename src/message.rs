#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryMessage {
    Length,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeywordMessage {
    Push,
    CanvasClear,
    CanvasFillRect,
    CanvasFillText,
    CanvasDot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResultiveMessage {
    PopTopDisk,
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
        ("맨위 원반", "빼낸") => Some(ResultiveMessage::PopTopDisk),
        _ => None,
    }
}
