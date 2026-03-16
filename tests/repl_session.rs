use ziium::InterpreterSession;

#[test]
fn session_keeps_bindings_between_runs() {
    let mut session = InterpreterSession::new();

    let first = session
        .run_source("이름은 \"철수\"이다")
        .expect("first snippet should run");
    assert!(first.output.is_empty());

    let second = session
        .run_source("이름을 출력한다")
        .expect("second snippet should run");
    assert_eq!(second.output, vec!["철수".to_string()]);
}

#[test]
fn session_keeps_functions_between_runs() {
    let mut session = InterpreterSession::new();

    session
        .run_source(
            r#"더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다"#,
        )
        .expect("function definition should run");

    let result = session
        .run_source(
            r#"합은 더하기(2, 3)이다
합을 출력한다"#,
        )
        .expect("function call should run");

    assert_eq!(result.output, vec!["5".to_string()]);
}
