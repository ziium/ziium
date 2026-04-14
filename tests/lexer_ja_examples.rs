use ziium::{Token, TokenKind, lex_ja};

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

fn assert_lex_ja(source: &str, expected: &[&str]) {
    let actual = lex_ja(source).expect("lexing should succeed");
    let actual = summarize(&actual);
    let expected = expected
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn lexes_binding_with_script_transition() {
    assert_lex_ja(
        "名前は\"哲夫\"だ。",
        &[
            "IDENT(\"名前\")",
            "Topic(\"は\")",
            "STRING(\"哲夫\")",
            "Copula(\"だ\")",
            "Period(\"。\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_number_binding() {
    assert_lex_ja(
        "年齢は20だ。",
        &[
            "IDENT(\"年齢\")",
            "Topic(\"は\")",
            "INT(20)",
            "Copula(\"だ\")",
            "Period(\"。\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_print_statement() {
    assert_lex_ja(
        "年齢を出力する。",
        &[
            "IDENT(\"年齢\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "Period(\"。\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_string_with_particle() {
    assert_lex_ja(
        "\"哲夫\"を出力する",
        &[
            "STRING(\"哲夫\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_conditional_with_operator() {
    assert_lex_ja(
        "年齢 > 19なら\n  \"成人\"を出力する。\nでなければ\n  \"未成年\"を出力する。",
        &[
            "IDENT(\"年齢\")",
            "Gt(\">\")",
            "INT(19)",
            "If(\"なら\")",
            "NEWLINE",
            "INDENT",
            "STRING(\"成人\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "Period(\"。\")",
            "NEWLINE",
            "DEDENT",
            "Else(\"でなければ\")",
            "NEWLINE",
            "INDENT",
            "STRING(\"未成年\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "Period(\"。\")",
            "NEWLINE",
            "DEDENT",
            "EOF",
        ],
    );
}

#[test]
fn lexes_function_definition() {
    assert_lex_ja(
        "挨拶 関数は 名前を 受けて\n  名前を返す。",
        &[
            "IDENT(\"挨拶\")",
            "Function(\"関数\")",
            "FunctionTopic(\"は\")",
            "IDENT(\"名前\")",
            "Object(\"を\")",
            "Receive(\"受けて\")",
            "NEWLINE",
            "INDENT",
            "IDENT(\"名前\")",
            "Object(\"を\")",
            "Return(\"返す\")",
            "Period(\"。\")",
            "NEWLINE",
            "DEDENT",
            "EOF",
        ],
    );
}

#[test]
fn lexes_gen_particle() {
    assert_lex_ja(
        "名前の長さを出力する",
        &[
            "IDENT(\"名前\")",
            "Gen(\"の\")",
            "IDENT(\"長さ\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_formal_copula() {
    assert_lex_ja(
        "名前は\"太郎\"である。",
        &[
            "IDENT(\"名前\")",
            "Topic(\"は\")",
            "STRING(\"太郎\")",
            "Copula(\"である\")",
            "Period(\"。\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_comment_line() {
    assert_lex_ja(
        "# コメント\n名前は\"太郎\"だ",
        &[
            "IDENT(\"名前\")",
            "Topic(\"は\")",
            "STRING(\"太郎\")",
            "Copula(\"だ\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_arithmetic_with_spaces() {
    assert_lex_ja(
        "合計は 10 + 20だ",
        &[
            "IDENT(\"合計\")",
            "Topic(\"は\")",
            "INT(10)",
            "Plus(\"+\")",
            "INT(20)",
            "Copula(\"だ\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_katakana_identifier() {
    assert_lex_ja(
        "データを出力する",
        &[
            "IDENT(\"データ\")",
            "Object(\"を\")",
            "Print(\"出力する\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_boolean_literals() {
    assert_lex_ja(
        "結果は真だ",
        &[
            "IDENT(\"結果\")",
            "Topic(\"は\")",
            "True(\"真\")",
            "Copula(\"だ\")",
            "NEWLINE",
            "EOF",
        ],
    );
}

#[test]
fn lexes_list_literal() {
    assert_lex_ja(
        "数字は[1、2、3]だ",
        &[
            "IDENT(\"数字\")",
            "Topic(\"は\")",
            "LBracket(\"[\")",
            "INT(1)",
            "Comma(\"、\")",
            "INT(2)",
            "Comma(\"、\")",
            "INT(3)",
            "RBracket(\"]\")",
            "Copula(\"だ\")",
            "NEWLINE",
            "EOF",
        ],
    );
}
