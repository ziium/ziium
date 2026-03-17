use ziium::{HirExpr, HirSendSelector, HirStmt, parse_source_to_hir};

#[test]
fn lowers_transform_call_to_send_expr() {
    let program = parse_source_to_hir("문장은 \"지음\"으로 인사만들기이다.")
        .expect("hir lowering should succeed");

    match &program.statements[0] {
        HirStmt::Bind { name, value, .. } => {
            assert_eq!(name, "문장");
            match value {
                HirExpr::Send {
                    receiver,
                    selector,
                    ..
                } => {
                    assert!(matches!(
                        receiver.as_ref(),
                        HirExpr::String { value, .. } if value == "지음"
                    ));
                    assert_eq!(selector, &HirSendSelector::Transform("인사만들기".into()));
                }
                other => panic!("expected transform send, got {other:?}"),
            }
        }
        other => panic!("expected bind statement, got {other:?}"),
    }
}

#[test]
fn lowers_property_and_resultive_to_send_exprs() {
    let program = parse_source_to_hir(
        "길이는 과일들의 길이이다.\n원반은 시작탑에서 맨위 원반을 빼낸 것이다.",
    )
    .expect("hir lowering should succeed");

    match &program.statements[0] {
        HirStmt::Bind { value, .. } => match value {
            HirExpr::Send { selector, .. } => {
                assert_eq!(selector, &HirSendSelector::Property("길이".into()));
            }
            other => panic!("expected property send, got {other:?}"),
        },
        other => panic!("expected bind statement, got {other:?}"),
    }

    match &program.statements[1] {
        HirStmt::Bind { value, .. } => match value {
            HirExpr::Send { selector, .. } => {
                assert_eq!(
                    selector,
                    &HirSendSelector::Resultive {
                        role: "맨위 원반".into(),
                        verb: "빼낸".into(),
                    }
                );
            }
            other => panic!("expected resultive send, got {other:?}"),
        },
        other => panic!("expected bind statement, got {other:?}"),
    }
}

#[test]
fn lowers_keyword_message_statement_to_send_stmt() {
    let program = parse_source_to_hir("과일들에 \"감\" 추가.").expect("hir lowering should succeed");

    match &program.statements[0] {
        HirStmt::Send {
            receiver,
            selector,
            args,
            ..
        } => {
            assert!(matches!(receiver, HirExpr::Name { name, .. } if name == "과일들"));
            assert_eq!(selector, &HirSendSelector::Keyword("추가".into()));
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], HirExpr::String { value, .. } if value == "감"));
        }
        other => panic!("expected send statement, got {other:?}"),
    }
}

#[test]
fn lowers_word_binary_to_send_expr() {
    let program = parse_source_to_hir("합은 7 더하기 8이다.").expect("hir lowering should succeed");

    match &program.statements[0] {
        HirStmt::Bind { value, .. } => match value {
            HirExpr::Send {
                receiver,
                selector,
                args,
                ..
            } => {
                assert!(matches!(
                    receiver.as_ref(),
                    HirExpr::Int { raw, .. } if raw == "7"
                ));
                assert_eq!(selector, &HirSendSelector::Word("더하기".into()));
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], HirExpr::Int { raw, .. } if raw == "8"));
            }
            other => panic!("expected word send, got {other:?}"),
        },
        other => panic!("expected bind statement, got {other:?}"),
    }
}

#[test]
fn lowers_named_call_statement() {
    let program = parse_source_to_hir(
        "탑옮기기를 { 원반수: 원반수 빼기 1, 시작, 보조, 목표 }로 호출한다.",
    )
    .expect("hir lowering should succeed");

    match &program.statements[0] {
        HirStmt::NamedCall {
            callee,
            named_args,
            ..
        } => {
            assert!(matches!(callee, HirExpr::Name { name, .. } if name == "탑옮기기"));
            match named_args {
                HirExpr::Record { entries, .. } => {
                    assert_eq!(entries.len(), 4);
                    assert_eq!(entries[0].key, "원반수");
                    assert!(matches!(
                        entries[0].value,
                        HirExpr::Send {
                            selector: HirSendSelector::Word(ref name),
                            ..
                        } if name == "빼기"
                    ));
                    assert_eq!(entries[1].key, "시작");
                    assert_eq!(entries[2].key, "보조");
                    assert_eq!(entries[3].key, "목표");
                }
                other => panic!("expected record args, got {other:?}"),
            }
        }
        other => panic!("expected named call statement, got {other:?}"),
    }
}
