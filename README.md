<div align="center">

# 지음 (ziium)

**한국어 문장 구조를 중심에 둔 프로그래밍 언어**

한글 키워드 번역이 아니라, 한국어 화자가 읽고 쓰는 흐름에 가까운 코드를 만든다.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.3-brightgreen.svg)](HISTORY.md)
[![Rust 2024](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)

[문법 명세](docs/LANGUAGE.md) · [예제 모음](samples/README.md) · [브라우저 Playground](#브라우저-playground)

</div>

<br>

```
이름은 "철수"이다.
나이는 20이다.

나이 > 19이면
  "성인이다"를 출력한다.
아니면
  "미성년자이다"를 출력한다.
```

---

## 왜 만드는가

- 한국어 화자가 코드를 읽을 때 겪는 **언어적 이질감**을 줄인다.
- 자연스러운 한국어 문장 흐름을 일부 보존할 수 있는지 **검증**한다.
- AI 에이전트와 사람이 같은 명세로 언어 구현을 함께 **진행**할 수 있게 한다.

## 핵심 방향

| 원칙 | 설명 |
|------|------|
| 한국어다운 문장 | 표면은 한국어 문장형, 내부 표현은 엄격 |
| 예측 가능한 문법 | 자연어 이해가 아니라 명시적 규칙 |
| 조사는 문법 요소 | `을/를`, `에`, `으로` 등은 장식이 아니라 의미 구분자 |
| 통제된 기호 사용 | 표현식 내부에서 일부 기호 연산 허용 |

## 맛보기

```
인사만들기 함수는 이름을 받아
  "안녕, " + 이름 + "!"을 돌려준다.

문장은 "지음"으로 인사만들기이다.
문장을 출력한다.
```

```
그림판에 { 배경색: "#f6efe2" }으로 지우기.
그림판에 { x: 120, y: 80, 색: 빨강 }으로 점찍기.
그림판에 { 글: "지음", x: 160, y: 60, 색: "#3b2f2f", 크기: 24 }로 글자쓰기.
```

```
체력은 100이다.
체력이 50보다 크면
  "안전하다"를 출력한다.
체력을 30만큼 줄인다.

선택은 ["공격", "도망"]에서 고른 것이다.
선택을 출력한다.
```

> 19개 샘플 전체는 [samples/README.md](samples/README.md)에 정리되어 있다.
> 인터랙티브 텍스트 어드벤처는 `samples/13_story.zm`.

## 현재 범위 (v0.3)

v0.1 코어 위에 v0.2에서 메시지 프레임 연구 원칙이 고정되었고, v0.3에서 코어 문법이 확장되었다.

| 범주 | 내용 |
|------|------|
| 바인딩 | 불변(`이다` = const), 가변(`넣는다` = let) |
| 자료형 | 숫자, 문자열, 불리언, 목록, 레코드 |
| 제어 흐름 | 블록/단문 if-else, while, for-each |
| 함수 | 정의, 호출, 적용 바인딩(`X를 Y한 것이다`) |
| 연산 | `의` 속성 접근, 인덱스 대입, 타입 변환 |
| 실험 프레임 | `X으로 Y`, `X 더하기 Y`, `대상에 값 추가`, `에서 고른 것이다` 등 |
| 기반 | 인터프리터 실행 모델, 한국어 오류 메시지 |

> 버전별 변화는 [HISTORY.md](HISTORY.md), 설계 결정 기록은 [DECISIONS.md](docs/DECISIONS.md).

## 시작하기

```bash
# 파일 실행
cargo run -- samples/00_hello_everyone.zm

# REPL (빈 줄로 블록 실행)
cargo run -- repl

# 테스트
cargo test
```

<details>
<summary>디버깅 도구</summary>

```bash
cargo run -- tokens path/to/program.zm   # 토큰 보기
cargo run -- ast path/to/program.zm      # AST 보기
cargo run -- hir path/to/program.zm      # HIR 보기
```

</details>

## 브라우저 Playground

`web/` 아래에 세 가지 데모가 있다.

- **텍스트 하노이탑** — 이동 경로를 출력으로 바로 확인
- **캔버스 하노이탑** — `그림판` 호출로 만든 프레임을 캔버스로 재생
- **숲속의 용사** — 비교, 상대적 변화, 선택 프레임을 사용한 텍스트 어드벤처

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --out-dir web/pkg
python3 -m http.server 8000 --directory web
# → http://localhost:8000
```

> 자세한 내용은 [web/README.md](web/README.md).

## 문서

| 문서 | 설명 |
|------|------|
| [LANGUAGE.md](docs/LANGUAGE.md) | 언어 철학, 문법, 의미, 메시지 경계 |
| [GRAMMAR.ebnf](docs/GRAMMAR.ebnf) | 파서 구현용 형식 문법 |
| [IMPLEMENTATION.md](docs/IMPLEMENTATION.md) | 구현 구조, 테스트, 작업 순서 |
| [DECISIONS.md](docs/DECISIONS.md) | 주요 설계 결정 기록 |
| [HISTORY.md](HISTORY.md) | 버전별 변화 요약 |
| [samples/](samples/README.md) | 바로 실행할 수 있는 샘플 프로그램 모음 |

## 아키텍처

```
소스 (.zm)
  → Lexer → Parser → Normalizer → Resolver → HIR Lowering → Interpreter
                                                              ↓
                                                          CLI / REPL / WASM
```

## 다음 단계

1. HIR `SendSelector` 일반화 범위 결정
2. 메시지 집합 확장 범위 설정
3. 정규화 규칙 고정
4. LLVM 또는 Cranelift 백엔드 초안

---

<div align="center">

MIT License · [기여 가이드](AGENTS.md)

</div>
