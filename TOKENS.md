# Tokens

이 문서는 v0.1 lexer가 parser에 넘겨야 하는 정규화 토큰 스트림을 정의한다. 구현은 순수 lexer 하나로 해도 되고, raw token scan 뒤에 particle split normalizer를 둬도 된다. 중요한 것은 parser가 이 문서의 토큰 결과를 보게 하는 것이다.

## 1. 처리 단계

권장 단계는 아래와 같다.

1. source text를 줄 단위로 읽는다.
2. raw token을 스캔한다.
3. 붙은 조사와 `함수는` 같은 결합형을 정규화한다.
4. 줄바꿈과 들여쓰기를 기준으로 `NEWLINE`, `INDENT`, `DEDENT`를 만든다.
5. `EOF`를 붙여 parser에 넘긴다.

`LANGUAGE_SPEC.md`와 `GRAMMAR.ebnf`는 정규화된 토큰 스트림을 기준으로 읽어야 한다.

## 2. 공백과 주석

- 공백 문자 중 스페이스는 허용한다.
- 탭은 금지한다.
- `#`부터 줄 끝까지는 주석이다.
- 빈 줄과 주석 전용 줄은 `INDENT`/`DEDENT` 계산에 영향을 주지 않는다.
- 괄호 `()`, 대괄호 `[]`, 중괄호 `{}` 내부 줄바꿈은 기본적으로 `NEWLINE`을 만들지 않는다.

## 3. 토큰 종류

### 3.1 구조 토큰

- `NEWLINE`
- `INDENT`
- `DEDENT`
- `EOF`

### 3.2 식별자와 리터럴

- `IDENT(text)`
- `INT(value)`
- `FLOAT(value)`
- `STRING(value)`

### 3.3 구두점과 연산자

- `LPAREN` `(`
- `RPAREN` `)`
- `LBRACKET` `[`
- `RBRACKET` `]`
- `LBRACE` `{`
- `RBRACE` `}`
- `COMMA` `,`
- `COLON` `:`
- `PERIOD` `.`
- `PLUS` `+`
- `MINUS` `-`
- `STAR` `*`
- `SLASH` `/`
- `PERCENT` `%`
- `EQ` `==`
- `NE` `!=`
- `LT` `<`
- `LE` `<=`
- `GT` `>`
- `GE` `>=`

### 3.4 예약어

- `FUNCTION` `함수`
- `FUNCTION_TOPIC` `는`
- `COPULA` `이다`
- `IF` `이면`
- `ELSE` `아니면`
- `IN` `인`
- `DURING` `동안`
- `RECEIVE` `받아`
- `NOTHING` `아무것도`
- `RECEIVE_NOT` `받지`
- `RECEIVE_NEG` `않아`
- `PRINT` `출력한다`
- `RETURN` `돌려준다`
- `CHANGE` `바꾼다`
- `TRUE` `참`
- `FALSE` `거짓`
- `NONE` `없음`
- `AND` `그리고`
- `OR` `또는`
- `NOT` `아니다`

### 3.5 조사 토큰

- `TOPIC` `은` 또는 `는`
- `SUBJECT` `이` 또는 `가`
- `OBJECT` `을` 또는 `를`
- `GEN` `의`
- `DIRECTION` `로` 또는 `으로`

`SUBJECT`는 v0.1 parser에서 아직 핵심 문법으로 쓰지 않지만, lexer 단계에서는 예약해 둔다.

### 3.6 선택적 문장 종결 기호

- `PERIOD` `.`

`PERIOD`는 lexer가 그대로 넘기되, parser는 완료된 문장 뒤에서만 선택적으로 소비한다. 블록을 여는 줄의 끝에서는 허용하지 않는다.

## 4. 식별자 규칙

식별자는 기본적으로 `LANGUAGE_SPEC.md`의 규칙을 따른다.

- 첫 글자: 한글 음절, 영문자, `_`
- 이후 글자: 한글 음절, 영문자, 숫자, `_`

raw scan 단계에서는 긴 식별자 후보를 먼저 읽은 뒤, 필요하면 뒤쪽 조사를 분리해 정규화한다.

