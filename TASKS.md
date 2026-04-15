# Test-Driven Language Evolution

플랜: `.claude/plans/test-driven-language-evolution.md`

## Plan 1: 인프라 (재귀 가드 + 테스트 감사) ✅

### 1-1. 재귀 깊이 가드

- [x] 실패 테스트 작성: `tests/rosetta_examples.rs`에 재귀 호출 → `RuntimeError` 확인
  ```rust
  #[test]
  fn rejects_recursion_exceeding_depth_limit() {
      let source = r#"세기 함수는 숫자를 받아
    세기(숫자 + 1)

  세기(0)"#;
      let err = run_source(source).expect_err("should hit recursion limit");
      assert!(err.to_string().contains("재귀 깊이 제한"));
  }
  ```
- [x] `Interpreter` struct에 `call_depth: usize` 필드 추가, `new()`에서 0 초기화
- [x] `call_value` User 함수 경로에 depth guard 구현: 진입 시 증가 + 상한(64) 검사, 결과 캡처 후 감소
  ```rust
  struct CallDepthGuard<'a> {
      depth: &'a mut usize,
  }
  impl<'a> CallDepthGuard<'a> {
      fn new(depth: &'a mut usize, limit: usize) -> Result<Self, RuntimeError> {
          *depth += 1;
          if *depth > limit {
              *depth -= 1;
              return Err(RuntimeError::new("재귀 깊이 제한을 초과했습니다."));
          }
          Ok(Self { depth })
      }
  }
  impl Drop for CallDepthGuard<'_> {
      fn drop(&mut self) { *self.depth -= 1; }
  }
  ```
- [x] `cargo test` — 전체 137개 통과 (새 1 + 기존 136), 회귀 0
- [x] 커밋: `38f37d5`

### 1-2. 기존 테스트 커버리지 감사

- [x] 126개 테스트를 10개 축으로 분류 → 커버리지 맵 테이블 작성
- [x] 빈 영역 식별 + 우선순위 정렬 → 8개 항목 산출
- [x] **게이트**: 우선순위 목록 승인됨

## Plan 2: 테스트 스위트 (5배치 × 10개 = 50개) ✅

- [x] 배치 1: 기능 테스트 — 연산자/조건문/반복문 (11개): `9280b15`
- [x] 배치 2: 기능 테스트 — 함수/문자열 (10개): `ffd3491`
- [x] 배치 3: 에러 테스트 — 타입 오류/인자/인덱스/빌트인 (9개)
- [x] 배치 4: 기능+조합 테스트 — 목록/레코드/논리/타입 (13개)
- [x] 배치 5: 조합+탐색 테스트 — Rosetta (10개, 탐색 2개 `#[ignore]`)

배치 게이트: green ≥ 8/10, red 전부 backlog 항목 존재, 기존 회귀 0

## Plan 3: 기능 구현 (실패 테스트 기반) ← 현재

- [x] 실패 분석 → 기능 백로그 작성 (P-1~P-4)
- [x] P-1 구현 ✅, P-2 구현 ✅, P-3 구현 ✅
- [x] P-4 구현 ✅ → 192 테스트 통과, 회귀 0

### Backlog: 배치 5 프로빙에서 발견한 파서 한계

#### P-1. 단음절 식별자가 조사 토큰으로 소비됨 ✅
- **증상**: 매개변수 `가, 나를 받아` → "첫 번째 매개변수 이름이 필요합니다"
- **수정**: `exact_word_kind`에서 `"이"|"가"` 제거 + normalizer에 문맥 기반 재분류 2건 추가
- **테스트**: `interpreter_examples.rs` — 6개 테스트 추가

#### P-2. 2음절 이상 식별자의 조사 접미사 오분리 ✅
- **증상**: `하노이 함수는` → `Ident("하노") + Subject("이")`, 함수 정의 파싱 실패
- **수정**: PARTICLES에서 `이`/`가` 제거 (17→15) + 이중 정규화 버그 수정 (`parser.rs`)
- **부수 수정**: `split_object_before_keyword`에 단음절 guard 추가 (`마을` 오분리 방지)
- **테스트**: `interpreter_examples.rs` — 5개 테스트 추가

#### P-3. 키워드 접미사 무조건 분리 (`인`, `이면`, `이다`) ✅
- **증상**: `회문확인 함수는` → `Ident("회문확") + In("인")`, "`인` 뒤에는 `동안`이 와야"
- **수정**: KEYWORD_SUFFIXES에서 `인` 제거 + `should_split_word` 화이트리스트 축소 + normalizer에 `split_in_before_during` 추가 + 리뷰에서 발견한 단음절 회귀 수정 (`이면`/`이다`를 화이트리스트에 복원)
- **테스트**: `interpreter_examples.rs` — 3개 테스트 추가

#### P-4. while 본문이 반복 간 스코프를 공유 → 루프 내 바인딩 불가 ✅
- **증상**: while 안에서 `중간은 ...이다` → 2회차에 "이미 정의되어 있습니다"
- **수정**: while 본문 실행 시 `Environment::new(Some(env.clone()))` — 매 반복 자식 스코프 생성
- **테스트**: `interpreter_examples.rs` — 2개 테스트 추가 (루프 내 바인딩 + 스코프 격리)

## Plan 4: 리스트 인덱스 대입 ✅

플랜: `.claude/plans/plan4-index-assignment.md`

### 4-1. 파서 — IndexAssign AST 파싱 ✅
- [x] `ast.rs`에 `IndexAssign { base, index, value }` variant 추가
- [x] `parser.rs` Object 분기 재구성 — `finish_index_assign` + `parse_index_relative_change` + NamedCall 인라인
- [x] `extract_index_target`: `Expr::Index { base: Name }` 추출, 비-Name 기반 에러
- [x] 파서 테스트 2개 통과 + 기존 parser_examples 회귀 0

### 4-2. 전체 파이프라인 — 기본 인덱스 대입 ✅
- [x] `hir.rs`: `IndexAssign` variant + `lower_stmt` + `Stmt::span()` match arm
- [x] `resolver.rs`: `IndexAssign` — `resolve_name(base)` + `resolve_expr(index, value)` + `collect_unconditional_names`
- [x] `interpreter.rs`: `IndexAssign` — `lookup_value` → `borrow_mut()[idx] = new_val`
- [x] 인터프리터 테스트 3개 통과 (기본, 표현식 인덱스, swap)

### 4-3. 상대적 변화 ✅
- [x] `parse_index_relative_change`: desugar `IndexAssign { value: Binary { Index, op, rhs } }`
- [x] 테스트 통과 (`runs_index_relative_change`)

### 4-4. 에러 경로 ✅
- [x] 4개 에러 테스트 통과 (범위 초과, 음수 인덱스, 문자열 대상, 미정의 변수)

### 4-5. 통합 — Bubble Sort ✅
- [x] `explore_bubble_sort`에서 `#[ignore]` 제거 → 통과
- [x] `cargo test` 전체 203 pass, 1 ignored, 0 fail

### 4-6. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `index_assign_stmt` 규칙 추가
- [x] `docs/LANGUAGE.md` — 인덱스 재대입 섹션 추가
- [x] `docs/DECISIONS.md` — "리스트 인덱스 대입: IndexAssign variant 채택" 기록
