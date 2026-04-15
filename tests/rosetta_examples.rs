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

fn assert_error(source: &str, expected_fragment: &str) {
    let err = run_source(source).expect_err("program should produce an error");
    let message = err.to_string();
    assert!(
        message.contains(expected_fragment),
        "expected error containing {:?}, got: {:?}",
        expected_fragment,
        message,
    );
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
        횟수에 0을 넣는다
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
        행에 1을 넣는다
        열에 0을 넣는다
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

        순서에 0을 넣는다
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

// ===========================================================================
// 배치 3: 에러 경로 — 타입 오류 / 함수 인자 / 인덱스 / 빌트인 타입
// ===========================================================================

// ---------------------------------------------------------------------------
// 타입 오류: 산술 / 비교
// ---------------------------------------------------------------------------

// Python: "글자" + 3  → TypeError: can only concatenate str (not "int") to str
// Ruby:   "글자" + 3  → TypeError: no implicit conversion of Integer into String
//
// 지음: 숫자끼리 또는 문자열끼리만 더할 수 있음
#[test]
fn rejects_addition_of_string_and_number() {
    // "글자" + 3
    let source = indoc! {r#"
        결과는 "글자" + 3이다
    "#};
    assert_error(source, "숫자끼리 또는 문자열끼리만");
}

// Python: "가" - "나"  → TypeError: unsupported operand type(s) for -
// Ruby:   "가" - "나"  → NoMethodError: undefined method `-' for String
#[test]
fn rejects_subtraction_of_strings() {
    // "가" - "나"
    let source = indoc! {r#"
        결과는 "가" - "나"이다
    "#};
    assert_error(source, "숫자 연산은 정수 또는 실수에만");
}

// Python: 3.5 % 2.0  → 1.5 (허용)
// Ruby:   3.5 % 2.0  → 1.5 (허용)
//
// 지음 고유: `%`를 정수 전용으로 제한 — 부동소수점 나머지의 혼란을 방지하는 설계 결정
#[test]
fn rejects_modulo_on_floats() {
    // 3.5 % 2.0
    let source = indoc! {"
        결과는 3.5 % 2.0이다
    "};
    assert_error(source, "정수끼리만");
}

// Python: "가" > "나"  → True (유니코드 코드포인트 순 비교)
// Ruby:   "가" > "나"  → true (동일)
//
// 지음 고유: 비교 연산은 숫자에만 허용 — 문자열 정렬은 별도 기능으로 제공 예정
#[test]
fn rejects_comparison_on_non_numbers() {
    // "가" > "나"
    let source = indoc! {r#"
        결과는 "가" > "나"이다
    "#};
    assert_error(source, "비교 연산은 숫자에만");
}

// ---------------------------------------------------------------------------
// 함수에 넘긴 값의 개수 불일치
// ---------------------------------------------------------------------------

// Python: def add(a, b): ...;  add(1)  → TypeError: missing 1 required positional argument
// Ruby:   def add(a, b) ...; add(1) → ArgumentError: wrong number of arguments (given 1, expected 2)
#[test]
fn rejects_function_call_with_too_few_args() {
    // 더하기(왼쪽, 오른쪽) 에 값 1개만 전달
    let source = indoc! {"
        더하기 함수는 왼쪽, 오른쪽을 받아
          왼쪽 + 오른쪽을 돌려준다

        더하기(1)을 출력한다
    "};
    assert_error(source, "함수에 넘긴 값의 개수가 맞지 않습니다");
}

// Python: def add(a, b): ...; add(1, 2, 3)  → TypeError: takes 2 positional arguments but 3 were given
// Ruby:   def add(a, b) ...; add(1, 2, 3) → ArgumentError: wrong number of arguments (given 3, expected 2)
#[test]
fn rejects_function_call_with_too_many_args() {
    // 더하기(왼쪽, 오른쪽) 에 값 3개 전달
    let source = indoc! {"
        더하기 함수는 왼쪽, 오른쪽을 받아
          왼쪽 + 오른쪽을 돌려준다

        더하기(1, 2, 3)을 출력한다
    "};
    assert_error(source, "함수에 넘긴 값의 개수가 맞지 않습니다");
}

// ---------------------------------------------------------------------------
// 인덱스: 범위 초과 / 음수
// ---------------------------------------------------------------------------

// Python: [10, 20, 30][5]  → IndexError: list index out of range
// Ruby:   [10, 20, 30][5]  → nil (에러가 아닌 nil 반환 — 다른 설계 철학)
#[test]
fn rejects_list_index_out_of_bounds() {
    // [10, 20, 30][5]
    let source = indoc! {"
        목록은 [10, 20, 30]이다
        목록[5]를 출력한다
    "};
    assert_error(source, "목록 인덱스가 범위를 벗어났습니다");
}

// Python: "가나다"[10]  → IndexError: string index out of range
// Ruby:   "가나다"[10]  → nil
#[test]
fn rejects_string_index_out_of_bounds() {
    // "가나다"[10]
    let source = indoc! {r#"
        글자는 "가나다"[10]이다
    "#};
    assert_error(source, "문자열 인덱스가 범위를 벗어났습니다");
}

// Python: [10, 20, 30][-1]  → 30 (뒤에서부터 — 음수 인덱스 허용)
// Ruby:   [10, 20, 30][-1]  → 30 (동일)
//
// 지음 고유: 음수 인덱스를 허용하지 않음 — 의도하지 않은 역방향 접근 방지
#[test]
fn rejects_negative_index() {
    // 목록[-1]
    let source = indoc! {"
        목록은 [10, 20, 30]이다
        목록[-1]을 출력한다
    "};
    assert_error(source, "인덱스는 0 이상의 정수여야 합니다");
}

// ---------------------------------------------------------------------------
// 빌트인 타입 검증
// ---------------------------------------------------------------------------

// Python: len(3)  → TypeError: object of type 'int' has no len()
// Ruby:   3.length  → NoMethodError: undefined method `length' for Integer
#[test]
fn rejects_length_on_non_collection() {
    // 길이(3)
    let source = indoc! {"
        길이(3)을 출력한다
    "};
    assert_error(source, "목록, 문자열, 레코드에만");
}

// ===========================================================================
// 배치 4: 기능 테스트 — 목록 / 레코드 / 논리 연산 / 타입 변환
// ===========================================================================

// ---------------------------------------------------------------------------
// 목록: 생성·순회, 추가, 꺼내기, 동치, 혼합 타입
// ---------------------------------------------------------------------------

// Python:
//   fruits = ["사과", "바나나", "포도"]
//   for i in range(len(fruits)):
//       print(fruits[i])
//
// Ruby:
//   fruits = ["사과", "바나나", "포도"]
//   fruits.each { |f| puts f }
//
// 지음: 인덱싱 + `의 길이` 속성으로 while 순회
#[test]
fn runs_list_create_and_iterate() {
    let source = indoc! {r#"
        과일들은 ["사과", "바나나", "포도"]이다
        차례에 0을 넣는다
        차례 < 과일들의 길이인 동안
          과일들[차례]를 출력한다
          차례를 차례 + 1로 바꾼다
    "#};
    assert_output(source, &["사과", "바나나", "포도"]);
}

// Python: scores = [90, 85]; scores.append(100); print(len(scores))
// Ruby:   scores = [90, 85]; scores.push(100); puts scores.length
//
// 지음 고유: 키워드 메시지 — `목록에 값 추가`
#[test]
fn runs_list_append_keyword_message() {
    let source = indoc! {"
        점수들은 [90, 85]이다
        점수들에 100 추가
        점수들의 길이를 출력한다
        점수들[2]을 출력한다
    "};
    assert_output(source, &["3", "100"]);
}

// Python: cards = [1, 2, 3]; drawn = cards.pop(); print(drawn, len(cards))
// Ruby:   cards = [1, 2, 3]; drawn = cards.pop; puts drawn, cards.length
//
// 지음 고유: 결과서술 바인딩 — `목록에서 맨위 요소를 꺼낸 것이다`
#[test]
fn runs_list_pop_resultive_binding() {
    let source = indoc! {"
        카드는 [1, 2, 3]이다
        뽑은것은 카드에서 맨위 요소를 꺼낸 것이다
        뽑은것을 출력한다
        카드의 길이를 출력한다
    "};
    assert_output(source, &["3", "2"]);
}

// Python: [1, 2, 3] == [1, 2, 3]  # True;  [1, 2, 3] == [1, 2, 4]  # False
// Ruby:   [1, 2, 3] == [1, 2, 3]  # true;  [1, 2, 3] == [1, 2, 4]  # false
#[test]
fn runs_list_equality() {
    let source = indoc! {"
        왼쪽은 [1, 2, 3]이다
        오른쪽은 [1, 2, 3]이다
        같은지는 왼쪽 == 오른쪽이다
        같은지를 출력한다
        다른것은 [1, 2, 4]이다
        다른지는 왼쪽 == 다른것이다
        다른지를 출력한다
    "};
    assert_output(source, &["참", "거짓"]);
}

// Python: items = [1, "둘", True]; print(len(items))
// Ruby:   items = [1, "둘", true]; puts items.length
#[test]
fn runs_list_heterogeneous_types() {
    let source = indoc! {r#"
        모음은 [42, "안녕", 참]이다
        모음의 길이를 출력한다
        모음[0]을 출력한다
        모음[1]을 출력한다
        모음[2]을 출력한다
    "#};
    assert_output(source, &["3", "42", "안녕", "참"]);
}

// ---------------------------------------------------------------------------
// 레코드: 생성·접근, 축약, 중첩 체이닝
// ---------------------------------------------------------------------------

// Python: person = {"이름": "영희", "나이": 25}; print(person["이름"])
// Ruby:   person = {이름: "영희", 나이: 25}; puts person[:이름]
//
// 지음 고유: 소유격 프레임 — `레코드의 필드`
#[test]
fn runs_record_create_and_property_access() {
    let source = indoc! {r#"
        사람은 { 이름: "영희", 나이: 25 }이다
        사람의 이름을 출력한다
        사람의 나이를 출력한다
    "#};
    assert_output(source, &["영희", "25"]);
}

// Python: N/A (no shorthand for dict)
// Ruby:   N/A (symbol shorthand is different)
//
// 지음 고유: 레코드 축약 — 변수명이 키 이름과 같으면 값 생략
#[test]
fn runs_record_shorthand_syntax() {
    let source = indoc! {r#"
        이름은 "민수"이다
        학생은 { 이름, 나이: 18 }이다
        학생의 이름을 출력한다
        학생의 나이를 출력한다
    "#};
    assert_output(source, &["민수", "18"]);
}

// Python: addr = {"도시": "서울"}
//         person = {"이름": "영희", "주소": addr}
//         print(person["주소"]["도시"])
//
// Ruby:   person[:주소][:도시]
//
// 지음 고유: 소유격 체이닝 — `레코드의 필드의 필드`
#[test]
fn runs_nested_record_property_chain() {
    let source = indoc! {r#"
        주소는 { 도시: "서울", 동네: "강남" }이다
        사람은 { 이름: "영희", 주소 }이다
        사람의 주소의 도시를 출력한다
        사람의 주소의 동네를 출력한다
    "#};
    assert_output(source, &["서울", "강남"]);
}

// ---------------------------------------------------------------------------
// 논리 연산: 그리고 / 또는 / 아니다
// ---------------------------------------------------------------------------

// Python: True and False  # False;  True or False  # True;  not True  # False
// Ruby:   true && false   # false;  true || false  # true;  !true     # false
#[test]
fn runs_logical_operators() {
    let source = indoc! {"
        결과1은 참 그리고 거짓이다
        결과1을 출력한다
        결과2는 참 또는 거짓이다
        결과2를 출력한다
        결과3은 아니다 참이다
        결과3을 출력한다
    "};
    assert_output(source, &["거짓", "참", "거짓"]);
}

// Python: if temp > 0 and temp < 40: print("적정")
// Ruby:   puts "적정" if temp > 0 && temp < 40
#[test]
fn runs_compound_boolean_in_condition() {
    let source = indoc! {r#"
        온도는 25이다
        온도 > 0 그리고 온도 < 40이면
          "적정"을 출력한다
        아니면
          "이상"을 출력한다
    "#};
    assert_output(source, &["적정"]);
}

// ---------------------------------------------------------------------------
// 타입 변환: 문자열로 / 정수로 / 실수로
// ---------------------------------------------------------------------------

// Python: 5 ** 2  # 25;  3.0 ** 2  # 9.0
// Ruby:   5 ** 2  # 25;  3.0 ** 2  # 9.0
//
// 지음 고유: 소유격 프레임 `의 제곱` — 속성 메시지로 거듭제곱 표현
#[test]
fn runs_square_property() {
    let source = indoc! {"
        정수제곱은 5의 제곱이다
        정수제곱을 출력한다
        실수제곱은 3.0의 제곱이다
        실수제곱을 출력한다
    "};
    assert_output(source, &["25", "9.0"]);
}

// ---------------------------------------------------------------------------
// 실수: 정수/실수 혼합 산술
// ---------------------------------------------------------------------------

// Python: 3 + 0.14  # 3.14 (int auto-promoted to float)
// Ruby:   3 + 0.14  # 3.14
#[test]
fn runs_float_int_mixed_arithmetic() {
    let source = indoc! {"
        원주율은 3 + 0.14이다
        원주율을 출력한다
        넓이는 2 * 3.5이다
        넓이를 출력한다
    "};
    assert_output(source, &["3.14", "7.0"]);
}

// ---------------------------------------------------------------------------
// 변환 호출: `값으로 함수이름` 구문
// ---------------------------------------------------------------------------

// Python: double = lambda n: n * 2;  double(5)  # 10
// Ruby:   def double(n) = n * 2;  double(5)  # 10
//
// 지음 고유: `값으로 함수이름` — 한 값을 변환하는 자연어 호출
#[test]
fn runs_transform_call_syntax() {
    let source = indoc! {"
        두배 함수는 숫자를 받아
          숫자 * 2를 돌려준다

        결과는 5로 두배이다
        결과를 출력한다
    "};
    assert_output(source, &["10"]);
}

// ===========================================================================
// 배치 5: 조합 + 탐색 — Rosetta Code
// ===========================================================================

// ---------------------------------------------------------------------------
// 조합: 여러 기능을 결합한 Rosetta 과제
// ---------------------------------------------------------------------------

// Rosetta: FizzBuzz
// Python: for i in range(1,16): print("FizzBuzz" if i%15==0 ...)
// Ruby:   (1..15).each { |i| puts ... }
//
// 반복 + 나머지 + 중첩 조건 + 문자열 출력
#[test]
fn runs_fizzbuzz() {
    let source = indoc! {r#"
        숫자에 1을 넣는다
        숫자 <= 15인 동안
          숫자 % 15 == 0이면
            "FizzBuzz"를 출력한다
          아니면
            숫자 % 3 == 0이면
              "Fizz"를 출력한다
            아니면
              숫자 % 5 == 0이면
                "Buzz"를 출력한다
              아니면
                숫자를 출력한다
          숫자를 숫자 + 1로 바꾼다
    "#};
    assert_output(
        source,
        &[
            "1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz", "11", "Fizz", "13",
            "14", "FizzBuzz",
        ],
    );
}

// Rosetta: Greatest common divisor — Euclidean algorithm
// Python: def gcd(a,b): return a if b==0 else gcd(b, a%b)
// Ruby:   def gcd(a,b) = b==0 ? a : gcd(b, a%b)
//
// 재귀 + 나머지 + 조건 + 다인자 함수
#[test]
fn runs_euclidean_gcd() {
    let source = indoc! {"
        최대공약수 함수는 큰수, 작은수를 받아
          작은수 == 0이면
            큰수를 돌려준다
          최대공약수(작은수, 큰수 % 작은수)를 돌려준다

        최대공약수(48, 18)을 출력한다
        최대공약수(100, 75)를 출력한다
    "};
    assert_output(source, &["6", "25"]);
}

// Rosetta: Collatz conjecture — sequence length
// Python: def collatz(n): steps=0; while n!=1: n=n//2 if n%2==0 else 3*n+1; steps+=1
// Ruby:   def collatz(n) steps=0; while n!=1 do n=n.even? ? n/2 : 3*n+1; steps+=1 end; steps end
//
// while + 나머지 + 조건 + 함수 + 카운터
#[test]
fn runs_collatz_sequence_length() {
    let source = indoc! {"
        콜라츠 함수는 시작값을 받아
          횟수에 0을 넣는다
          현재에 시작값을 넣는다
          현재 != 1인 동안
            현재 % 2 == 0이면
              현재를 현재 / 2로 바꾼다
            아니면
              현재를 현재 * 3 + 1로 바꾼다
            횟수를 횟수 + 1로 바꾼다
          횟수를 돌려준다

        콜라츠(27)를 출력한다
        콜라츠(1)을 출력한다
    "};
    assert_output(source, &["111", "0"]);
}

// Rosetta: Palindrome detection
// Python: def is_palindrome(s): return s == s[::-1]
// Ruby:   def palindrome?(s) = s == s.reverse
//
// 문자열 인덱싱 + 길이 + 반복 + 비교 + 함수
#[test]
fn runs_palindrome_check() {
    let source = indoc! {r#"
        회문검사 함수는 단어를 받아
          왼쪽에 0을 넣는다
          오른쪽에 단어의 길이 - 1을 넣는다
          왼쪽 < 오른쪽인 동안
            단어[왼쪽] != 단어[오른쪽]이면
              거짓을 돌려준다
            왼쪽을 왼쪽 + 1로 바꾼다
            오른쪽을 오른쪽 - 1로 바꾼다
          참을 돌려준다

        회문검사("토마토")를 출력한다
        회문검사("사과")를 출력한다
        회문검사("기러기")를 출력한다
    "#};
    assert_output(source, &["참", "거짓", "참"]);
}

// Rosetta: Binary search
// Python: bisect.bisect_left / manual implementation
// Ruby:   bsearch
//
// 목록 + 반복 + 비교 + 함수 + 사전 선언 후 재대입
#[test]
fn runs_binary_search() {
    let source = indoc! {"
        이진탐색 함수는 목록, 찾는값을 받아
          왼쪽에 0을 넣는다
          오른쪽에 목록의 길이 - 1을 넣는다
          중간에 0을 넣는다
          왼쪽 <= 오른쪽인 동안
            중간을 (왼쪽 + 오른쪽) / 2로 바꾼다
            목록[중간] == 찾는값이면
              중간을 돌려준다
            목록[중간] < 찾는값이면
              왼쪽을 중간 + 1로 바꾼다
            아니면
              오른쪽을 중간 - 1로 바꾼다
          -1을 돌려준다

        자료는 [2, 5, 8, 12, 16, 23, 38, 56, 72, 91]이다
        이진탐색(자료, 23)을 출력한다
        이진탐색(자료, 99)를 출력한다
    "};
    assert_output(source, &["5", "-1"]);
}

// Rosetta: Classify numbers as perfect, abundant, or deficient
// Python: def classify(n): s=sum(i for i in range(1,n) if n%i==0); ...
// Ruby:   def classify(n) s=(1...n).select{|i|n%i==0}.sum; ... end
//
// 함수 + 나머지 + 반복 + 조건 캐스케이드 + 문자열 반환
#[test]
fn runs_number_classifier() {
    let source = indoc! {r#"
        분류 함수는 숫자를 받아
          약수합에 0을 넣는다
          나눔수에 1을 넣는다
          나눔수 < 숫자인 동안
            숫자 % 나눔수 == 0이면
              약수합을 약수합 + 나눔수로 바꾼다
            나눔수를 나눔수 + 1로 바꾼다
          약수합 == 숫자이면
            "완전수"를 돌려준다
          약수합 > 숫자이면
            "과잉수"를 돌려준다
          "부족수"를 돌려준다

        분류(6)를 출력한다
        분류(12)를 출력한다
        분류(7)을 출력한다
    "#};
    assert_output(source, &["완전수", "과잉수", "부족수"]);
}

// Rosetta: Towers of Hanoi
// Python: def hanoi(n, src, dst, aux): ...
// Ruby:   def hanoi(n, src, dst, aux) ... end
//
// 지음 고유: `호출한다` 이름 붙은 호출 + 재귀 + 레코드 축약
#[test]
fn runs_towers_of_hanoi() {
    let source = indoc! {r#"
        탑옮기기 함수는 원반수, 출발, 도착, 경유를 받아
          원반수 == 1이면
            출발을 출력한다
            도착을 출력한다
            0을 돌려준다
          탑옮기기를 { 원반수: 원반수 - 1, 출발, 도착: 경유, 경유: 도착 }로 호출한다
          출발을 출력한다
          도착을 출력한다
          탑옮기기를 { 원반수: 원반수 - 1, 출발: 경유, 도착, 경유: 출발 }로 호출한다
          0을 돌려준다

        탑옮기기(3, "A", "C", "B")를 출력한다
    "#};
    // 3개 원반: 7회 이동 (각 이동은 출발+도착 두 줄) + 최종 반환값 0
    assert_output(
        source,
        &["A", "C", "A", "B", "C", "B", "A", "C", "B", "A", "B", "C", "A", "C", "0"],
    );
}

// Rosetta: Exponentiation — iterative
// Python: def power(b, e): r=1; for _ in range(e): r*=b; return r
// Ruby:   def power(b, e) r=1; e.times{r*=b}; r end
//
// 반복 + 누적 + 함수
#[test]
fn runs_iterative_power() {
    let source = indoc! {"
        거듭제곱 함수는 밑, 지수를 받아
          결과에 1을 넣는다
          횟수에 0을 넣는다
          횟수 < 지수인 동안
            결과를 결과 * 밑으로 바꾼다
            횟수를 횟수 + 1로 바꾼다
          결과를 돌려준다

        거듭제곱(2, 10)을 출력한다
        거듭제곱(3, 4)를 출력한다
    "};
    assert_output(source, &["1024", "81"]);
}

// ---------------------------------------------------------------------------
// 탐색: 미구현 기능이 필요한 Rosetta 과제 (#[ignore])
// ---------------------------------------------------------------------------

// Rosetta: Bubble sort
// 필요 기능: 목록 인덱스 대입 — `목록[i]를 값으로 바꾼다`
// 현재 지음은 목록 원소의 인덱스 기반 재대입을 지원하지 않음
#[test]
fn explore_bubble_sort() {
    let source = indoc! {"
        숫자들은 [5, 3, 8, 1, 9, 2]이다
        크기는 숫자들의 길이이다
        바깥에 0을 넣는다
        바깥 < 크기 - 1인 동안
          안쪽에 0을 넣는다
          안쪽 < 크기 - 1 - 바깥인 동안
            숫자들[안쪽] > 숫자들[안쪽 + 1]이면
              임시는 숫자들[안쪽]이다
              숫자들[안쪽]을 숫자들[안쪽 + 1]로 바꾼다
              숫자들[안쪽 + 1]을 임시로 바꾼다
            안쪽을 안쪽 + 1로 바꾼다
          바깥을 바깥 + 1로 바꾼다
        숫자들[0]을 출력한다
        숫자들[5]를 출력한다
    "};
    assert_output(source, &["1", "9"]);
}

// Rosetta: For-each sum
#[test]
fn explore_foreach_sum() {
    let source = indoc! {"
        숫자들은 [10, 20, 30, 40]이다
        합계에 0을 넣는다
        숫자들의 각각 항목에 대해
          합계를 합계 + 항목으로 바꾼다
        합계를 출력한다
    "};
    assert_output(source, &["100"]);
}
