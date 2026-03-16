# Architecture

이 문서는 언어 구현의 권장 구조를 설명한다. 아직 코드가 없더라도 어떤 순서와 경계로 시스템을 나눌지 먼저 고정해야 구현이 흔들리지 않는다.

## 목표

- parser와 interpreter를 먼저 완성할 수 있는 최소 구조를 잡는다.
- 문서와 테스트가 구현 구조를 자연스럽게 지지하도록 만든다.
- 향후 type checker, formatter, LSP를 붙일 수 있는 모양을 유지한다.

## 전체 파이프라인

1. 소스 읽기
2. lexer
3. parser
4. AST 생성
5. 이름 해석
6. 인터프리터 실행
7. 진단 출력

확장 파이프라인:

1. AST
2. HIR lowering
3. 타입 검사
4. 최적화용 IR
5. 코드 생성 또는 VM 실행

## 권장 모듈

### `source`

- 파일 읽기
- 줄/열 계산
- span 관리

### `lexer`

- UTF-8 텍스트 순회
- 식별자/예약어/리터럴 인식
- 붙은 조사 분리
- 들여쓰기 토큰 생성

핵심 난점:

- `사람의` 같은 형태를 `IDENT + GEN`으로 분리
- 숫자 뒤 조사 처리
- 줄바꿈과 괄호 내부 줄바꿈 구분

### `parser`

- 문장 파싱
- 표현식 우선순위 파싱
- 블록 구조 파싱
- AST 생성

권장 방식:

- 문장 파서는 recursive descent
- 표현식은 precedence climbing 또는 Pratt parser

### `ast`

- 노드 정의
- span 포함
- pretty print 또는 debug dump 지원

### `resolve`

- 스코프 테이블
- 이름 중복 검출
- 미정의 이름 검출

### `interp`

- 값 표현
- 환경 체인
- 함수 호출 프레임
- 내장 함수
- 런타임 오류

### `diagnostics`

- 에러 구조체
- span 기반 위치 출력
- 한국어 메시지 포맷
- 수정 제안 생성

### `tests`

- lexer golden test
- parser snapshot test
- interpreter 실행 테스트
- 오류 메시지 테스트

## 데이터 흐름

```txt
source
  -> tokens
  -> AST
  -> resolved AST or runtime-ready form
  -> runtime values / diagnostics
```

## 권장 디렉터리 구조

```txt
/src
  /source
  /lexer
  /parser
  /ast
  /resolve
  /interp
  /diagnostics
  /builtins
  /cli
  /tests
```

초기 구현이 더 작다면 아래처럼 시작해도 된다.

```txt
/src
  lexer.rs
  parser.rs
  ast.rs
  interp.rs
  diagnostics.rs
  main.rs
```

## 핵심 설계 원칙

### 1. lexer에서 조사 역할을 최대한 정리한다

parser가 표면 문자열을 직접 다시 분해하지 않도록 한다. `은/는`, `을/를`, `의`, `로/으로`는 토큰 단계에서 드러나는 편이 안정적이다.

### 2. parser는 의미 노드를 만든다

AST는 "한국어 조사가 붙은 문자열"이 아니라 "바인딩, 재대입, 조건문" 같은 의미 노드로 구성되어야 한다.

### 3. diagnostics는 처음부터 분리한다

언어 경험의 큰 부분이 오류 메시지다. 인터프리터 내부에서 문자열을 즉석 조합하지 말고 구조화된 진단을 통해 출력한다.

### 4. 문서와 테스트를 구조의 일부로 취급한다

새 문법 추가 시 최소 아래 세 가지를 함께 바꾸는 것을 원칙으로 한다.

- `LANGUAGE_SPEC.md`
- `GRAMMAR.ebnf`
- parser/interpreter test

## `의` 처리 전략

`의`는 v0.1 핵심 문법이다. 구현에서는 아래 단계를 추천한다.

1. lexer가 `사람의`를 `IDENT("사람") GEN("의")`로 분리한다.
2. parser가 후위 연산자로 `PropertyExpr`를 만든다.
3. interpreter가 레코드 키 조회로 실행한다.
4. 진단 시스템이 누락 키 오류를 한국어로 보고한다.

## 런타임 모델

런타임 값 예시:

```txt
Value =
  Int
  Float
  Bool
  String
  None
  List(Vec<Value>)
  Record(Map<String, Value>)
  Function(UserFunction | BuiltinFunction)
```

환경 모델:

```txt
Environment {
  values: Map<String, Value>
  parent: Option<Environment>
}
```

## 확장 지점

v0.2 이후 아래를 끼워 넣을 수 있게 인터페이스를 나누는 것이 좋다.

- type checker
- formatter
- REPL
- LSP
- bytecode or LLVM backend

## 구현 우선순위

1. source/span
2. lexer
3. parser
4. AST snapshot test
5. interpreter
6. diagnostics polish
7. CLI or REPL

이 순서를 바꾸면 앞단 명세 검증 없이 뒤단이 커질 가능성이 높다.
