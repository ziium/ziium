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
fn lexes_keyword_message_statement_with_direction() {
    assert_lex(
        "그림판에 { 배경색: \"#f6efe2\" }으로 지우기.",
        &[
            "IDENT(\"그림판\")",
            "Locative(\"에\")",
            "LBrace(\"{\")",
            "IDENT(\"배경색\")",
            "Colon(\":\")",
            "STRING(\"#f6efe2\")",
            "RBrace(\"}\")",
            "Direction(\"으로\")",
            "IDENT(\"지우기\")",
            "Period(\".\")",
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

#[test]
fn rejects_unterminated_string() {
    let err = lex("이름은 \"철수이다").expect_err("unterminated string should fail");
    assert!(matches!(err, LexError::UnterminatedString { .. }));
}

#[test]
fn rejects_unexpected_character() {
    let err = lex("@이름은 1이다").expect_err("unexpected character should fail");
    assert!(matches!(err, LexError::UnexpectedCharacter { ch: '@', .. }));
}

#[test]
fn rejects_inconsistent_dedent() {
    let source = "참이면\n    \"안\"을 출력한다\n  \"밖\"을 출력한다";
    let err = lex(source).expect_err("inconsistent dedent should fail");
    assert!(matches!(err, LexError::InconsistentDedent { .. }));
}

// P-5: 조사 음절로 끝나는 식별자 보호

#[test]
fn keeps_with_syllable_in_bare_identifier_before_comma() {
    assert_lex(
        "갱신 함수는 기울기결과, 학습률을 받아",
        &[
            "IDENT(\"갱신\")",
            "Function(\"함수\")",
            "FunctionTopic(\"는\")",
            "IDENT(\"기울기결과\")",
            "Comma(\",\")",
            "IDENT(\"학습률\")",
            "Object(\"을\")",
            "Receive(\"받아\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_with_syllable_in_bare_identifier_before_paren() {
    assert_lex(
        "역전파(신경망, 순전파결과)",
        &[
            "IDENT(\"역전파\")",
            "LParen(\"(\")",
            "IDENT(\"신경망\")",
            "Comma(\",\")",
            "IDENT(\"순전파결과\")",
            "RParen(\")\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_with_syllable_before_binary_operator() {
    assert_lex(
        "값은 순전파결과 - 1이다",
        &[
            "IDENT(\"값\")",
            "Topic(\"은\")",
            "IDENT(\"순전파결과\")",
            "Minus(\"-\")",
            "INT(1)",
            "Copula(\"이다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_object_syllable_before_operator_index_and_call() {
    assert_lex(
        "값은 작은마을 + 1이다",
        &[
            "IDENT(\"값\")",
            "Topic(\"은\")",
            "IDENT(\"작은마을\")",
            "Plus(\"+\")",
            "INT(1)",
            "Copula(\"이다\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "이동경로[0]을 출력한다",
        &[
            "IDENT(\"이동경로\")",
            "LBracket(\"[\")",
            "INT(0)",
            "RBracket(\"]\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "작은마을()을 출력한다",
        &[
            "IDENT(\"작은마을\")",
            "LParen(\"(\")",
            "RParen(\")\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "값에 (1 + 2)를 넣는다",
        &[
            "IDENT(\"값\")",
            "Locative(\"에\")",
            "LParen(\"(\")",
            "INT(1)",
            "Plus(\"+\")",
            "INT(2)",
            "RParen(\")\")",
            "Object(\"를\")",
            "Store(\"넣는다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_rang_syllable_in_bare_identifier() {
    assert_lex(
        "(첫사랑)을 출력한다",
        &[
            "LParen(\"(\")",
            "IDENT(\"첫사랑\")",
            "RParen(\")\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn splits_with_particle_only_before_comparison_verb() {
    assert_lex(
        "점수가 최고점과 같으면",
        &[
            "IDENT(\"점수\")",
            "Subject(\"가\")",
            "IDENT(\"최고점\")",
            "With(\"과\")",
            "IDENT(\"같으면\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "값이 나이랑 같으면",
        &[
            "IDENT(\"값\")",
            "Subject(\"이\")",
            "IDENT(\"나이\")",
            "With(\"랑\")",
            "IDENT(\"같으면\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "맞음여부가 참과 같으면",
        &[
            "IDENT(\"맞음여부\")",
            "Subject(\"가\")",
            "True(\"참\")",
            "With(\"과\")",
            "IDENT(\"같으면\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "값이 x와 같으면",
        &[
            "IDENT(\"값\")",
            "Subject(\"이\")",
            "IDENT(\"x\")",
            "With(\"와\")",
            "IDENT(\"같으면\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "점수가 19와 다르면",
        &[
            "IDENT(\"점수\")",
            "Subject(\"가\")",
            "INT(19)",
            "With(\"와\")",
            "IDENT(\"다르면\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_object_syllable_in_bare_identifier_before_closer() {
    assert_lex(
        "(작은마을)을 출력한다",
        &[
            "LParen(\"(\")",
            "IDENT(\"작은마을\")",
            "RParen(\")\")",
            "Object(\"을\")",
            "Print(\"출력한다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_direction_syllable_in_record_key_and_param() {
    assert_lex(
        "{ 이동경로: 1 }",
        &[
            "LBrace(\"{\")",
            "IDENT(\"이동경로\")",
            "Colon(\":\")",
            "INT(1)",
            "RBrace(\"}\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "열기 함수는 파일경로, 모드를 받아",
        &[
            "IDENT(\"열기\")",
            "Function(\"함수\")",
            "FunctionTopic(\"는\")",
            "IDENT(\"파일경로\")",
            "Comma(\",\")",
            "IDENT(\"모드\")",
            "Object(\"를\")",
            "Receive(\"받아\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn keeps_particle_syllable_in_function_name() {
    assert_lex(
        "작은마을 함수는 아무것도 받지 않아",
        &[
            "IDENT(\"작은마을\")",
            "Function(\"함수\")",
            "FunctionTopic(\"는\")",
            "Nothing(\"아무것도\")",
            "ReceiveNot(\"받지\")",
            "ReceiveNeg(\"않아\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn still_splits_attached_particles_before_expression_or_verb() {
    assert_lex(
        "값을 파일경로로 바꾼다",
        &[
            "IDENT(\"값\")",
            "Object(\"을\")",
            "IDENT(\"파일경로\")",
            "Direction(\"로\")",
            "Change(\"바꾼다\")",
            "NEWLINE",
            "EOF",
        ],
    );
    assert_lex(
        "결과는 -7을 절대값한 것이다",
        &[
            "IDENT(\"결과\")",
            "Topic(\"는\")",
            "Minus(\"-\")",
            "INT(7)",
            "Object(\"을\")",
            "IDENT(\"절대값한\")",
            "ResultMarker(\"것이다\")",
            "NEWLINE",
            "EOF",
        ],
    );
}
