# Parser Test Cases

이 문서는 v0.1 parser golden/snapshot 테스트 입력 세트 초안이다. 각 케이스는 입력과 핵심 assert를 포함한다. AST 표기는 `docs/LANGUAGE.md`의 실행 의미와 구현 구조를 따른다.

## 사용 원칙

- 각 케이스는 독립 파일 또는 fixture 하나로 옮길 수 있어야 한다.
- 성공 케이스는 AST snapshot을 고정한다.
- 실패 케이스는 오류 종류와 대략의 실패 위치를 고정한다.
- 새 문법을 추가할 때는 이 파일에 먼저 케이스를 추가하는 편이 좋다.

## 1. 성공 케이스

### P01. 문자열 바인딩

```txt
이름은 "철수"이다
```

기대:

- top-level `BindStmt`
- `name="이름"`
- `value=LiteralExpr("철수")`

### P02. 숫자 바인딩

```txt
나이는 20이다
```

기대:

- top-level `BindStmt`
- `value=LiteralExpr(20)`

### P03. 목록 바인딩

```txt
숫자들은 [1, 2, 3]이다
```

기대:

- `BindStmt`
- `ListExpr(items=[1, 2, 3])`

### P04. 레코드 바인딩

```txt
사람은 { 이름: "영희", 나이: 18 }이다
```

기대:

- `BindStmt`
- `RecordExpr(entries=[...])`

### P05. 재대입

```txt
점수를 점수 + 1로 바꾼다
```

기대:

- `AssignStmt`
- 우변이 `BinaryExpr(NameExpr("점수"), "+", LiteralExpr(1))`

### P06. 출력

```txt
이름을 출력한다
```

기대:

- `PrintStmt(NameExpr("이름"))`

### P07. 독립 호출 문장

```txt
추가(숫자들, 3)
```

기대:

- `ExprStmt`
- 내부가 `CallExpr(NameExpr("추가"), [...])`

### P08. 함수 호출 바인딩

```txt
합은 더하기(3, 5)이다
```

기대:

- `BindStmt`
- 우변이 `CallExpr(NameExpr("더하기"), [3, 5])`

### P08A. unary transform 호출 바인딩

```txt
문장은 "지음"으로 인사만들기이다
```

기대:

- `BindStmt`
- 우변이 `TransformCallExpr(StringExpr("지음"), "인사만들기")`

### P09. 인덱싱 출력

```txt
숫자들[0]을 출력한다
```

기대:

- `PrintStmt`
- 내부가 `IndexExpr(NameExpr("숫자들"), LiteralExpr(0))`

### P10. 속성 접근 출력

```txt
사람의 이름을 출력한다
```

기대:

- `PrintStmt`
- 내부가 `PropertyExpr(NameExpr("사람"), "이름")`

### P11. 중첩 속성 접근

```txt
사용자의 주소의 도시를 출력한다
```

기대:

- `PropertyExpr(PropertyExpr(NameExpr("사용자"), "주소"), "도시")`

### P12. 인덱싱 뒤 속성 접근

```txt
목록[0]의 이름을 출력한다
```

기대:

- `PropertyExpr(IndexExpr(NameExpr("목록"), LiteralExpr(0)), "이름")`

### P13. 괄호 뒤 조사

```txt
(마을)을 출력한다
```

기대:

- `PrintStmt(NameExpr("마을"))`
- 괄호 표현식 뒤 `OBJECT` 처리 성공

### P14. 단항 부정

```txt
참이아님은 아니다 참이다
```

기대:

- `BindStmt`
- 우변이 `UnaryExpr("아니다", LiteralExpr(true))`

비고:

- 식별자 `참이아님`은 단순 이름이다.

### P15. 산술 우선순위

```txt
값은 1 + 2 * 3이다
```

기대:

- 최상위가 `1 + (2 * 3)` 형태

### P16. 괄호 우선순위

```txt
값은 (1 + 2) * 3이다
```

기대:

