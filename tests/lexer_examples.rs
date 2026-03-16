use ziium::{LexError, Token, TokenKind, lex};

fn summarize(tokens: &[Token]) -> Vec<String> {
    tokens
        .iter()
        .map(|token| match token.kind {
            TokenKind::Ident => format!("IDENT({:?})", token.lexeme),
            TokenKind::Int => format!("INT({})", token.lexeme),
            TokenKind::Float => format!("FLOAT({})", token.lexeme),
            TokenKind::String => format!("STRING({:?})", token.lexeme),
            TokenKind::Newline => "NEWLINE".to_string(),
            TokenKind::Indent => "INDENT".to_string(),
            TokenKind::Dedent => "DEDENT".to_string(),
            TokenKind::Eof => "EOF".to_string(),
            _ => format!("{:?}({:?})", token.kind, token.lexeme),
        })
        .collect()
}

fn assert_lex(source: &str, expected: &[&str]) {
    let actual = lex(source).expect("lexing should succeed");
    let actual = summarize(&actual);
    let expected = expected
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn lexes_binding_with_attached_topic_particle() {
    assert_lex(
        "이름은 \"철수\"이다.",
        &[
            "IDENT(\"이름\")",
            "Topic(\"은\")",
            "STRING(\"철수\")",
            "Copula(\"이다\")",
            "Period(\".\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_optional_statement_period() {
    assert_lex(
        "\"하하\"를 출력한다.",
        &[
            "STRING(\"하하\")",
            "Object(\"를\")",
            "Print(\"출력한다\")",
            "Period(\".\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_property_chain_and_print() {
    assert_lex(
        "사용자의 주소의 도시를 출력한다",
        &[
            "IDENT(\"사용자\")",
            "Gen(\"의\")",
            "IDENT(\"주소\")",
            "Gen(\"의\")",
            "IDENT(\"도시\")",
            "Object(\"를\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_function_header_with_function_topic() {
    assert_lex(
        "더하기 함수는 왼쪽, 오른쪽을 받아",
        &[
            "IDENT(\"더하기\")",
            "Function(\"함수\")",
            "FunctionTopic(\"는\")",
            "IDENT(\"왼쪽\")",
            "Comma(\",\")",
            "IDENT(\"오른쪽\")",
            "Object(\"을\")",
            "Receive(\"받아\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_keyword_message_statement() {
    assert_lex(
        "과일들에 \"감\" 추가",
        &[
            "IDENT(\"과일들\")",
            "Locative(\"에\")",
            "STRING(\"감\")",
            "IDENT(\"추가\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_transform_call_expression() {
    assert_lex(
        "\"지음\"으로 인사만들기",
        &[
            "STRING(\"지음\")",
            "Direction(\"으로\")",
            "IDENT(\"인사만들기\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_single_syllable_keyword_message_receiver_after_normalization() {
    assert_lex(
        "합에 3 추가",
        &[
            "IDENT(\"합\")",
            "Locative(\"에\")",
            "INT(3)",
            "IDENT(\"추가\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_indented_if_block() {
    assert_lex(
        "참이면\n  \"성인\"을 출력한다\n\"끝\"을 출력한다",
        &[
            "True(\"참\")",
            "If(\"이면\")",
            "NEWLINE",
            "INDENT",
            "STRING(\"성인\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "DEDENT",
            "STRING(\"끝\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_particles_after_parentheses_and_numbers() {
    assert_lex(
        "(마을)을 출력한다\n3으로 바꾼다",
        &[
            "LParen(\"(\")",
            "IDENT(\"마을\")",
            "RParen(\")\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "INT(3)",
            "Direction(\"으로\")",
            "Change(\"바꾼다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_single_syllable_print_target_after_normalization() {
    assert_lex(
        "합을 출력한다",
        &[
            "IDENT(\"합\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_single_syllable_assignment_target_after_normalization() {
    assert_lex(
        "합을 3으로 바꾼다",
        &[
            "IDENT(\"합\")",
            "Object(\"을\")",
            "INT(3)",
            "Direction(\"으로\")",
            "Change(\"바꾼다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn ignores_comments_and_blank_lines() {
    assert_lex(
        "# 주석\n\n이름은 \"철수\"이다  # 뒤쪽 주석\n",
        &[
            "IDENT(\"이름\")",
            "Topic(\"은\")",
            "STRING(\"철수\")",
            "Copula(\"이다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn reports_tab_indentation() {
    let err = lex("참이면\n\t\"성인\"을 출력한다").expect_err("tab indentation should fail");
    assert!(matches!(err, LexError::TabIndentation { .. }));
}
