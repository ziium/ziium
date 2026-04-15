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

## Plan 5: 적용 바인딩 (`X를 Y한 것이다`) ✅

플랜: `.claude/plans/parsed-hatching-coral.md`

### 5-1. 파서 — applied bind 파싱 ✅
- [x] RED: `parses_applied_bind_expression` 테스트 → 실패 확인
- [x] GREEN: `parse_bind`에 `Object` 분기 + `parse_applied_bind_expression` 구현
- [x] 기존 테스트 회귀 0 확인

### 5-2. 인터프리터 — 기본 실행 ✅
- [x] `runs_applied_bind_expression` — `5를 두배한 것이다` → 10
- [x] `runs_applied_bind_with_complex_input` — `-7을 절대값한 것이다` → 7

### 5-3. 에러 경로 ✅
- [x] `rejects_applied_bind_without_han_suffix` — `한` 없는 함수 이름 → 에러

### 5-4. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `applied_expr` 규칙 추가
- [x] `docs/LANGUAGE.md` — 적용 바인딩 섹션 추가
- [x] `docs/DECISIONS.md` — "적용 바인딩: 파서 접근 채택" 기록

## Plan 6: 단문 if-else (`조건이면 문장~고 아니면 문장~다`) ✅

플랜: `.claude/plans/inline-if-else.md`

### 6-1. 렉서 + 파서 — 단문 if-else 파싱 + 기본 실행 ✅
- [x] RED: `tests/interpreter_examples.rs`에 `runs_inline_if_else_return` 추가
- [x] GREEN: `src/lexer.rs` — `exact_word_kind`에 연결형 동사 3개 추가
- [x] GREEN: `src/parser.rs` — `parse_if_tail` 수정 (Newline 분기 + inline 경로)
- [x] 기존 테스트 207개 + 1 ignored 회귀 0 확인

### 6-2. 추가 테스트 ✅
- [x] `runs_inline_if_else_print` — 출력문 분기
- [x] `runs_inline_if_without_else` — else 없는 guard clause
- [x] `runs_inline_if_else_with_korean_comparison` — 한국어 비교문과 조합
- [x] `parses_inline_if_else` — AST 구조 검증 (parser_examples)

### 6-3. 통합 — GCD 함수 ✅
- [x] `runs_gcd_with_inline_if` — 최대공약수 함수 전체 실행

### 6-4. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `if_stmt` 규칙에 인라인 형식 추가
- [x] `docs/LANGUAGE.md` — 단문 if-else 섹션 추가
- [x] `docs/DECISIONS.md` — "단문 if-else: 렉서 접근 채택" 기록

## Plan 7: 가변 바인딩 (`넣는다` = let, `이다` = const) ✅

플랜: `.claude/plans/mutable-bind.md`

플랜: `.claude/plans/mutable-bind.md`

### Phase A: `넣는다` 파이프라인 관통 (비파괴적) ✅

#### 7-1. Token + Lexer + AST + HIR — 기반 구조 ✅
- [x] `src/token.rs`: `Store` variant 추가
- [x] `src/lexer.rs`: `넣는다`/`넣고` → Store 매핑 + `에` 단음절 분리 화이트리스트 추가
- [x] `src/ast.rs`: `Bind { mutable: bool }` — 기존 match arm에 `..` 추가
- [x] `src/hir.rs`: `Bind { mutable: bool }` + `lower_stmt`에서 mutable 전달
- [x] 컴파일 확인 (기존 테스트 전체 통과)

#### 7-2. Parser — `넣는다` 바인딩 파싱 ✅
- [x] RED: `tests/interpreter_examples.rs`에 `runs_mutable_bind_basic` 추가
- [x] GREEN: `src/parser.rs` — Locative 분기에서 Object 유무로 mutable bind vs keyword message 구분
- [x] `parse_keyword_message` → `finish_keyword_message` 리팩터 (arg 사전 파싱)
- [x] 기존 테스트 전체 통과 + 새 테스트 통과