- 최상위가 `(1 + 2) * 3` 형태

### P17. 비교식

```txt
성인은 나이 >= 20이다
```

기대:

- `BindStmt`
- 우변이 `BinaryExpr(NameExpr("나이"), ">=", LiteralExpr(20))`

### P17A. built-in noun message

```txt
결과는 5의 제곱이다
```

기대:

- `BindStmt`
- 우변이 `PropertyExpr(LiteralExpr(5), "제곱")`

### P18. 논리 연산 우선순위

```txt
로그인가능은 관리자 또는 편집자 그리고 활성이다
```

기대:

- `그리고`가 `또는`보다 먼저 묶인다.
- 형태는 `관리자 또는 (편집자 그리고 활성)`

### P19. `이면`만 있는 조건문

```txt
나이 > 19이면
  "성인"을 출력한다
```

기대:

- `IfStmt`
- `else_block=None`

### P20. `아니면`이 있는 조건문

```txt
나이 > 19이면
  "성인"을 출력한다
아니면
  "미성년자"를 출력한다
```

기대:

- `IfStmt`
- `then_block`와 `else_block` 모두 존재

### P21. 중첩 조건문

```txt
참이면
  거짓이면
    "a"를 출력한다
  아니면
    "b"를 출력한다
```

기대:

- 바깥 `IfStmt` 안에 안쪽 `IfStmt`

### P22. 반복문

```txt
인덱스 < 길이(숫자들)인 동안
  숫자들[인덱스]을 출력한다
  인덱스를 인덱스 + 1로 바꾼다
```

기대:

- `WhileStmt`
- 본문에 `PrintStmt`, `AssignStmt` 순서로 포함

### P23. 매개변수 있는 함수 정의

```txt
더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다
```

기대:

- `FunctionDefStmt(name="더하기", params=["왼쪽", "오른쪽"])`
- 본문 첫 문장이 `ReturnStmt`

### P24. 매개변수 없는 함수 정의

```txt
시작 함수는 아무것도 받지 않아
  "시작"을 출력한다
```

기대:

- `FunctionDefStmt(name="시작", params=[])`

### P25. 함수 내부 바인딩과 반환

```txt
두배 함수는 값을 받아
  결과는 값 * 2이다
  결과를 돌려준다
```

기대:

- 함수 본문에 `BindStmt` 다음 `ReturnStmt`

### P26. 함수 정의 뒤 호출

```txt
더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다

합은 더하기(1, 2)이다
```

기대:

- top-level 문장 2개
- 첫 문장은 `FunctionDefStmt`, 둘째는 `BindStmt`

### P27. 호출 뒤 인덱싱

```txt
첫값은 가져오기()[0]이다
```

기대:

- 우변이 `IndexExpr(CallExpr(NameExpr("가져오기"), []), LiteralExpr(0))`

### P28. 호출 뒤 속성 접근

```txt
이름은 사용자()의 이름이다
```

기대:

- 우변이 `PropertyExpr(CallExpr(NameExpr("사용자"), []), "이름")`

### P29. 레코드 안의 목록

```txt
설정은 { 이름: "앱", 포트들: [80, 443] }이다
```

기대:

- `RecordExpr` 내부에 `ListExpr`가 중첩된다

### P30. 여러 top-level 문장

```txt
이름은 "철수"이다
나이는 20이다
이름을 출력한다
```

기대:

- top-level 문장 3개
- 순서 유지

### P31. 문자열과 속성 접근 혼합

```txt
메시지는 사람의 이름 + "님"이다
```

기대:

- `BinaryExpr(PropertyExpr(NameExpr("사람"), "이름"), "+", LiteralExpr("님"))`

### P32. 중첩 괄호와 논리식

```txt
통과는 (점수 >= 70) 그리고 (결석 == 0)이다
```

기대:

- 우변이 최상위 `BinaryExpr(..., "그리고", ...)`

### P33. 결과 서술 effect statement

```txt
시작탑에서 맨위 원반을 꺼낸다
```

