use indoc::indoc;
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

// Python: sys.setrecursionlimit(N); 초과 시 RecursionError
// Ruby:   SystemStackError on deep recursion
//
// def count(n): count(n + 1)
// count(0)  # => RecursionError
#[test]
fn rejects_recursion_exceeding_depth_limit() {
    let source = indoc! {"
        세기 함수는 숫자를 받아
          세기(숫자 + 1)

        세기(0)
    "};
    let err = run_source(source).expect_err("should hit recursion limit");
    assert!(err.to_string().contains("재귀 깊이 제한"));
}

// ===========================================================================
// 배치 1: 기능 테스트 — 연산자 / 조건문 / 반복문
// ===========================================================================

// ---------------------------------------------------------------------------
// 연산: 곱셈/나눗셈/나머지/같다/다르다/단항 마이너스
// ---------------------------------------------------------------------------

// Python: area = 6 * 4; half = area // 2
// Ruby:   area = 6 * 4; half = area / 2
#[test]
fn runs_multiplication_and_division() {
    let source = indoc! {"
        넓이는 6 * 4이다
        넓이를 출력한다
        반은 넓이 / 2이다
        반을 출력한다
    "};
    assert_output(source, &["24", "12"]);
}

// Ruby: area = 6.*(4)   — 메서드 호출 형태의 산술
// 지음의 워드 바이너리 프레임은 Ruby의 메서드 호출 산술과 유사한 발상
#[test]
fn runs_word_binary_multiply_and_divide() {
    let source = indoc! {"
        넓이는 6 곱하기 4이다
        넓이를 출력한다
        반은 넓이 나누기 2이다
        반을 출력한다
    "};
    assert_output(source, &["24", "12"]);
}

// Python: 17 // 5 == 3; 17 % 5 == 2
// Ruby:   17 / 5 == 3; 17 % 5 == 2
#[test]
fn runs_modulo_operator() {
    let source = indoc! {"
        몫은 17 / 5이다
        몫을 출력한다
        나머지는 17 % 5이다
        나머지를 출력한다
    "};
    assert_output(source, &["3", "2"]);
}

// Python: 3 == 3  # True;  3 != 5  # True;  3 == 5  # False
// Ruby:   3 == 3  # true;  3 != 5  # true;  3 == 5  # false
#[test]
fn runs_equality_and_inequality() {
    let source = indoc! {"
        같은지는 3 == 3이다
        같은지를 출력한다
        다른지는 3 != 5이다
        다른지를 출력한다
        거짓인지는 3 == 5이다
        거짓인지를 출력한다
    "};
    assert_output(source, &["참", "참", "거짓"]);
}

// Python: x = 10; y = -x  # -10;  x + y  # 0
// Ruby:   x = 10; y = -x  # -10;  x + y  # 0
#[test]
fn runs_unary_negate() {
    let source = indoc! {"
        양수는 10이다
        음수는 -양수이다
        음수를 출력한다
        합은 양수 + 음수이다
        합을 출력한다
    "};
    assert_output(source, &["-10", "0"]);
}

// ---------------------------------------------------------------------------
// 조건문: 중첩 if, 한국어 비교 프레임
// ---------------------------------------------------------------------------

// Python:
//   score = 85
//   if score >= 90:   print("A")
//   elif score >= 80: print("B")
//   else:             print("C")
//
// Ruby:
//   case score
//   when 90.. then puts "A"
//   when 80.. then puts "B"
//   else puts "C"
//   end
#[test]
fn runs_nested_if_else() {
    let source = indoc! {r#"
        점수는 85이다
        점수 >= 90이면
          "A"를 출력한다
        아니면
          점수 >= 80이면
            "B"를 출력한다
          아니면
            "C"를 출력한다
    "#};
    assert_output(source, &["B"]);
}

// Python: if score > 60: print("합격")
// Ruby:   puts "합격" if score > 60
//
// 지음 고유: 한국어 비교 프레임 "X가 Y보다 크면"
#[test]
fn runs_korean_comparison_greater() {
    let source = indoc! {r#"
        점수는 85이다
        점수가 60보다 크면
          "합격"을 출력한다
        아니면
          "불합격"을 출력한다
    "#};
    assert_output(source, &["합격"]);
}

// Python: x < y / x == y / x != y
// Ruby:   x < y / x == y / x != y
//
// 지음 고유: "X가 Y보다 작으면", "X가 Y와 같으면", "X가 Y과 다르면"
#[test]
fn runs_korean_comparison_all_predicates() {
    let source = indoc! {r#"
        온도는 36이다
        온도가 37보다 작으면
          "정상"을 출력한다

        숫자는 5이다
        숫자가 5와 같으면
          "일치"를 출력한다

        점수는 80이다
        점수가 100과 다르면
          "다름"을 출력한다
    "#};
    assert_output(source, &["정상", "일치", "다름"]);
}

// ---------------------------------------------------------------------------
// 반복문: 경계 조건
// ---------------------------------------------------------------------------

// Python: while False: print("never")  # body never executes
// Ruby:   while false do puts "never" end
#[test]
fn runs_while_zero_iterations() {
    let source = indoc! {r#"
        횟수는 0이다
        횟수 > 0인 동안
          "실행됨"을 출력한다
          횟수를 횟수 - 1로 바꾼다
        "끝"을 출력한다
    "#};
    assert_output(source, &["끝"]);
}

// Python:
//   for i in range(1, 3):
//       for j in range(1, 4):
//           print(j)
//
// Ruby:
//   (1..2).each { |i| (1..3).each { |j| puts j } }
#[test]
fn runs_nested_while_loops() {
    let source = indoc! {"
        행은 1이다
        열은 0이다
        행 <= 2인 동안
          열을 1로 바꾼다
          열 <= 3인 동안
            열을 출력한다
            열을 열 + 1로 바꾼다
          행을 행 + 1로 바꾼다
    "};
    assert_output(source, &["1", "2", "3", "1", "2", "3"]);
}