#### 7-3. 추가 테스트 — `넣는다` 동작 확인 ✅
- [x] `runs_mutable_bind_with_expression` — `오른쪽에 목록의 길이 - 1을 넣는다`
- [x] `runs_mutable_bind_with_inline_if` — `만약 x > 0이면 결과에 x를 넣고 아니면 결과에 0을 넣는다`
- [x] `parses_mutable_bind` — AST 구조 검증: `Bind { mutable: true }`

### Phase B: 불변성 강제 + 마이그레이션 ✅

#### 7-4. Resolver — 불변 바인딩 재대입 에러 ✅
- [x] RED: `tests/resolver_examples.rs`에 `rejects_reassign_to_const_binding` 추가
- [x] GREEN: `src/resolver.rs` — `defined_now`를 `HashMap<String, bool>`로 변경, Assign 시 `check_mutable`
- [x] 함수 매개변수는 mutable로 등록, FunctionDef는 immutable
- [x] `allows_reassign_to_mutable_binding`, `allows_reassign_to_function_param` 추가

#### 7-5. Interpreter — 불변 체크 + 에러 ✅
- [x] `src/interpreter.rs` — Environment에 `immutable: HashSet<String>` 추가
- [x] `Stmt::Bind` 핸들러에서 `mutable: false` → immutable set에 등록
- [x] `assign_value`에서 immutable 체크 (리졸버와 이중 방어)

#### 7-6. 테스트 마이그레이션 ✅
- [x] `tests/rosetta_examples.rs` — 재대입되는 `이다` 바인딩 20+ 개를 `넣는다`로 변경
- [x] `tests/interpreter_examples.rs` — 재대입되는 `이다` 바인딩 5개를 `넣는다`로 변경
- [x] 전체 222 테스트 통과, 1 ignored, 회귀 0

#### 7-7. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `mutable_bind_stmt` 규칙 + `STORE` 토큰 정의
- [x] `docs/LANGUAGE.md` — 불변 바인딩 / 가변 바인딩 / 재대입 섹션 분리
- [x] `docs/DECISIONS.md` — "가변/불변 바인딩: 넣는다=let, 이다=const 채택" 기록

## Plan 8: for-each 반복문 (`<컬렉션>의 각각 <변수>에 대해`) ✅

### 8-1. Token + Lexer + AST + HIR — 기반 구조 ✅
- [x] `src/token.rs`: `Each`, `About` variant 추가
- [x] `src/lexer.rs`: `각각` → Each, `대해` → About 매핑
- [x] `src/ast.rs`: `ForEach { collection, variable, body }` 추가
- [x] `src/hir.rs`: `ForEach` HIR variant + `lower_stmt` + `Stmt::span()` match arm
- [x] 컴파일 확인 (기존 224 테스트 전체 통과, 회귀 0)

### 8-2. 파서 + Resolver + Interpreter ✅
- [x] `src/parser.rs`: `parse_postfix`에서 Gen + Each 루카헤드 → break (property 파싱 방지)
- [x] `src/parser.rs`: `parse_statement`에서 Gen + Each 감지 → for-each 파싱
- [x] `src/resolver.rs`: 새 자식 스코프 push + 반복 변수 mutable 선언 + body 해석 + pop
- [x] `src/interpreter.rs`: List 순회 + 매 반복 자식 스코프 + 변수 바인딩

### 8-3. 테스트 ✅
- [x] `runs_foreach_basic` — 기본 목록 순회 + 출력
- [x] `runs_foreach_sum` — 합계 계산 (가변 외부 변수 수정)
- [x] `runs_foreach_empty_list` — 빈 목록 → 0회 반복
- [x] `runs_foreach_scope_isolation` — 반복 변수 스코프 격리 (외부 동명 변수 미영향)
- [x] `runs_foreach_mutable_outer` — 외부 가변 변수 수정 가능
- [x] `runs_foreach_nested` — 이중 for-each
- [x] `rejects_foreach_non_list` — 비목록 → 런타임 에러
- [x] `parses_foreach` — AST 구조 검증
- [x] `explore_foreach_sum`에서 `#[ignore]` 제거 → 통과
- [x] 전체 233 테스트 통과, 0 ignored, 회귀 0

