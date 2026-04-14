# Research: 일본어 Lexer 프로토타입

2026-04-14

## 동기

지음(ziium)은 한국어 문장 구조를 중심에 둔 프로그래밍 언어다. 일본어는 한국어와 같은 교착어(SOV, 조사 기반)이므로, 동일한 `TokenKind` 체계 위에 일본어 lexer를 올릴 수 있는지 프로토타입으로 검증했다.

**목표**: 일본어 소스 → 기존 `TokenKind` 토큰 스트림 → 기존 파서/HIR/인터프리터로 실행. 어디까지 되고 어디서 깨지는지 매핑.

## 접근: Script Transition Tokenization

한국어 lexer는 **공백**으로 토큰 경계를 잡는다. 일본어는 띄어쓰기가 없으므로 다른 전략이 필요하다.

핵심 아이디어: 일본어 문자 체계(漢字, ひらがな, カタカナ)의 **전환점**이 자연적인 토큰 경계다.

```
名前は"哲夫"だ。
├漢字┤ひ├문자열┤ひ├구두점
```

`名前`(kanji) → `は`(hiragana) 전환 = 토큰 경계. `は`는 조사 테이블에서 `Topic`으로 매칭.

### 2-Phase Tokenization

각 위치에서:

1. **Keyword longest match** — 키워드 테이블을 길이 내림차순으로 시도. `出力する`(kanji+hiragana 혼합)처럼 script를 넘나드는 키워드도 원자적으로 매칭.
2. **Particle match** — 히라가나 전용 조사 테이블 시도.
3. **Segment collection** — 같은 script class의 문자를 수집. 漢字 뒤의 送り仮名(okurigana)는 키워드/조사가 아닌 경우 확장.

### 경계 판정

키워드 `だ`(Copula)와 `だけ`(Amount)의 모호성 해결:

```
매칭 후 다음 문자가 같은 script class면 → 경계 아님 → 매칭 실패
```

`だけ`에서 `だ` 시도 → 다음 `け`도 hiragana → 경계 아님 → skip → `だけ` 시도 → 성공.

### 送り仮名 (Okurigana)

漢字 뒤에 오는 히라가나가 같은 단어의 일부인 경우:

```
長さを → IDENT("長さ") Object("を")
```

漢字 수집 후, 뒤따르는 히라가나가 키워드/조사의 시작이 아니면 같은 토큰으로 확장.

## 구현 결과

### 성공 (MVP 경로)

| 기능 | 한국어 | 일본어 | 결과 |
|------|--------|--------|------|
| 바인딩 | `이름은 "철수"이다` | `名前は"哲夫"だ` | 동작 |
| 출력 | `나이를 출력한다` | `年齢を出力する` | 동작 |
| 조건문 | `나이 > 19이면` | `年齢 > 19なら` | 동작 |
| 함수 정의 | `인사 함수는 이름을 받아` | `挨拶 関数は 名前を 受けて` | 동작 |
| 기호 연산 | `합계는 10 + 20이다` | `合計は 10 + 20だ` | 동작 |
| 리스트 | `[1, 2, 3]` | `[1、2、3]` | 동작 |

**파서, HIR, 인터프리터를 한 줄도 수정하지 않고 동작한다.**

### 실패 (MVP 밖)

파서에서 한국어 lexeme을 직접 비교하는 기능은 일본어에서 깨진다:

| 기능 | 파서 코드 | 깨지는 이유 |
|------|-----------|-------------|
| Sleep | `lexeme == "초"`, `lexeme == "쉬기"` | 한국어 문자열 하드코딩 |
| 한국어 비교 | `lexeme == "크면"` 등 | 한국어 문자열 하드코딩 |
| Resultive | `lexeme == "꺼낸"` 등 | 한국어 문자열 하드코딩 |
| Named call | `lexeme == "호출한다"` | 한국어 문자열 하드코딩 |
| Relative change | `lexeme == "줄인다"` 등 | 한국어 문자열 하드코딩 |
| Canvas | `receiver == "그림판"` | 한국어 문자열 하드코딩 |

## 핵심 발견: 아키텍처 건강 진단

이 프로토타입은 지음의 아키텍처에 두 계층이 있음을 드러냈다.

### 언어 중립 계층 (건강)

