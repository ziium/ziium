use ziium::{BinaryOp, BinarySurface, Expr, Program, RecordEntry, Stmt, UnaryOp, parse_source};

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
                form: BinarySurface::Symbol,
            },
        }]
    );
}

#[test]
fn parses_sleep_statement() {
    let program = parse_source("0.5초 쉬기.").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::Sleep {
            duration_seconds: Expr::Float("0.5".into()),
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
                form: BinarySurface::Word,
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
fn parses_named_call_statement() {
    let program = parse_source("탑옮기기를 { 원반수: 원반수 빼기 1, 시작, 보조, 목표 }로 호출한다.")
        .expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::NamedCall {
            callee: Expr::Name("탑옮기기".into()),
            named_args: Expr::Record(vec![
                RecordEntry {
                    key: "원반수".into(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Name("원반수".into())),
                        op: BinaryOp::Subtract,
                        right: Box::new(Expr::Int("1".into())),
                        form: BinarySurface::Word,
                    },
                },
                RecordEntry {
                    key: "시작".into(),
                    value: Expr::Name("시작".into()),
                },
                RecordEntry {
                    key: "보조".into(),
                    value: Expr::Name("보조".into()),
                },
                RecordEntry {
                    key: "목표".into(),
                    value: Expr::Name("목표".into()),
                },
            ]),
        }]
    );
}

#[test]
fn rejects_unknown_keyword_message_selector() {
    let err = parse_source("과일들에 \"감\" 넣기.").expect_err("parse should fail");
    assert!(err.to_string().contains("현재 키워드 메시지는"));
}

#[test]
fn rejects_push_keyword_message_with_direction() {
    let err = parse_source("과일들에 \"감\" 으로 추가.").expect_err("parse should fail");
    assert!(err.to_string().contains("`추가`는 `<목록>에 <값> 추가` 형식"));
}

#[test]
fn rejects_canvas_keyword_message_without_direction() {
    let err = parse_source("그림판에 { x: 120, y: 80, 색: 빨강 } 점찍기.")
        .expect_err("parse should fail");
    assert!(err
        .to_string()
        .contains("`그림판` 동작은 `그림판에 <레코드>로/으로 <동작>` 형식"));
}

#[test]
fn rejects_canvas_keyword_message_without_canvas_receiver() {
    let err = parse_source("보드에 { x: 120, y: 80, 색: 빨강 }으로 점찍기.")
        .expect_err("parse should fail");
    assert!(err.to_string().contains("`그림판`에만 사용할 수 있습니다"));
}

#[test]
fn rejects_named_call_statement_without_record_args() {
    let err = parse_source("탑옮기기를 원반수로 호출한다.").expect_err("parse should fail");
    assert!(err.to_string().contains("이름 붙은 호출의 인수는 레코드여야 합니다"));
}

#[test]
fn parses_keyword_message_with_record_and_direction() {
    let program = parse_source(
        "그림판에 { x: 120, y: 80, 너비: 180, 높이: 40, 색: \"#d94841\" }으로 사각형채우기.",
    )
    .expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::KeywordMessage {
            receiver: Expr::Name("그림판".into()),
            selector: "사각형채우기".into(),
            arg: Expr::Record(vec![
                RecordEntry {
                    key: "x".into(),
                    value: Expr::Int("120".into()),
                },
                RecordEntry {
                    key: "y".into(),
                    value: Expr::Int("80".into()),
                },
                RecordEntry {
                    key: "너비".into(),
                    value: Expr::Int("180".into()),
                },
                RecordEntry {
                    key: "높이".into(),
                    value: Expr::Int("40".into()),
                },
                RecordEntry {
                    key: "색".into(),
                    value: Expr::String("#d94841".into()),
                },
            ]),
        }]
    );
}

#[test]
fn parses_canvas_dot_message() {
    let program = parse_source("그림판에 { x: 120, y: 80, 색: 빨강 }으로 점찍기.")
        .expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::KeywordMessage {
            receiver: Expr::Name("그림판".into()),
            selector: "점찍기".into(),
            arg: Expr::Record(vec![
                RecordEntry {
                    key: "x".into(),
                    value: Expr::Int("120".into()),
                },
                RecordEntry {
                    key: "y".into(),
                    value: Expr::Int("80".into()),
                },
                RecordEntry {
                    key: "색".into(),
                    value: Expr::Name("빨강".into()),
                },
            ]),
        }]
    );
}

#[test]
fn parses_transform_call_in_binding() {
    let program = parse_source("문장은 \"지음\"으로 인사만들기이다").expect("parse should succeed");
    assert_eq!(
        program.statements,
        vec![Stmt::Bind {
            name: "문장".into(),
            value: Expr::TransformCall {
                input: Box::new(Expr::String("지음".into())),
                callee: "인사만들기".into(),
            },
        }]
    );
}

#[test]
fn parses_resultive_binding() {
    let program = parse_source("원반은 시작탑에서 맨위 원반을 빼낸 것이다.")
        .expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::Bind {
            name: "원반".into(),
            value: Expr::Resultive {
                receiver: Box::new(Expr::Name("시작탑".into())),
                role: "맨위 원반".into(),
                verb: "빼낸".into(),
            },
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
fn parses_record_literal_with_shorthand_entries() {
    let program = parse_source("사람은 { 이름, 나이: 18 }이다").expect("parse should succeed");

    assert_eq!(
        program.statements,
        vec![Stmt::Bind {
            name: "사람".into(),
            value: Expr::Record(vec![
                RecordEntry {
                    key: "이름".into(),
                    value: Expr::Name("이름".into()),
                },
                RecordEntry {
                    key: "나이".into(),
                    value: Expr::Int("18".into()),
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
                        form: BinarySurface::Symbol,
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
                form: BinarySurface::Symbol,
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
인덱스 < 숫자들의 길이인 동안
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
                right: Box::new(Expr::Property {
                    base: Box::new(Expr::Name("숫자들".into())),
                    name: "길이".into(),
                }),
                form: BinarySurface::Symbol,
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
                        form: BinarySurface::Symbol,
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
                    form: BinarySurface::Symbol,
                }),
                op: BinaryOp::Or,
                right: Box::new(Expr::Bool(true)),
                form: BinarySurface::Symbol,
            },
        }
    );
}