## 5. 조사 정규화 규칙

### 5.1 기본 원칙

v0.1에서는 아래 형태를 같은 의미로 본다.

```txt
이름은 "철수"이다
이름 은 "철수"이다
이름은 "철수"이다.
```

정규화 후 parser는 둘 다 아래와 같은 토큰 흐름을 받는다.

```txt
IDENT("이름"), TOPIC("은"), STRING("철수"), COPULA("이다"), [ PERIOD(".") ]
```

### 5.2 붙은 조사 분리 대상

아래 뒤에는 공백 없이 조사가 붙을 수 있다.

- `IDENT`
- `INT`
- `FLOAT`
- `STRING`
- `RPAREN`
- `RBRACKET`
- `RBRACE`

예:

- `이름은`
- `3으로`
- `"안녕"을`
- `길이(목록)을`
- `숫자들[0]을`
- `{ 이름: "철수" }의`

### 5.3 지원하는 조사 부착

정규화는 아래 조사만 다룬다.

- `은`
- `는`
- `이`
- `가`
- `을`
- `를`
- `의`
- `로`
- `으로`

`에게`, `에서`, `까지`, `부터` 같은 조사는 v0.1 구조 토큰이 아니다.

### 5.4 longest-match 규칙

조사 후보가 둘 이상이면 더 긴 쪽을 먼저 본다.

예:

- `집으로` -> `IDENT("집"), DIRECTION("으로")`
- `길이로` -> `IDENT("길이"), DIRECTION("로")`

### 5.5 `함수는` 규칙

`함수는`은 아래로 정규화한다.

```txt
FUNCTION("함수"), FUNCTION_TOPIC("는")
```

v0.1 함수 정의 헤더는 `더하기 함수는 ...`만 허용한다. 따라서 `함수은`은 잘못된 입력이다.

### 5.6 모호성 처리

한국어에서는 식별자 자체가 `은/는/이/가/을/를/의/로`로 끝날 수 있다. 예를 들어 `마을`, `바늘`, `고리` 같은 이름은 순수 lexer 규칙만으로는 구조 조사와 충돌할 수 있다.

권장 구현 전략:

1. raw scan으로 긴 식별자 후보를 읽는다.
2. 조사 분리 가능한 suffix 후보를 만든다.
3. 현재 구문 위치에서 구조 조사가 필요한 경우에만 분리한다.
4. 그렇지 않으면 원래 `IDENT`를 유지한다.

즉, 정규화는 순수 문자 스캔만이 아니라 "parser가 기대하는 형태"를 반영한 post-lex normalization이어도 된다.

### 5.7 애매한 경우의 문서 스타일

애매한 식별자를 강제로 분명히 하고 싶으면 아래 형식을 권장한다.

```txt
(마을)을 출력한다
(바늘)을 출력한다
```

이 형식은 `IDENT + OBJECT`를 더 안정적으로 분리하게 해 준다.

## 6. 예시 토큰화

아래 예시는 구현 테스트에 그대로 쓸 수 있다.

### T01

입력:

```txt
이름은 "철수"이다
```

토큰:

```txt
IDENT("이름") TOPIC("은") STRING("철수") COPULA("이다") NEWLINE EOF
```

### T02

입력:

```txt
점수를 점수 + 1로 바꾼다
```

토큰:

```txt
IDENT("점수") OBJECT("를") IDENT("점수") PLUS INT(1) DIRECTION("로") CHANGE("바꾼다") NEWLINE EOF
```

### T03

입력:

```txt
사람의 이름을 출력한다
```

토큰:

```txt
IDENT("사람") GEN("의") IDENT("이름") OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

### T04

입력:

```txt
길이(목록)을 출력한다
```

토큰:

```txt
IDENT("길이") LPAREN IDENT("목록") RPAREN OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

### T05

입력:

```txt
"안녕"을 출력한다
```

토큰:

