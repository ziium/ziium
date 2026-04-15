use indoc::indoc;
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
    let source = r#"점수에 10을 넣는다
점수를 점수 + 5로 바꾼다
점수를 출력한다"#;
    assert_output(source, &["15"]);
}

#[test]
fn records_sleep_event_from_statement() {
    let source = r#"0.5초 쉬기.
"끝"을 출력한다."#;
    let result = run_source(source).expect("program should run");

    assert_eq!(result.output, vec!["끝".to_string()]);
    assert_eq!(result.events.len(), 2);
    assert_eq!(
        result.events[0],
        ziium::ExecutionEvent::Sleep { seconds: 0.5 }
    );
    assert_eq!(
        result.events[1],
        ziium::ExecutionEvent::Output {
            text: "끝".to_string(),
        }
    );
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
인덱스에 0을 넣는다
인덱스 < 숫자들의 길이인 동안
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
fn runs_transform_call_example() {
    let source = r#"인사만들기 함수는 이름을 받아
  "안녕, " + 이름 + "!"을 돌려준다

문장은 "지음"으로 인사만들기이다
문장을 출력한다"#;
    assert_output(source, &["안녕, 지음!"]);
}

#[test]
fn reports_transform_call_arity_mismatch_for_non_unary_function() {
    let source = r#"합치기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다

결과는 "지음"으로 합치기이다"#;
    let err = run_source(source).expect_err("program should fail");
    let message = err.to_string();
    assert!(message.contains("함수에 넘긴 값의 개수가 맞지 않습니다. 필요: 2, 받음: 1"));
}

#[test]
fn runs_applied_bind_expression() {
    let source = r#"두배 함수는 숫자를 받아
  숫자 * 2를 돌려준다

결과는 5를 두배한 것이다
결과를 출력한다"#;
    assert_output(source, &["10"]);
}

#[test]
fn runs_applied_bind_with_complex_input() {
    let source = r#"절대값 함수는 숫자를 받아
  숫자 < 0이면
    숫자 * -1을 돌려준다
  숫자를 돌려준다

결과는 -7을 절대값한 것이다
결과를 출력한다"#;
    assert_output(source, &["7"]);
}

#[test]
fn runs_inline_if_else_return() {
    let source = indoc! {r#"
        큰값 함수는 가, 나를 받아
          가 > 나이면 가를 돌려주고 아니면 나를 돌려준다

        큰값(3, 7)을 출력한다
    "#};
    assert_output(source, &["7"]);
}

#[test]
fn runs_inline_if_else_print() {
    let source = indoc! {r#"
        점수는 85이다
        점수 >= 90이면 "우수"를 출력하고 아니면 "보통"을 출력한다
    "#};
    assert_output(source, &["보통"]);
}

#[test]
fn runs_inline_if_without_else() {
    let source = indoc! {"
        확인 함수는 숫자를 받아
          숫자 == 0이면 0을 돌려주고
          숫자를 돌려준다

        확인(0)을 출력한다
        확인(5)을 출력한다
    "};
    assert_output(source, &["0", "5"]);
}

#[test]
fn runs_inline_if_else_with_korean_comparison() {
    let source = indoc! {r#"
        가격은 5000이다
        가격이 10000보다 크면 "비싸다"를 출력하고 아니면 "괜찮다"를 출력한다
    "#};
    assert_output(source, &["괜찮다"]);
}

#[test]
fn runs_inline_if_with_manyak_prefix() {
    let source = indoc! {r#"
        숫자는 5이다
        만약 숫자 > 3이면 "크다"를 출력하고 아니면 "작다"를 출력한다
    "#};
    assert_output(source, &["크다"]);
}

#[test]
fn runs_inline_if_else_on_separate_lines() {
    let source = indoc! {"
        최대공약수 함수는 큰수, 작은수를 받아
          만약 작은수 == 0이면 큰수를 돌려주고
          아니면 최대공약수(작은수, 큰수 % 작은수)를 돌려준다

        최대공약수(48, 18)를 출력한다
    "};
    assert_output(source, &["6"]);
}

#[test]
fn runs_gcd_with_inline_if() {
    let source = indoc! {"
        최대공약수 함수는 큰수, 작은수를 받아
          작은수 == 0이면 큰수를 돌려주고 아니면 최대공약수(작은수, 큰수 % 작은수)를 돌려준다

        최대공약수(48, 18)를 출력한다
        최대공약수(100, 75)를 출력한다
        최대공약수(7, 0)을 출력한다
    "};
    assert_output(source, &["6", "25", "7"]);
}

#[test]
fn runs_named_call_statement_example() {
    let source = r#"탑옮기기 함수는 원반수, 시작, 보조, 목표를 받아
  시작 + " -> " + 목표를 출력한다

탑옮기기를 { 원반수: 1, 시작: "A", 보조: "B", 목표: "C" }로 호출한다"#;
    assert_output(source, &["A -> C"]);
}

#[test]
fn runs_square_property_example() {
    let source = r#"결과는 5의 제곱이다
결과를 출력한다
실수결과는 1.5의 제곱이다
실수결과를 출력한다"#;
    assert_output(source, &["25", "2.25"]);
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
fn runs_record_shorthand_example() {
    let source = r#"이름은 "영희"이다
사람은 { 이름, 나이: 18 }이다
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
fn record_key_named_length_wins_over_builtin_length_property() {
    let source = r#"상자는 { 길이: 99, 폭: 3 }이다
상자의 길이를 출력한다"#;
    assert_output(source, &["99"]);
}

#[test]
fn record_without_length_key_falls_back_to_record_length_property() {
    let source = r#"상자는 { 폭: 3, 높이: 4 }이다
상자의 길이를 출력한다"#;
    assert_output(source, &["2"]);
}

#[test]
fn record_key_named_square_wins_over_builtin_square_property() {
    let source = r#"수상자는 { 제곱: 7 }이다
수상자의 제곱을 출력한다"#;
    assert_output(source, &["7"]);
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
fn runs_pop_last_builtin_example() {
    let source = r#"숫자들은 [1, 2, 3]이다
마지막은 마지막꺼내기(숫자들)이다
마지막을 출력한다
숫자들의 길이를 출력한다"#;
    assert_output(source, &["3", "2"]);
}

#[test]
fn runs_resultive_binding_example() {
    let source = r#"시작탑은 { 원반들: [3, 2, 1] }이다
원반은 시작탑의 원반들에서 맨위 요소를 꺼낸 것이다.
원반을 출력한다.
시작탑의 원반들의 길이를 출력한다."#;
    assert_output(source, &["1", "2"]);
}

#[test]
fn runs_back_resultive_binding_example() {
    let source = r#"목록은 [1, 2, 3]이다
마지막은 목록에서 맨뒤 요소를 꺼낸 것이다.
마지막을 출력한다.
목록의 길이를 출력한다."#;
    assert_output(source, &["3", "2"]);
}

#[test]
fn runs_front_resultive_binding_example() {
    let source = r#"목록은 [1, 2, 3]이다
처음은 목록에서 맨앞 요소를 꺼낸 것이다.
처음을 출력한다.
목록의 길이를 출력한다."#;
    assert_output(source, &["1", "2"]);
}

#[test]
fn runs_resultive_statement_example() {
    let source = r#"시작탑은 { 원반들: [3, 2, 1] }이다
시작탑의 원반들에서 맨위 요소를 꺼낸다.
시작탑의 원반들의 길이를 출력한다."#;
    assert_output(source, &["2"]);
}

#[test]
fn runs_back_resultive_statement_example() {
    let source = r#"목록은 [1, 2, 3]이다
목록에서 맨뒤 요소를 꺼낸다.
목록의 길이를 출력한다."#;
    assert_output(source, &["2"]);
}

#[test]
fn runs_front_resultive_statement_example() {
    let source = r#"목록은 [1, 2, 3]이다
목록에서 맨앞 요소를 꺼낸다.
목록의 길이를 출력한다."#;
    assert_output(source, &["2"]);
}

#[test]
fn records_canvas_frames_from_keyword_messages() {
    let source = r##"빨강은 "#d94841"이다.
그림판에 { 배경색: "#f6efe2" }으로 지우기.
그림판에 { x: 120, y: 80, 색: 빨강 }으로 점찍기.
그림판에 { 글: "지음", x: 160, y: 60, 색: "#3b2f2f", 크기: 24 }로 글자쓰기."##;
    let result = run_source(source).expect("program should run");

    assert!(result.output.is_empty());
    assert_eq!(result.canvas_frames.len(), 1);
    assert_eq!(result.canvas_frames[0].commands.len(), 3);
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

// ---------------------------------------------------------------------------
// 단음절 식별자 (P-1: "가"/"이"를 변수/매개변수로 사용)
// ---------------------------------------------------------------------------

#[test]
fn allows_single_syllable_ga_as_variable() {
    let source = "가는 10이다\n가를 출력한다";
    assert_output(source, &["10"]);
}

#[test]
fn allows_single_syllable_ga_as_parameter() {
    let source = "더하기 함수는 가, 나를 받아\n  가 + 나를 돌려준다\n\n더하기(3, 7)을 출력한다";
    assert_output(source, &["10"]);
}

#[test]
fn allows_single_syllable_ee_as_variable() {
    let source = "이는 5이다\n이를 출력한다";
    assert_output(source, &["5"]);
}

#[test]
fn preserves_subject_particle_after_number() {
    let source = "3이 3과 같으면\n  \"예\"를 출력한다";
    assert_output(source, &["예"]);
}

#[test]
fn preserves_subject_particle_after_string() {
    let source = r#""가"가 "가"와 같으면
  "예"를 출력한다"#;
    assert_output(source, &["예"]);
}

#[test]
fn preserves_subject_particle_after_paren() {
    let source = "(2 + 3)이 5와 같으면\n  \"예\"를 출력한다";
    assert_output(source, &["예"]);
}

// ---------------------------------------------------------------------------
// P-2: 다음절 식별자의 조사 접미사 오분리 방지 (하노이/고양이 등)
// ---------------------------------------------------------------------------

#[test]
fn allows_identifier_ending_in_ee() {
    // "하노이"가 "하노"+"이"로 분리되지 않아야 함
    let source = r#"하노이는 "탑"이다
하노이를 출력한다"#;
    assert_output(source, &["탑"]);
}

#[test]
fn allows_identifier_ending_in_ga() {
    // "요가"가 "요"+"가"로 분리되지 않아야 함
    let source = r#"요가는 "운동"이다
요가를 출력한다"#;
    assert_output(source, &["운동"]);
}

#[test]
fn allows_identifier_ending_in_eul_as_param() {
    // "마을"이 "마"+"을"로 분리되지 않아야 함
    let source = r#"마을은 "서울"이다
마을을 출력한다"#;
    assert_output(source, &["서울"]);
}

#[test]
fn allows_identifier_ending_in_ee_in_function_def() {
    // 함수 이름 "하노이"가 정의에서 분리되지 않아야 함
    let source = r#"하노이 함수는 숫자를 받아
  숫자를 돌려준다

하노이(42)를 출력한다"#;
    assert_output(source, &["42"]);
}

#[test]
fn allows_identifier_ending_in_ee_in_comparison() {
    // "고양이가 ... 같으면" — 주격 분리가 정상 작동
    let source = r#"고양이는 "야옹"이다
고양이가 "야옹"과 같으면
  "맞다"를 출력한다"#;
    assert_output(source, &["맞다"]);
}

// ---------------------------------------------------------------------------
// P-3: 키워드 접미사("인") 무조건 분리 방지
// ---------------------------------------------------------------------------

#[test]
fn allows_identifier_ending_in_in() {
    // "확인"이 "확"+"인"으로 분리되지 않아야 함
    let source = "확인 함수는 값을 받아\n  값을 돌려준다\n\n확인(42)를 출력한다";
    assert_output(source, &["42"]);
}

#[test]
fn allows_compound_identifier_ending_in_in() {
    // "회문확인"이 "회문확"+"인"으로 분리되지 않아야 함
    let source = "회문확인 함수는 글을 받아\n  글을 돌려준다\n\n회문확인(\"aba\")를 출력한다";
    assert_output(source, &["aba"]);
}

#[test]
fn splits_in_before_during_for_while_loop() {
    // "길이인 동안" → "길이"+"인"+"동안"으로 정상 분리
    let source = r#"숫자들은 [3, 5, 8]이다
인덱스에 0을 넣는다
인덱스 < 숫자들의 길이인 동안
  숫자들[인덱스]을 출력한다
  인덱스를 인덱스 + 1로 바꾼다"#;
    assert_output(source, &["3", "5", "8"]);
}

// ---------------------------------------------------------------------------
// P-4: while 본문 스코프 — 반복 간 바인딩 독립
// ---------------------------------------------------------------------------

#[test]
fn allows_binding_inside_while_loop() {
    // 매 반복마다 새 변수를 바인딩할 수 있어야 함
    let source = r#"목록은 [10, 20, 30]이다
인덱스에 0을 넣는다
인덱스 < 목록의 길이인 동안
  값은 목록[인덱스]이다
  값을 출력한다
  인덱스를 인덱스 + 1로 바꾼다"#;
    assert_output(source, &["10", "20", "30"]);
}

#[test]
fn while_body_does_not_leak_bindings_to_outer_scope() {
    // while 본문에서 정의한 변수가 외부로 누출되면 안 됨
    let source = r#"횟수에 0을 넣는다
횟수 < 1인 동안
  임시는 "안녕"이다
  횟수를 횟수 + 1로 바꾼다
임시를 출력한다"#;
    let err = ziium::run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("정의되지 않았습니다"));
}

// --- 인덱스 대입 ---

#[test]
fn runs_index_assignment() {
    let source = r#"숫자들은 [10, 20, 30]이다
숫자들[1]을 99로 바꾼다
숫자들[1]을 출력한다"#;
    assert_output(source, &["99"]);
}

#[test]
fn runs_index_assignment_with_expression_index() {
    let source = r#"목록은 [1, 2, 3, 4, 5]이다
위치는 2이다
목록[위치 + 1]을 100으로 바꾼다
목록[3]을 출력한다"#;
    assert_output(source, &["100"]);
}

#[test]
fn runs_index_assignment_swap() {
    let source = r#"숫자들은 [5, 3]이다
임시는 숫자들[0]이다
숫자들[0]을 숫자들[1]로 바꾼다
숫자들[1]을 임시로 바꾼다
숫자들[0]을 출력한다
숫자들[1]을 출력한다"#;
    assert_output(source, &["3", "5"]);
}

#[test]
fn runs_index_relative_change() {
    let source = r#"숫자들은 [10, 20, 30]이다
숫자들[0]을 3만큼 줄인다
숫자들[2]를 5만큼 늘린다
숫자들[0]을 출력한다
숫자들[2]를 출력한다"#;
    assert_output(source, &["7", "35"]);
}

#[test]
fn rejects_index_assignment_out_of_bounds() {
    let source = r#"숫자들은 [10, 20]이다
숫자들[5]를 99로 바꾼다"#;
    let err = run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("범위를 벗어났습니다"));
}

#[test]
fn rejects_index_assignment_negative_index() {
    let source = r#"숫자들은 [10, 20]이다
숫자들[-1]을 99로 바꾼다"#;
    let err = run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("0 이상의 정수"));
}

#[test]
fn rejects_index_assignment_on_string() {
    let source = r#"텍스트는 "안녕"이다
텍스트[0]을 "가"로 바꾼다"#;
    let err = run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("목록에만"));
}

#[test]
fn rejects_index_assignment_on_undefined_variable() {
    let err = run_source(r#"없는변수[0]을 99로 바꾼다"#).expect_err("should fail");
    assert!(err.to_string().contains("정의되지 않았습니다"));
}

#[test]
fn runs_mutable_bind_basic() {
    let source = indoc! {"
        횟수에 0을 넣는다
        횟수를 횟수 + 1로 바꾼다
        횟수를 출력한다
    "};
    assert_output(source, &["1"]);
}

#[test]
fn runs_mutable_bind_with_expression() {
    let source = indoc! {"
        목록은 [10, 20, 30]이다
        오른쪽에 목록의 길이 - 1을 넣는다
        오른쪽을 출력한다
    "};
    assert_output(source, &["2"]);
}

#[test]
fn runs_mutable_bind_with_inline_if() {
    let source = indoc! {"
        x는 5이다
        만약 x > 0이면 결과에 x를 넣고 아니면 결과에 0을 넣는다
        결과를 출력한다
    "};
    assert_output(source, &["5"]);
}

#[test]
fn allows_index_assign_on_const_binding() {
    // D2: const는 바인딩 불변, 내용 가변 (JS const 의미론)
    let source = indoc! {"
        목록은 [1, 2, 3]이다
        목록[0]을 99로 바꾼다
        목록[0]을 출력한다
    "};
    assert_output(source, &["99"]);
}

#[test]
fn rejects_reassign_on_const_binding() {
    // D2: const 바인딩 자체를 교체하면 에러
    let source = indoc! {"
        목록은 [1, 2, 3]이다
        목록을 [4, 5, 6]으로 바꾼다
    "};
    let err = run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("변경할 수 없습니다"));
}

// ---------------------------------------------------------------------------
// for-each 반복문
// ---------------------------------------------------------------------------

#[test]
fn runs_foreach_basic() {
    let source = indoc! {"
        과일들은 [\"사과\", \"배\", \"감\"]이다
        과일들의 각각 과일에 대해
          과일을 출력한다
    "};
    assert_output(source, &["사과", "배", "감"]);
}

#[test]
fn runs_foreach_sum() {
    let source = indoc! {"
        숫자들은 [10, 20, 30, 40]이다
        합계에 0을 넣는다
        숫자들의 각각 항목에 대해
          합계를 합계 + 항목으로 바꾼다
        합계를 출력한다
    "};
    assert_output(source, &["100"]);
}

#[test]
fn runs_foreach_empty_list() {
    let source = indoc! {"
        빈목록은 []이다
        빈목록의 각각 항목에 대해
          항목을 출력한다
        \"끝\"을 출력한다
    "};
    assert_output(source, &["끝"]);
}

#[test]
fn runs_foreach_scope_isolation() {
    // 반복 변수는 블록 밖에서 접근 불가 — 외부 같은 이름 변수에 영향 없음
    let source = indoc! {"
        항목은 \"원래값\"이다
        목록은 [1, 2, 3]이다
        목록의 각각 항목에 대해
          항목을 출력한다
        항목을 출력한다
    "};
    assert_output(source, &["1", "2", "3", "원래값"]);
}

#[test]
fn runs_foreach_mutable_outer() {
    // 외부 가변 변수를 for-each 안에서 수정 가능
    let source = indoc! {"
        결과에 \"\"을 넣는다
        단어들은 [\"안\", \"녕\", \"하\", \"세\", \"요\"]이다
        단어들의 각각 글자에 대해
          결과를 결과 + 글자로 바꾼다
        결과를 출력한다
    "};
    assert_output(source, &["안녕하세요"]);
}

#[test]
fn runs_foreach_nested() {
    let source = indoc! {"
        행들은 [1, 2]이다
        열들은 [10, 20]이다
        행들의 각각 행에 대해
          열들의 각각 열에 대해
            (행 * 100 + 열)을 출력한다
    "};
    assert_output(source, &["110", "120", "210", "220"]);
}

#[test]
fn rejects_foreach_non_list() {
    let source = indoc! {"
        숫자는 42이다
        숫자의 각각 항목에 대해
          항목을 출력한다
    "};
    let err = run_source(source).expect_err("should fail on non-list");
    assert!(err.to_string().contains("목록이어야"));
}

// ---------------------------------------------------------------------------
// 존재 바인딩 (`있다`)
// ---------------------------------------------------------------------------

#[test]
fn runs_exist_binding_basic() {
    let source = indoc! {"
        바구니에 [1, 2, 3]이 있다
        바구니를 출력한다
    "};
    assert_output(source, &["[1, 2, 3]"]);
}

#[test]
fn runs_exist_binding_with_ga() {
    let source = indoc! {"
        상자에 \"보물\"가 있다
        상자를 출력한다
    "};
    assert_output(source, &["보물"]);
}

#[test]
fn rejects_exist_binding_reassign() {
    let source = indoc! {"
        바구니에 [1, 2, 3]이 있다
        바구니를 [4, 5, 6]으로 바꾼다
    "};
    let err = run_source(source).expect_err("should fail");
    assert!(err.to_string().contains("변경할 수 없습니다"));
}

#[test]
fn runs_exist_binding_with_topic() {
    // `에는` 형태: `바구니에는 [...]이 있다`
    let source = indoc! {"
        바구니에는 [\"사과\", \"배\"]가 있다
        바구니를 출력한다
    "};
    assert_output(source, &["[사과, 배]"]);
}

#[test]
fn runs_mutable_bind_with_topic() {
    // `에는` 형태: `횟수에는 0을 넣는다`
    let source = indoc! {"
        횟수에는 0을 넣는다
        횟수를 횟수 + 1로 바꾼다
        횟수를 출력한다
    "};
    assert_output(source, &["1"]);
}

#[test]
fn runs_exist_binding_with_foreach() {
    let source = indoc! {"
        과일들에 [\"사과\", \"배\", \"감\"]이 있다
        과일들의 각각 과일에 대해
          과일을 출력한다
    "};
    assert_output(source, &["사과", "배", "감"]);
}

// ---------------------------------------------------------------------------
// Choose 결과적 프레임
// ---------------------------------------------------------------------------

#[test]
fn runs_choose_resultive_bind_default() {
    let source = indoc! {r#"
        선택지는 ["사과", "배", "감"]이다
        결과는 선택지에서 고른 것이다
        결과를 출력한다
    "#};
    assert_output(source, &["사과"]);
}

#[test]
fn runs_choose_effect_default() {
    let source = indoc! {r#"
        선택지는 ["사과", "배", "감"]이다
        선택지에서 고른다
        "완료"를 출력한다
    "#};
    assert_output(source, &["완료"]);
}

#[test]
fn rejects_choose_on_empty_list() {
    let source = indoc! {r#"
        빈목록은 []이다
        결과는 빈목록에서 고른 것이다
    "#};
    let err = run_source(source).expect_err("should fail on empty list");
    assert!(err.to_string().contains("빈 목록에서는 고를 수 없습니다"));
}

#[test]
fn rejects_choose_on_non_list() {
    let source = indoc! {r#"
        숫자는 42이다
        결과는 숫자에서 고른 것이다
    "#};
    let err = run_source(source).expect_err("should fail on non-list");
    assert!(err
        .to_string()
        .contains("목록에만 사용할 수 있습니다"));
}