```
TokenKind 체계 → 파서 MVP 경로 → HIR → 인터프리터
```

- `TokenKind`의 격 조사 분류(`Topic`, `Subject`, `Object`, `Gen` 등)는 교착어 일반에 적용되는 보편적 체계
- 파서의 바인딩·출력·조건문·함수 경로는 순수 `TokenKind` 기반
- 이 계층에 추가되는 기능은 자동으로 다국어 호환

### 한국어 종속 계층 (기술 부채)

```
파서의 lexeme == "한국어문자열" 비교 50+ 곳
normalizer의 한국어 조사 분리
message.rs의 한국어 ↔ enum 매핑
```

- 이 계층에 추가되는 기능은 한국어에서만 동작
- 장기적으로 `TokenKind` 기반으로 올려야 하는 부채

### 리트머스 테스트

한국어 기능을 확장할 때:

> "이 기능이 `TokenKind`만으로 표현되는가, 아니면 lexeme 문자열에 의존하는가?"

전자면 건강한 확장. 후자면 나중에 갚아야 할 부채.

## TokenKind 대응표

일본어 프로토타입에서 사용한 매핑:

### 키워드

| 일본어 | TokenKind | 한국어 대응 |
|--------|-----------|-------------|
| `でなければ` | Else | 아니면 |
| `ではない` | Not | 아니다 |
| `出力する` | Print | 출력한다 |
| `である` | Copula | 이다 (formal) |
| `受けて` | Receive | 받아 |
| `または` | Or | 또는 |
| `変える` | Change | 바꾼다 |
| `返す` | Return | 돌려준다 |
| `の間` | During | 동안 |
| `なら` | If | 이면 |
| `かつ` | And | 그리고 |
| `なし` | None | 없음 |
| `真` | True | 참 |
| `偽` | False | 거짓 |
| `だ` | Copula | 이다 |

### 조사

| 일본어 | TokenKind | 한국어 대응 |
|--------|-----------|-------------|
| `は` | Topic | 은/는 |
| `が` | Subject | 이/가 |
| `を` | Object | 을/를 |
| `の` | Gen | 의 |
| `に` | Locative | 에 |
| `で` | Direction | 으로/로 |
| `から` | From | 에서 |
| `より` | Than | 보다 |
| `だけ` | Amount | 만큼 |

## Mixed Lexer 설계 스케치 (미구현)

한일 혼용 코드(`名前は "진토"이다`)를 지원하는 설계를 검토했다.

### 핵심 아이디어

시작 문자의 script class로 각 언어의 tokenization 전략에 위임:

```
Hangul로 시작 → 한국어 전략 (공백 기반 + 키워드 매칭)
Kanji/Hiragana/Katakana로 시작 → 일본어 전략 (script transition)
Latin으로 시작 → 공통 라틴 식별자
```

### Trait 설계

```rust
trait LanguageBackend {
    fn is_word_start(ch: char) -> bool;
    fn lex_word(lexer: &mut Lexer, chars: &[char], i: usize) -> usize;
    fn consume_postfix(lexer: &mut Lexer, chars: &[char], i: usize) -> usize;
    fn extra_punct(ch: char) -> Option<TokenKind>;
}
```

### 구현 판단

Mixed lexer를 **제품 기능**으로 유지하면 모든 새 기능에 다국어 대응 부담이 생겨 한국어 발전의 걸림돌이 된다. **아키텍처 테스트**로 유지하면 `TokenKind` 중립성 위반을 조기 감지하는 도구가 된다.

현재 결론: 구현하지 않는다. 일본어 프로토타입 자체가 이미 아키텍처 건강 진단의 목적을 달성했다. 향후 Language Backend 분리가 필요할 때 이 설계를 참고한다.

## 파일 목록

| 파일 | 역할 |
|------|------|
| `src/lexer_ja.rs` | 일본어 lexer 구현 |
| `src/lib.rs` | `lex_ja` 공개 API |
| `src/main.rs` | `.zmj` 확장자 감지 및 dispatch |
| `samples/ja_00_hello.zmj` | 바인딩·출력·조건문 샘플 |
| `samples/ja_01_function.zmj` | 함수 정의·호출 샘플 |
| `tests/lexer_ja_examples.rs` | lexer 테스트 13개 |
