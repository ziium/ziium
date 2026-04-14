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

## Plan 2: 테스트 스위트 (5배치 × 10개 = 50개) ← 현재

- [x] 배치 1: 기능 테스트 — 연산자/조건문/반복문 (11개): `9280b15`
- [x] 배치 2: 기능 테스트 — 함수/문자열 (10개): `ffd3491`
- [x] 배치 3: 에러 테스트 — 타입 오류/인자/인덱스/빌트인 (9개)
- [x] 배치 4: 기능+조합 테스트 — 목록/레코드/논리/타입 (13개)
- [ ] 배치 5: 조합+탐색 테스트 — Rosetta (10개, 탐색은 `#[ignore]`)

배치 게이트: green ≥ 8/10, red 전부 backlog 항목 존재, 기존 회귀 0

## Plan 3: 기능 구현 (실패 테스트 기반)

- [ ] 실패 분석 → 기능 백로그 작성
- [ ] 설계 게이트 → 구현 → 회귀 확인
