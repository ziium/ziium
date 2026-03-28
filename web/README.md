# 웹 playground

브라우저용 데모는 `web/` 아래에 있다.

- 텍스트 하노이탑: 이동 경로를 그대로 출력
- 캔버스 하노이탑: 지음 코드가 `그림판에 { ... }으로/로 ...`와 `0.5초 쉬기`를 직접 호출해서 만든 프레임을 브라우저 캔버스에 재생
- 숲속의 용사 (`story.html`): 한국어 비교, 상대적 변화, 선택 프레임을 사용한 인터랙티브 텍스트 어드벤처. Scene-restart 모델로 동작하며, 선택지 버튼을 누르면 이전 선택을 큐에 넣고 처음부터 재실행한다.

## 준비

1. `rustup target add wasm32-unknown-unknown`
2. `cargo install wasm-pack`

## 빌드

```bash
wasm-pack build --target web --out-dir web/pkg
```

## 실행

정적 파일 서버로 `web/`를 열면 된다.

```bash
python3 -m http.server 8000 --directory web
```

그 다음 브라우저에서 `http://localhost:8000`을 열면 된다.
