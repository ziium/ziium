# 웹 playground

브라우저용 데모는 `web/` 아래에 있다. 현재는 두 가지 하노이탑 데모를 제공한다.

- 텍스트 하노이탑: 이동 경로를 그대로 출력
- 캔버스 하노이탑: 이동 경로를 해석해서 브라우저 캔버스에 그림

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
