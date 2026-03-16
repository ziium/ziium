# Contributing

이 프로젝트는 아직 문서 중심 단계에 있다. 구현을 시작하더라도 문서를 먼저 읽고, 변경이 문법 철학과 일치하는지 확인해야 한다.

## 시작 전에 읽을 문서

1. `README.md`
2. `VISION.md`
3. `TOKENS.md`
4. `LANGUAGE_SPEC.md`
5. `GRAMMAR.ebnf`
6. `PARSER_TEST_CASES.md`
7. `ARCHITECTURE.md`
8. `DECISIONS.md`

## 기여 절차

1. 변경 목적을 분명히 한다.
2. 관련 명세 문서를 먼저 확인한다.
3. 문법 변경이면 문서부터 고친다.
4. 구현과 테스트를 함께 수정한다.
5. 변경 이유를 짧게 기록한다.

## 문법 변경 체크리스트

- `LANGUAGE_SPEC.md`를 수정했는가
- `TOKENS.md` 영향이 있는가
- `GRAMMAR.ebnf`를 수정했는가
- `SEMANTICS.md` 영향이 있는가
- parser test를 추가했는가
- diagnostics 예시를 확인했는가
- `DECISIONS.md` 기록이 필요한가

## 버그 수정 체크리스트

- 재현 예제를 남겼는가
- 회귀 테스트를 추가했는가
- 문서 예제와 충돌하지 않는가

## 커밋 또는 변경 설명에 포함할 것

- 문제 요약
- 해결 방식
- 테스트
- 문서 변경 여부
- 남은 제한 사항

## 우선순위 가이드

초기 기여자는 아래 순서를 권장한다.

1. lexer/parser
2. parser snapshot test
3. interpreter
4. diagnostics
5. REPL 또는 CLI

## 토론이 필요한 변경

아래는 바로 구현하지 말고 먼저 합의하는 편이 좋다.

- 새 제어문 추가
- 조사 생략 허용
- 비교식 자연어 확장
- 타입 시스템 도입
- 블록 문법 변경
