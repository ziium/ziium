# AI Context

이 문서는 이 프로젝트를 다루는 AI 에이전트용 작업 지침이다. 목표는 설계 철학을 망치지 않으면서도 구현 속도를 유지하는 것이다.

## 이 프로젝트의 목표

- 한국어 문장형 프로그래밍 언어 v0.1을 설계하고 구현한다.
- 문서와 구현을 함께 발전시킨다.
- 문장형 표면 문법과 엄격한 내부 표현을 동시에 유지한다.

## 반드시 먼저 읽을 문서

1. `README.md`
2. `VISION.md`
3. `TOKENS.md`
4. `LANGUAGE_SPEC.md`
5. `SEMANTICS.md`
6. `GRAMMAR.ebnf`
7. `PARSER_TEST_CASES.md`
8. `ARCHITECTURE.md`
9. `DECISIONS.md`

## 절대 바꾸면 안 되는 원칙

- 이 언어를 "영어 키워드의 한글 번역판"으로 되돌리지 말 것
- 조사와 서술어의 역할을 문법에서 제거하지 말 것
- 문서와 구현이 어긋나는 변경을 단독으로 하지 말 것
- 테스트 없이 새 문법을 추가하지 말 것
- 암시적 자연어 추론으로 parser를 복잡하게 만들지 말 것

## 작업 규칙

- 문법 변경 시 `LANGUAGE_SPEC.md`와 `GRAMMAR.ebnf`를 함께 수정한다.
- 의미 변화가 있으면 `SEMANTICS.md`를 함께 수정한다.
- 중요한 판단은 `DECISIONS.md`에 기록한다.
- 예제 추가 시 `EXAMPLES.md`나 `SYNTAX_GUIDE.md`에 반영한다.

## 구현 우선순위

1. lexer 안정화
2. parser 안정화
3. parser snapshot test
4. interpreter
5. diagnostics polish
6. 문법 확장

## 코드 작성 원칙

- surface syntax와 내부 AST를 분리한다.
- 조사 처리 규칙은 lexer/parser 경계에서 명확히 둔다.
- 오류 메시지는 구조화된 진단 시스템을 통해 만든다.
- 블록 규칙은 early error로 처리한다.

## PR 또는 변경 설명 원칙

- 어떤 문법을 바꿨는지
- 어떤 테스트를 추가했는지
- 어떤 문서를 갱신했는지
- 남은 위험은 무엇인지

## 피해야 할 흔한 실수

- `의`를 문자열 치환 수준으로 처리
- `...은 ...이다`를 재대입으로 허용
- truthiness 허용
- `if`, `while`, `return` 같은 영어 키워드 도입
- 문서 예제를 구현과 분리된 채 방치
