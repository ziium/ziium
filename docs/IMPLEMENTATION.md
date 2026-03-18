# Implementation

이 문서는 지음의 현재 구현 구조, 테스트 방식, 작업 순서, 기여 기준을 한곳에 모은다. 언어 표면 규칙은 `LANGUAGE.md`, 형식 문법은 `GRAMMAR.ebnf`, 설계 판단 기록은 `DECISIONS.md`를 기준으로 본다.

## 현재 상태

- 구현 언어: Rust
- 실행 모델: tree-walk interpreter
- 제공 도구: CLI, REPL, 테스트 세트
- 현재 단계: 최소 실행 가능한 프런트엔드와 인터프리터가 동작한다.

아직 없는 것:

- 타입체커
- 바이트코드 VM
- LLVM/Cranelift 백엔드
- 패키지/모듈 시스템

## 전체 파이프라인

```txt
source -> lexer -> normalizer -> parser -> resolver -> hir lowering -> interpreter
```

### 각 단계의 책임

- `lexer`: 토큰화, 들여쓰기 토큰, 붙은 조사 1차 분리
- `normalizer`: 한글 조사 모호성에 대한 문맥 보정
- `parser`: AST 생성
- `resolver`: 이름 해석, 스코프 검사, definite binding 검사
- `hir lowering`: surface AST를 `Send` 중심 HIR로 낮춘다
- `interpreter`: HIR 실행, 내장 함수/메시지 처리, 런타임 진단
- `interpreter` 실행 결과는 출력, 캔버스 프레임, `쉬기` 시간을 같은 이벤트 열로도 유지한다.

### 현재 내부 의미 모델

표면 AST는 여전히 `Call`, `TransformCall`, `Property`, `KeywordMessage`, `Resultive`처럼 surface syntax 중심으로 나뉜다. 그 다음 HIR lowering이 이를 `Send` 중심 표현으로 다시 정리한다.

- unary message: `길이`, `제곱`
- keyword message: `추가`, `지우기`, `점찍기`, `사각형채우기`, `글자쓰기`
- resultive message: `맨위 원반을 빼낸`

즉 현재 구현은 `surface AST + send-centered HIR`의 2단 구조다. parser는 한국어 문장형 표면을 보존하고, interpreter는 HIR의 selector 기반 메시지 모델만 실행한다.

현재 연구 원칙은 아래와 같다.

- 실험 프레임은 표면 형태가 달라도 내부에서는 `Send` 공통형으로 모은다.
- `A의 B`는 parser에서 속성 접근으로 먼저 읽고, built-in noun message 판단은 lowering/runtime으로 미룬다.
- `A으로 B`는 당분간 unary transform frame으로만 다룬다.
- 상태 변경과 결과 서술 프레임은 조회/계산 프레임보다 더 좁은 built-in 경계로 유지한다.

## 주요 소스 구조

- `src/token.rs`: 토큰 종류
- `src/lexer.rs`: 토큰화
- `src/normalizer.rs`: 조사 보정
- `src/ast.rs`: AST
- `src/parser.rs`: 구문 분석
- `src/resolver.rs`: 이름 해석
- `src/hir.rs`: HIR과 AST -> HIR lowering
- `src/interpreter.rs`: HIR 실행기
- `src/error.rs`: 구조화된 진단
- `src/main.rs`: CLI/REPL

## 저장소 구조

- `README.md`: 사람용 입구와 맛보기 예제
- `HISTORY.md`: 버전별 단계와 변화 요약
- `docs/LANGUAGE.md`: 공식 언어 문서
- `docs/GRAMMAR.ebnf`: 파서용 형식 문법
- `docs/DECISIONS.md`: 설계 결정 기록
- `docs/IMPLEMENTATION.md`: 구현 구조와 작업 방식
- `samples/*.zm`: 실행 가능한 샘플
- `samples/README.md`: 샘플 인덱스
- `tests/*.rs`: 회귀 테스트
- `tests/fixtures/parser_cases.md`: parser 문서 기반 fixture
- `AGENTS.md`: 에이전트 작업 규칙

## 테스트 전략

기본 검증 명령:

```bash
cargo test
```

현재 테스트 범주:

- lexer 테스트
- parser 테스트
- resolver 테스트
- interpreter 테스트
- CLI/REPL 스모크 테스트
- parser fixture 문서 개수 검증

### parser fixture

- 파일: `tests/fixtures/parser_cases.md`
- 목적: 대표 성공/실패 입력 세트를 문서 형태로 유지하고 테스트 자산으로 재사용

## 문법 변경 작업 규칙

문법을 바꾸면 아래를 함께 갱신한다.

1. `docs/LANGUAGE.md`
2. `docs/GRAMMAR.ebnf`
3. 관련 테스트
4. 필요 시 `docs/DECISIONS.md`
5. 필요 시 `README.md` 또는 `samples/*.zm`

메시지 문법을 바꿀 때는 특히 아래를 먼저 확인한다.

- 현재 메시지 집합이 built-in으로 닫혀 있는지
- 일반 식별자와 충돌하지 않는지
- parser가 자연어 추론으로 과도하게 넓어지지 않는지

## 기여 기준

### 변경 절차

1. 변경 목적을 명확히 한다.
2. 언어 규칙이면 `LANGUAGE.md`를 먼저 확인한다.
3. 구현과 문서를 어긋나지 않게 같이 수정한다.
4. 회귀 테스트를 추가하거나 갱신한다.
5. 설계 판단이 새로 생기면 `DECISIONS.md`에 기록한다.

### 변경 설명에 포함할 것

- 무엇을 바꿨는가
- 어떤 테스트를 추가하거나 갱신했는가
- 어떤 문서를 갱신했는가
- 남은 제한이나 리스크는 무엇인가

### 토론이 필요한 변경

- 새 제어문 추가
- 조사 생략 허용
- 비교식 자연어 확장
- 타입 시스템 도입
- 블록 문법 변경
- 메시지 집합 개방

## 현재 우선순위

1. `SendSelector`를 현재 닫힌 메시지 집합과 조사 프레임 경계에 맞게 더 명확히 정리
2. `의` 프레임의 property-first parsing과 built-in noun fallback 규칙을 lowering/runtime 기준으로 고정
3. `으로` 프레임의 unary transform 제한을 문서와 테스트로 더 분명히 고정
4. 상태 변경 및 결과 서술 프레임의 닫힌 범위를 유지하면서 회귀 테스트를 보강

## 중장기 방향

### v0.1 완료 기준

- README와 샘플의 핵심 예제가 대부분 실행된다.
- parser 회귀 테스트가 안정적으로 유지된다.
- 진단이 줄/열과 코드 프레임을 제공한다.

### v0.2 후보

- 더 넓은 메시지 중심 HIR
- unary noun / binary word / keyword verb 메시지 확장
- 더 나은 REPL 경험
- 포매터 초안

### v0.3 후보

- 정적 검사 계층
- 바이트코드 또는 LLVM/Cranelift 백엔드
- LSP
- 패키지/모듈 구조

## 구현 원칙

- surface syntax와 내부 표현을 분리한다.
- 조사 처리 규칙은 lexer와 normalizer 경계에서 최대한 명확히 둔다.
- parser는 추측보다 명시적 구조를 우선한다.
- diagnostics는 처음부터 구조화된 데이터로 유지한다.
- 문서와 테스트를 구현 일부로 취급한다.
