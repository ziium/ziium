# 지음 (ziium)

지음은 한국어 문장 구조를 중심에 둔 프로그래밍 언어다. 목표는 기존 언어의 키워드를 한글로 번역하는 것이 아니라, 한국어 화자가 읽고 쓰는 흐름에 더 가까운 코드 표면 문법을 만드는 것이다.

현재 문서는 v0.1 설계 초안이다. v0.1은 "문장형 문법 + 엄격한 내부 구조 + 작은 실행 가능 범위"를 우선한다.

## 왜 만드는가

- 한국어 화자가 코드를 읽을 때 겪는 언어적 이질감을 줄인다.
- 교육용 언어가 아니라도 자연스러운 한국어 문장 흐름을 일부 보존할 수 있는지 검증한다.
- AI 에이전트와 사람이 같은 명세를 보고 안정적으로 언어 구현을 함께 진행할 수 있게 한다.

## 핵심 방향

- 문장은 한국어답게 보이되 내부 표현은 엄격해야 한다.
- v0.1은 자연어 이해가 아니라 명시적이고 예측 가능한 문법을 택한다.
- 표현식 내부에서는 일부 기호 연산을 허용해 구현 난도를 통제한다.
- 조사와 서술어는 장식이 아니라 의미를 구분하는 문법 요소로 취급한다.

## 예제

```txt
이름은 "철수"이다.
나이는 20이다.

나이를 출력한다.

나이 > 19이면
  "성인이다"를 출력한다.
아니면
  "미성년자이다"를 출력한다.
```

```txt
더하기 함수는 왼쪽, 오른쪽을 받아
  왼쪽 + 오른쪽을 돌려준다.

합은 더하기(3, 5)이다.
합을 출력한다.
```

```txt
사람은 { 이름: "영희", 나이: 18 }이다.
사람의 이름을 출력한다.
```

## v0.1 범위

- 값 바인딩과 재대입
- 숫자, 문자열, 불리언, 목록, 레코드
- 출력
- 조건문과 반복문
- 함수 정의와 호출
- `의`를 이용한 속성 접근
- 인터프리터 기준 실행 모델
- 한국어 오류 메시지 원칙

## 문서 안내

- [VISION.md](./VISION.md): 언어 철학과 설계 원칙
- [PRD.md](./PRD.md): 제품 수준 목표와 범위
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md): v0.1 공식 표면 문법 명세
- [TOKENS.md](./TOKENS.md): lexer용 토큰 규격과 조사 정규화 규칙
- [SEMANTICS.md](./SEMANTICS.md): AST, 스코프, 실행 의미
- [SYNTAX_GUIDE.md](./SYNTAX_GUIDE.md): 예제 중심 문법 가이드
- [GRAMMAR.ebnf](./GRAMMAR.ebnf): 파서 구현용 EBNF
- [PARSER_TEST_CASES.md](./PARSER_TEST_CASES.md): parser golden/snapshot 테스트 입력 초안
- [ARCHITECTURE.md](./ARCHITECTURE.md): 구현 구조 초안
- [ROADMAP.md](./ROADMAP.md): 단계별 개발 계획
- [TASKS.md](./TASKS.md): 바로 착수 가능한 작업 목록
- [DECISIONS.md](./DECISIONS.md): 주요 설계 결정 기록
- [TEST_PLAN.md](./TEST_PLAN.md): 테스트 전략
- [ERRORS.md](./ERRORS.md): 오류 메시지 원칙
- [AI_CONTEXT.md](./AI_CONTEXT.md): AI 협업 지침
- [CONTRIBUTING.md](./CONTRIBUTING.md): 기여 가이드
- [EXAMPLES.md](./EXAMPLES.md): 대표 예제 모음
- [samples/README.md](./samples/README.md): 바로 실행할 수 있는 샘플 프로그램 모음

## 현재 상태

현재는 Rust 기반의 최소 `lexer + parser + normalizer + interpreter + CLI` 골격이 들어가 있다. 문서 초안만 있는 단계는 지났고, 문서에 맞춘 기본 실행 경로와 테스트 세트가 함께 유지되고 있다.

## 실행과 확인

파일 실행:

```bash
cargo run -- path/to/program.zm
```

샘플 실행:

```bash
cargo run -- samples/00_hello_everyone.zm
cargo run -- samples/04_add_function.zm
```

토큰 보기:

```bash
cargo run -- tokens path/to/program.zm
```

AST 보기:

```bash
cargo run -- ast path/to/program.zm
```

REPL:

```bash
cargo run -- repl
```

블록을 포함하는 입력은 빈 줄을 한 번 더 입력하면 실행된다.

테스트:

```bash
cargo test
```

## 다음 단계

1. `TOKENS.md`, `LANGUAGE_SPEC.md`, `GRAMMAR.ebnf`를 기준으로 정규화 규칙을 더 고정한다.
2. `name resolution` 계층을 추가해 모호한 조사 분리와 이름 오류를 더 엄격하게 다룬다.
3. `PARSER_TEST_CASES.md`, `TEST_PLAN.md`를 기준으로 golden test와 snapshot test를 더 늘린다.
4. 인터프리터 위에 LLVM 또는 Cranelift 백엔드 초안을 올린다.