기대:

- `ParseSuccess`
- 현재 닫힌 effect frame `...에서 맨위 원반을 꺼낸다`를 문장으로 파싱

## 2. 실패 케이스

### N01. `이다` 누락

```txt
이름은 "철수"
```

기대:

- `ParseError`
- 바인딩 문장 끝에서 `COPULA` 기대

### N02. 잘못된 함수 헤더 순서

```txt
함수 더하기는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다
```

기대:

- `ParseError`
- 함수 정의는 `<이름> 함수는` 형식을 기대

### N03. `함수는` 조사 생략

```txt
더하기 함수 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다
```

기대:

- `ParseError`
- `FUNCTION_TOPIC("는")` 기대

### N04. 함수 헤더 목적격 누락

```txt
더하기 함수는 왼쪽, 오른쪽 받아
  왼쪽 + 오른쪽을 돌려준다
```

기대:

- `ParseError`
- 매개변수 목록 뒤 `OBJECT` 기대

### N05. `이면` 뒤 들여쓰기 없음

```txt
참이면
"a"를 출력한다
```

기대:

- `ParseError`
- `INDENT` 기대

### N06. 단독 `아니면`

```txt
아니면
  "a"를 출력한다
```

기대:

- `ParseError`
- 앞선 `IfStmt` 없이 `ELSE` 사용 불가

### N07. 잘못된 속성 접근

```txt
사람의 "이름"을 출력한다
```

기대:

- `ParseError`
- `GEN` 뒤에는 `IDENT` 기대

### N08. 재대입 방향 조사 누락

```txt
점수를 10 바꾼다
```

기대:

- `ParseError`
- `DIRECTION("로"|"으로")` 기대

### N09. 잘못된 무인수 함수 문장

```txt
시작 함수는 아무것도 받아
  "시작"을 출력한다
```

기대:

- `ParseError`
- `받지 않아` 조합 기대

### N10. 닫히지 않은 레코드

```txt
사람은 { 이름: "철수", 나이: 20이다
```

기대:

- `ParseError`
- `RBRACE` 기대

### N11. 잘못된 들여쓰기 폭

```txt
참이면
  "a"를 출력한다
    "b"를 출력한다
```

기대:

- `ParseError`
- 같은 블록 안에서 예기치 않은 추가 `INDENT`

### N12. `의` 뒤 식별자 없음

```txt
사람의 을 출력한다
```

기대:

- `ParseError`
- `GEN` 뒤 `IDENT` 기대

### N13. 닫힌 키워드 메시지 집합 밖 동사

```txt
과일들에 "감" 넣기
```

기대:

- `ParseError`
- 현재 키워드 메시지 집합 밖 동사는 거부

### N14. `추가` 앞의 잘못된 방향 조사

```txt
과일들에 "감" 으로 추가
```

기대:

- `ParseError`
- `추가`는 `<목록>에 <값> 추가` 형식만 허용

### N15. `그림판` 동작의 방향 조사 누락

```txt
그림판에 { x: 120, y: 80, 색: 빨강 } 점찍기
```

기대:

- `ParseError`
- `그림판에 <레코드>로/으로 <동작>` 형식 기대

### N16. 이름 붙은 호출의 비레코드 인수

```txt
탑옮기기를 원반수로 호출한다
```

기대:

- `ParseError`
- 이름 붙은 호출의 인수는 레코드여야 함

### N17. 바인딩 없는 결과 서술 식 문장

```txt
시작탑에서 맨위 원반을 꺼낸 것이다
```

기대:

- `ParseError`
- 독립 `...꺼낸 것이다`는 허용하지 않고 `...꺼낸다`만 문장으로 허용

## 3. 우선 구현 순서

처음부터 전부 구현하지 말고 아래 순서로 snapshot을 고정하는 편이 좋다.

1. `P01`-`P12`
2. `P19`-`P26`
3. `N01`-`N06`
4. 나머지 우선순위/혼합 케이스
