use ziium::run_source;

fn assert_output(source: &str, expected: &[&str]) {
    let result = run_source(source).expect("program should run");
    let expected = expected
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    assert_eq!(result.output, expected);
}

// ---------------------------------------------------------------------------
// 재귀 깊이 가드
// ---------------------------------------------------------------------------

#[test]
fn rejects_recursion_exceeding_depth_limit() {
    let source = r#"세기 함수는 숫자를 받아
  세기(숫자 + 1)

세기(0)"#;
    let err = run_source(source).expect_err("should hit recursion limit");
    assert!(err.to_string().contains("재귀 깊이 제한"));
}
