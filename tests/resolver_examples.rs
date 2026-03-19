use ziium::{FrontendError, ResolveError, RunError, run_source};

#[test]
fn reports_undefined_name_before_runtime() {
    let err = run_source("이름을 출력한다").expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    let message = err.to_string();
    assert!(message.contains("1번째 줄 1번째 열"));
    assert!(message.contains("아직 정의되지 않았습니다"));
}

#[test]
fn reports_assign_to_undefined_name_before_runtime() {
    let err = run_source("점수를 3으로 바꾼다").expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    let message = err.to_string();
    assert!(message.contains("1번째 줄 1번째 열"));
    assert!(message.contains("아직 정의되지 않았습니다"));
}

#[test]
fn rejects_undefined_transform_callee_before_runtime() {
    let err = run_source("문장은 \"지음\"으로 인사만들기이다").expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    let message = err.to_string();
    assert!(message.contains("1번째 줄 9번째 열"));
    assert!(message.contains("아직 정의되지 않았습니다"));
}

#[test]
fn reports_return_outside_function_before_runtime() {
    let err = run_source("3을 돌려준다").expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    let message = err.to_string();
    assert!(message.contains("1번째 줄 4번째 열"));
    assert!(message.contains("함수 본문 안에서만"));
}

#[test]
fn rejects_branch_local_name_after_if() {
    let err = run_source(
        r#"참이면
  이름은 "철수"이다
이름을 출력한다"#,
    )
    .expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    assert!(err.to_string().contains("아직 정의되지 않았습니다"));
}

#[test]
fn allows_name_bound_in_both_if_branches_after_merge() {
    let result = run_source(
        r#"참이면
  이름은 "철수"이다
아니면
  이름은 "영희"이다

이름을 출력한다"#,
    )
    .expect("program should run");
    assert_eq!(result.output, vec!["철수".to_string()]);
}

#[test]
fn rejects_name_bound_only_in_loop_after_loop() {
    let err = run_source(
        r#"거짓인 동안
  이름은 "철수"이다

이름을 출력한다"#,
    )
    .expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    assert!(err.to_string().contains("아직 정의되지 않았습니다"));
}

#[test]
fn rejects_unknown_name_in_direct_function_body() {
    let err = run_source(
        r#"시작 함수는 아무것도 받지 않아
  없는것을 출력한다

시작()"#,
    )
    .expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
    ));
    assert!(err.to_string().contains("아직 정의되지 않았습니다"));
}

#[test]
fn allows_late_bound_global_from_function_body() {
    let result = run_source(
        r#"시작 함수는 아무것도 받지 않아
  값을 출력한다

값은 "하하"이다
시작()"#,
    )
    .expect("program should run");
    assert_eq!(result.output, vec!["하하".to_string()]);
}

#[test]
fn allows_nested_function_to_capture_late_bound_outer_name() {
    let result = run_source(
        r#"바깥 함수는 아무것도 받지 않아
  안쪽 함수는 아무것도 받지 않아
    값을 출력한다

  값은 "하하"이다
  안쪽()

바깥()"#,
    )
    .expect("program should run");
    assert_eq!(result.output, vec!["하하".to_string()]);
}