### 8-4. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `foreach_stmt` 규칙 + `EACH`, `ABOUT` 토큰 정의
- [x] `docs/LANGUAGE.md` — for-each 반복문 섹션 추가
- [x] `docs/DECISIONS.md` — "for-each 반복문: `각각 ... 에 대해` 채택" 기록

## Plan 9: 존재 바인딩 (`X에(는) Y가/이 있다`) ✅

### 9-1. Phase 1: MVP — `X에 Y가/이 있다` ✅
- [x] `src/token.rs`: `Exist` variant 추가
- [x] `src/lexer.rs`: `있다` → Exist 매핑
- [x] `src/normalizer.rs`: `is_standalone_subject_particle` 확장 (Exist 꼬리 허용)
- [x] `src/parser.rs`: Locative 분기에 Subject + Exist → `Bind { mutable: false }` 경로 추가

### 9-2. Phase 2: Polish — `X에는 Y가/이 있다` ✅
- [x] `src/normalizer.rs`: `split_locative_before_topic` + `line_has_bind_tail` 추가
- [x] `src/parser.rs`: Locative 뒤 선택적 Topic 소비 (에는 형태 허용)

### 9-3. 테스트 ✅
- [x] `runs_exist_binding_basic` — `바구니에 [1,2,3]이 있다` → 출력 확인
- [x] `runs_exist_binding_with_ga` — `상자에 "보물"가 있다` (가 particle)
- [x] `rejects_exist_binding_reassign` — const 체크
- [x] `runs_exist_binding_with_foreach` — `있다` + for-each 조합
- [x] `runs_exist_binding_with_topic` — `에는` 형태
- [x] `runs_mutable_bind_with_topic` — `에는` 가변 바인딩 부수 효과
- [x] `parses_exist_binding` — AST 구조 검증
- [x] 전체 240 테스트 통과, 0 ignored, 회귀 0

### 9-4. 문서 갱신 ✅
- [x] `docs/GRAMMAR.ebnf` — `exist_bind_stmt` 규칙 + `EXIST` 토큰 정의
- [x] `docs/LANGUAGE.md` — 존재 바인딩 섹션 추가
- [x] `docs/DECISIONS.md` — "존재 바인딩: `있다` = const syntactic sugar 채택" 기록

## Plan 10: v0.3 Quality Gate ✅

플랜: `.claude/plans/plan10-quality-gate.md`

### Phase 1: 테스트 보강 ✅
- [x] 샘플 통합 테스트 (19개 .zm 파일) + 깨진 샘플 3개 수정 (이다→넣는다)
- [x] LexError 3종 테스트 (UnterminatedString, UnexpectedCharacter, InconsistentDedent)
- [x] Choose 프레임 테스트 (기본, effect, 빈 목록, 비목록)
- [x] Named call 에러 경로 테스트 (builtin 대상, 누락 파라미터, 초과 키)
- [x] 빌트인 에러 경로 테스트 (정수로, 실수로, 맨뒤 꺼내기)
- [x] CLI ast 서브커맨드 테스트

### Phase 2: 유령 기능 문서화 + 누락 샘플 ✅
- [x] 타입 변환 빌트인 문서 + 샘플 (`문자열로`/`정수로`/`실수로`)
- [x] v0.3 누락 샘플 (넣는다/이다, inline if-else)

### Phase 3: DX 개선 ✅
- [x] `--version` / `--help` CLI 플래그
- [x] REPL 히스토리 영속화 (`~/.ziium/history`)
- [x] 낡은 주석 제거 + Clippy 수정
