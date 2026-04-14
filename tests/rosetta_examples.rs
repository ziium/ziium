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

// ===========================================================================
// 배치 2: 기능 테스트 — 함수 / 문자열
// ===========================================================================

// ---------------------------------------------------------------------------
// 함수: 호출 체인, 조기 반환, 재귀, 조건 분기 반환
// ---------------------------------------------------------------------------

// Python:
//   def double(n): return n * 2
//   def quadruple(n): return double(double(n))
//   print(quadruple(3))  # 12
//
// Ruby:
//   def double(n) = n * 2
//   def quadruple(n) = double(double(n))
//   puts quadruple(3)  # 12
#[test]
fn runs_function_calling_function() {
    let source = indoc! {"
        두배 함수는 숫자를 받아
          숫자 * 2를 돌려준다

        네배 함수는 숫자를 받아
          두배(두배(숫자))를 돌려준다

        네배(3)을 출력한다
    "};
    assert_output(source, &["12"]);
}

// Python:
//   def abs(n):
//       if n < 0: return -n
//       return n
//   print(abs(-7))  # 7
//   print(abs(5))   # 5
//
// Ruby:
//   def abs(n) = n < 0 ? -n : n
#[test]
fn runs_early_return() {
    let source = indoc! {"
        절대값 함수는 숫자를 받아
          숫자 < 0이면
            -숫자를 돌려준다
          숫자를 돌려준다

        절대값(-7)을 출력한다
        절대값(5)를 출력한다
    "};
    assert_output(source, &["7", "5"]);
}

// Python:
//   def factorial(n):
//       if n <= 1: return 1
//       return n * factorial(n - 1)
//   print(factorial(5))  # 120
//
// Ruby:
//   def factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)
#[test]
fn runs_recursive_factorial() {
    let source = indoc! {"
        팩토리얼 함수는 숫자를 받아
          숫자 <= 1이면
            1을 돌려준다
          숫자 * 팩토리얼(숫자 - 1)을 돌려준다

        팩토리얼(5)를 출력한다
    "};
    assert_output(source, &["120"]);
}

// Python:
//   def grade(score):
//       if score >= 90: return "A"
//       if score >= 80: return "B"
//       return "C"
//   print(grade(95))  # A
//   print(grade(82))  # B
//   print(grade(70))  # C
//
// Ruby:
//   def grade(score)
//     return "A" if score >= 90
//     return "B" if score >= 80
//     "C"
//   end
#[test]
fn runs_function_with_branching_returns() {
    let source = indoc! {r#"
        등급 함수는 점수를 받아
          점수 >= 90이면
            "A"를 돌려준다
          점수 >= 80이면
            "B"를 돌려준다
          "C"를 돌려준다

        등급(95)를 출력한다
        등급(82)를 출력한다
        등급(70)을 출력한다
    "#};
    assert_output(source, &["A", "B", "C"]);
}

// Python:
//   def add(a, b): return a + b
//   def mul(a, b): return a * b
//   print(add(mul(2, 3), mul(4, 5)))  # 26
//
// Ruby:
//   def add(a, b) = a + b
//   def mul(a, b) = a * b
//   puts add(mul(2, 3), mul(4, 5))  # 26
#[test]
fn runs_multi_param_function_composition() {
    let source = indoc! {"
        더하기 함수는 왼쪽, 오른쪽을 받아
          왼쪽 + 오른쪽을 돌려준다

        곱 함수는 왼쪽, 오른쪽을 받아
          왼쪽 * 오른쪽을 돌려준다

        더하기(곱(2, 3), 곱(4, 5))를 출력한다
    "};
    assert_output(source, &["26"]);
}

// Python:
//   def fib(n):
//       if n <= 1: return n
//       return fib(n - 1) + fib(n - 2)
//   for i in range(8): print(fib(i))
//
// Ruby:
//   def fib(n) = n <= 1 ? n : fib(n-1) + fib(n-2)
//   8.times { |i| puts fib(i) }
#[test]
fn runs_fibonacci_recursive() {
    let source = indoc! {"
        피보나치 함수는 숫자를 받아
          숫자 <= 1이면
            숫자를 돌려준다
          피보나치(숫자 - 1) + 피보나치(숫자 - 2)를 돌려준다

        순서는 0이다
        순서 < 8인 동안
          피보나치(순서)를 출력한다
          순서를 순서 + 1로 바꾼다
    "};
    assert_output(source, &["0", "1", "1", "2", "3", "5", "8", "13"]);
}

// ---------------------------------------------------------------------------
// 문자열: 연결, 길이, 인덱싱, 비교
// ---------------------------------------------------------------------------

// Python: "Hello" + ", " + "World!"  # "Hello, World!"
// Ruby:   "Hello" + ", " + "World!"  # "Hello, World!"
#[test]
fn runs_string_concatenation_chain() {
    let source = indoc! {r#"
        인사는 "안녕" + ", " + "세상아!"이다
        인사를 출력한다
    "#};
    assert_output(source, &["안녕, 세상아!"]);
}

// Python: len("가나다")  # 3
// Ruby:   "가나다".length  # 3 (chars)
//
// 지음 고유: 소유격 프레임 "X의 길이"
#[test]
fn runs_string_length_unicode() {
    let source = indoc! {r#"
        단어는 "가나다라마"이다
        단어의 길이를 출력한다
    "#};
    assert_output(source, &["5"]);
}

// Python: "안녕하세요"[2]  # "하"
// Ruby:   "안녕하세요"[2]  # "하"
#[test]
fn runs_string_indexing() {
    let source = indoc! {r#"
        글자는 "안녕하세요"[2]이다
        글자를 출력한다
    "#};
    assert_output(source, &["하"]);
}

// Python: "abc" == "abc"  # True;  "abc" == "xyz"  # False
// Ruby:   "abc" == "abc"  # true;  "abc" == "xyz"  # false
#[test]
fn runs_string_equality() {
    let source = indoc! {r#"
        같음은 "지음" == "지음"이다
        같음을 출력한다
        다름은 "지음" != "다른말"이다
        다름을 출력한다
    "#};
    assert_output(source, &["참", "참"]);
}
