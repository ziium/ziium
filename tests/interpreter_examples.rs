use ziium::{FrontendError, ResolveError, RunError, RuntimeError, run_source};

fn assert_output(source: &str, expected: &[&str]) {
    let result = run_source(source).expect("program should run");
    let expected = expected
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    assert_eq!(result.output, expected);
}

#[test]
fn runs_basic_binding_and_print() {
    let source = r#"이름은 "철수"이다.
나이는 20이다.
이름을 출력한다.
나이를 출력한다."#;
    assert_output(source, &["철수", "20"]);
}

#[test]
fn runs_assignment_and_print() {
    let source = r#"점수는 10이다
점수를 점수 + 5로 바꾼다
점수를 출력한다"#;
    assert_output(source, &["15"]);
}

#[test]
fn runs_if_else_example() {
    let source = r#"나이는 20이다
나이 >= 20이면
  "성인"을 출력한다
아니면
  "미성년자"를 출력한다"#;
    assert_output(source, &["성인"]);
}

#[test]
fn runs_while_loop_example() {
    let source = r#"숫자들은 [3, 5, 8]이다
인덱스는 0이다
인덱스 < 길이(숫자들)인 동안
  숫자들[인덱스]을 출력한다
  인덱스를 인덱스 + 1로 바꾼다"#;
    assert_output(source, &["3", "5", "8"]);
}

#[test]
fn runs_function_example() {
    let source = r#"더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다

합은 더하기(7, 8)이다
합을 출력한다"#;
    assert_output(source, &["15"]);
}

#[test]
fn runs_binary_word_message_example() {
    let source = r#"합은 7 더하기 8이다
합을 출력한다"#;
    assert_output(source, &["15"]);
}

#[test]
fn runs_keyword_message_example() {
    let source = r#"과일들은 ["사과", "배"]이다
과일들에 "감" 추가
과일들의 길이를 출력한다
과일들[2]을 출력한다"#;
    assert_output(source, &["3", "감"]);
}

#[test]
fn runs_no_arg_function_example() {
    let source = r#"시작 함수는 아무것도 받지 않아
  "시작합니다"를 출력한다

시작()"#;
    assert_output(source, &["시작합니다"]);
}

#[test]
fn runs_record_property_example() {
    let source = r#"사람은 { 이름: "영희", 나이: 18 }이다
사람의 이름을 출력한다
사람의 나이를 출력한다"#;
    assert_output(source, &["영희", "18"]);
}

#[test]
fn runs_list_length_property_example() {
    let source = r#"과일들은 ["사과", "배"]이다
추가(과일들, "감")
과일들의 길이를 출력한다"#;
    assert_output(source, &["3"]);
}

#[test]
fn runs_string_length_property_example() {
    let source = r#"인사는 "안녕"이다
인사의 길이를 출력한다"#;
    assert_output(source, &["2"]);
}

#[test]
fn runs_builtin_push_and_length_example() {
    let source = r#"숫자들은 [1, 2]이다
추가(숫자들, 3)
길이(숫자들)을 출력한다
숫자들[2]을 출력한다"#;
    assert_output(source, &["3", "3"]);
}

#[test]
fn reports_redeclaration_error() {
    let source = r#"개수는 1이다
개수는 2이다"#;
    let err = run_source(source).expect_err("program should fail");
    assert!(matches!(
        err,
        RunError::Frontend(FrontendError::Resolve(ResolveError { .. }))
            | RunError::Runtime(RuntimeError { .. })
    ));
    assert!(err.to_string().contains("이미 정의"));
}

#[test]
fn reports_non_boolean_condition_error() {
    let source = r#"개수는 1이다
개수이면
  "있다"를 출력한다"#;
    let err = run_source(source).expect_err("program should fail");
    let message = err.to_string();
    assert!(message.contains("2번째 줄 1번째 열"));
    assert!(message.contains("조건식은 `참` 또는 `거짓`"));
}

#[test]
fn reports_non_callable_value_with_call_site_span() {
    let source = r#"값은 1이다
값()"#;
    let err = run_source(source).expect_err("program should fail");
    let message = err.to_string();
    assert!(message.contains("2번째 줄 2번째 열"));
    assert!(message.contains("호출할 수 없는 값을 호출했습니다"));
}

#[test]
fn reports_nested_call_stack_with_function_names_and_call_sites() {
    let source = r#"안쪽 함수는 아무것도 받지 않아
  "하나" - 1을 출력한다

바깥 함수는 아무것도 받지 않아
  안쪽()

바깥()"#;
    let err = run_source(source).expect_err("program should fail");
    let message = err.to_string();
    assert!(message.contains("숫자 연산은 정수 또는 실수에만 사용할 수 있습니다"));
    assert!(message.contains("호출 경로:"));
    assert!(message.contains("`안쪽` 호출: 5번째 줄 5번째 열"));
    assert!(message.contains("`바깥` 호출: 7번째 줄 3번째 열"));
}

#[test]
fn reports_builtin_call_in_stack_trace() {
    let source = r#"시작 함수는 아무것도 받지 않아
  길이(1)을 출력한다

시작()"#;
    let err = run_source(source).expect_err("program should fail");
    let message = err.to_string();
    assert!(message.contains("`길이` 호출: 2번째 줄 5번째 열"));
    assert!(message.contains("`시작` 호출: 4번째 줄 3번째 열"));
}
