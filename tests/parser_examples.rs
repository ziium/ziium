use ziium::{BinaryOp, Expr, Program, RecordEntry, Stmt, UnaryOp, parse_source};

#[test]
fn parses_string_binding() {
    let program = parse_source("이름은 \"철수\"이다.").expect("parse should succeed");
    assert_eq!(
        program,
        Program {
            statements: vec![Stmt::Bind {
                name: "이름".into(),
                value: Expr::String("철수".into()),
            }],
        }
    );
}

#[test]
fn parses_optional_statement_period() {
    let program =
        parse_source("\"하하\"를 출력한다.\n추가(숫자들, 3).").expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![
            Stmt::Print {
                value: Expr::String("하하".into()),
            },
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Name("추가".into())),
                args: vec![Expr::Name("숫자들".into()), Expr::Int("3".into())],
            }),
        ]
    );
}

#[test]
fn parses_assignment_statement() {
    let program = parse_source("점수를 점수 + 1로 바꾼다").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::Assign {
            name: "점수".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Name("점수".into())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Int("1".into())),
            },
        }]
    );
}

#[test]
fn parses_single_syllable_assignment_statement() {
    let program = parse_source("합을 3으로 바꾼다").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::Assign {
            name: "합".into(),
            value: Expr::Int("3".into()),
        }]
    );
}

#[test]
fn parses_binary_word_message_in_binding() {
    let program = parse_source("합은 7 더하기 8이다").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::Bind {
            name: "합".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Int("7".into())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Int("8".into())),
            },
        }]
    );
}

#[test]
fn parses_keyword_message_statement() {
    let program = parse_source("과일들에 \"감\" 추가.").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::KeywordMessage {
            receiver: Expr::Name("과일들".into()),
            selector: "추가".into(),
            arg: Expr::String("감".into()),
        }]
    );
}

#[test]
fn parses_property_print_statement() {
    let program = parse_source("사용자의 주소의 도시를 출력한다").expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::Print {
            value: Expr::Property {
                base: Box::new(Expr::Property {
                    base: Box::new(Expr::Name("사용자".into())),
                    name: "주소".into(),
                }),
                name: "도시".into(),
            },
        }]
    );
}

#[test]
fn parses_list_and_record_literals() {
    let program = parse_source("설정값은 { 이름: \"앱\", 포트들: [80, 443] }이다")
        .expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::Bind {
            name: "설정값".into(),
            value: Expr::Record(vec![
                RecordEntry {
                    key: "이름".into(),
                    value: Expr::String("앱".into()),
                },
                RecordEntry {
                    key: "포트들".into(),
                    value: Expr::List(vec![Expr::Int("80".into()), Expr::Int("443".into())]),
                },
            ]),
        }]
    );
}

#[test]
fn parses_function_definition_and_call_binding() {
    let source = r#"더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다

결과는 더하기(3, 5)이다"#;
    let program = parse_source(source).expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![
            Stmt::FunctionDef {
                name: "더하기".into(),
                params: vec!["왼쪽".into(), "오른쪽".into()],
                body: vec![Stmt::Return {
                    value: Expr::Binary {
                        left: Box::new(Expr::Name("왼쪽".into())),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Name("오른쪽".into())),
                    },
                }],
            },
            Stmt::Bind {
                name: "결과".into(),
                value: Expr::Call {
                    callee: Box::new(Expr::Name("더하기".into())),
                    args: vec![Expr::Int("3".into()), Expr::Int("5".into())],
                },
            },
        ]
    );
}

#[test]
fn parses_if_else_block() {
    let source = r#"나이는 20이다
나이 >= 20이면
  "성인"을 출력한다
아니면
  "미성년자"를 출력한다"#;
    let program = parse_source(source).expect("parse should succeed");

    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.statements[0], Stmt::Bind { .. }));
    assert_eq!(
        program.statements[1],
        Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Name("나이".into())),
                op: BinaryOp::GreaterEqual,
                right: Box::new(Expr::Int("20".into())),
            },
            then_block: vec![Stmt::Print {
                value: Expr::String("성인".into()),
            }],
            else_block: Some(vec![Stmt::Print {
                value: Expr::String("미성년자".into()),
            }]),
        }
    );
}

#[test]
fn parses_while_block_with_index_and_assignment() {
    let source = r#"인덱스는 0이다
인덱스 < 길이(숫자들)인 동안
  숫자들[인덱스]을 출력한다
  인덱스를 인덱스 + 1로 바꾼다"#;
    let program = parse_source(source).expect("parse should succeed");

    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.statements[0], Stmt::Bind { .. }));

    assert_eq!(
        program.statements[1],
        Stmt::While {
            condition: Expr::Binary {
                left: Box::new(Expr::Name("인덱스".into())),
                op: BinaryOp::Less,
                right: Box::new(Expr::Call {
                    callee: Box::new(Expr::Name("길이".into())),
                    args: vec![Expr::Name("숫자들".into())],
                }),
            },
            body: vec![
                Stmt::Print {
                    value: Expr::Index {
                        base: Box::new(Expr::Name("숫자들".into())),
                        index: Box::new(Expr::Name("인덱스".into())),
                    },
                },
                Stmt::Assign {
                    name: "인덱스".into(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Name("인덱스".into())),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Int("1".into())),
                    },
                },
            ],
        }
    );
}

#[test]
fn parses_standalone_call_and_precedence() {
    let source = r#"추가(숫자들, 3)
결과는 아니다 참 그리고 거짓 또는 참이다"#;
    let program = parse_source(source).expect("parse should succeed");

    assert_eq!(
        program.statements[0],
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Name("추가".into())),
            args: vec![Expr::Name("숫자들".into()), Expr::Int("3".into())],
        })
    );

    assert_eq!(
        program.statements[1],
        Stmt::Bind {
            name: "결과".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(Expr::Bool(true)),
                    }),
                    op: BinaryOp::And,
                    right: Box::new(Expr::Bool(false)),
                }),
                op: BinaryOp::Or,
                right: Box::new(Expr::Bool(true)),
            },
        }
    );
}