```txt
STRING("안녕") OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

### T06

입력:

```txt
3으로 바꾼다
```

토큰:

```txt
INT(3) DIRECTION("으로") CHANGE("바꾼다") NEWLINE EOF
```

### T07

입력:

```txt
숫자들[0]을 출력한다
```

토큰:

```txt
IDENT("숫자들") LBRACKET INT(0) RBRACKET OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

### T08

입력:

```txt
사용자의 주소의 도시를 출력한다
```

토큰:

```txt
IDENT("사용자") GEN("의") IDENT("주소") GEN("의") IDENT("도시") OBJECT("를") PRINT("출력한다") NEWLINE EOF
```

### T09

입력:

```txt
더하기 함수는 왼쪽, 오른쪽을 받아
```

토큰:

```txt
IDENT("더하기") FUNCTION("함수") FUNCTION_TOPIC("는") IDENT("왼쪽") COMMA IDENT("오른쪽") OBJECT("을") RECEIVE("받아") NEWLINE EOF
```

### T10

입력:

```txt
시작 함수는 아무것도 받지 않아
```

토큰:

```txt
IDENT("시작") FUNCTION("함수") FUNCTION_TOPIC("는") NOTHING("아무것도") RECEIVE_NOT("받지") RECEIVE_NEG("않아") NEWLINE EOF
```

### T11

입력:

```txt
나이 >= 20이면
```

토큰:

```txt
IDENT("나이") GE INT(20) IF("이면") NEWLINE EOF
```

### T12

입력:

```txt
참이면
  "성인"을 출력한다
```

토큰:

```txt
TRUE("참") IF("이면") NEWLINE INDENT STRING("성인") OBJECT("을") PRINT("출력한다") NEWLINE DEDENT EOF
```

### T13

입력:

```txt
아니면
```

토큰:

```txt
ELSE("아니면") NEWLINE EOF
```

### T14

입력:

```txt
인덱스 < 길이(숫자들)인 동안
```

토큰:

```txt
IDENT("인덱스") LT IDENT("길이") LPAREN IDENT("숫자들") RPAREN IN("인") DURING("동안") NEWLINE EOF
```

### T15

입력:

```txt
로그인가능은 관리자 또는 편집자이다
```

토큰:

```txt
IDENT("로그인가능") TOPIC("은") IDENT("관리자") OR("또는") IDENT("편집자") COPULA("이다") NEWLINE EOF
```

### T16

입력:

```txt
마을을 출력한다
```

정규화 기대:

```txt
IDENT("마을") OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

비고:

- 이 케이스는 모호성 처리 테스트다.

### T17

입력:

```txt
(마을)을 출력한다
```

토큰:

```txt
LPAREN IDENT("마을") RPAREN OBJECT("을") PRINT("출력한다") NEWLINE EOF
```

### T18

입력:

```txt
추가(숫자들, 3)
```

토큰:

```txt
IDENT("추가") LPAREN IDENT("숫자들") COMMA INT(3) RPAREN NEWLINE EOF
```

## 7. 들여쓰기 규칙

- 들여쓰기는 줄 시작의 스페이스 수로 계산한다.
- 같은 블록 안에서는 같은 깊이를 유지해야 한다.
- 첫 줄보다 깊어지면 `INDENT`
- 이전 블록보다 얕아지면 필요한 수만큼 `DEDENT`
- 파일 끝에서 열린 들여쓰기는 모두 `DEDENT`로 닫는다.

예:

```txt
참이면
  "a"를 출력한다
  "b"를 출력한다
"끝"을 출력한다
```

정규화 결과:

```txt
TRUE IF NEWLINE
INDENT STRING OBJECT PRINT NEWLINE
STRING OBJECT PRINT NEWLINE
DEDENT
STRING OBJECT PRINT NEWLINE
EOF
```

## 8. v0.1 바깥 규칙

아래는 lexer가 받아도 parser나 명세가 아직 처리하지 않는다.

- `SUBJECT`를 사용하는 주어 문장
- `에게`, `에서` 같은 추가 조사
- 복합 조사 `에게는`, `으로부터`
- escape sequence가 풍부한 문자열 문법

이 항목들은 v0.2 이상 논의 대상으로 남겨 둔다.
