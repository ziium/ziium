# Message Model

이 문서는 지음의 v0.2 이후 문법 연구를 위해 메시지 중심 설계안을 정리한다.
v0.1은 함수 호출, 속성 접근, 기호 연산을 별개로 다루지만, 한국어 문장형 확장을 더 밀어붙이려면 이 셋을 한 축으로 묶는 편이 더 일관적일 수 있다.

현재 구현의 구체 해석 우선순위는 `MESSAGE_BOUNDARIES.md`를 따른다.

## 문제 인식

현재 지음은 아래 세 가지를 서로 다른 범주로 취급한다.

- `길이(과일들)` 같은 내장 함수 호출
- `사람의 이름` 같은 `의` 속성 접근
- `7 + 8` 같은 기호 연산

하지만 자연스러운 한국어형 표면 문법을 더 밀어붙이면 아래와 같은 표현이 다시 등장한다.

```txt
합은 7 더하기 8이다.
과일들에 "감" 추가
과일들의 길이
```

이 표현들을 단순 sugar로만 취급하면 문법이 조각나기 쉽다. 반대로 "모든 계산은 수신자에게 메시지를 보내는 일"이라고 보면, 위 셋은 같은 의미층으로 정리된다.

## 핵심 주장

지음의 v0.2 연구 축으로 아래 의미 모델을 검토한다.

```txt
Send {
  receiver,
  selector,
  args
}
```

즉:

- `과일들의 길이`는 `과일들`이 `길이` 메시지를 받는 unary message
- `과일들에 "감" 추가`는 `과일들`이 `"감"`를 받아 `추가`를 수행하는 keyword message
- `7 더하기 8`은 `7`이 `더하기` 메시지를 받아 `8`을 인수로 받는 binary word message

이 모델은 "함수 호출의 보기 좋은 표기"보다 더 강한 주장이다. 함수, 속성, 연산자를 모두 message send의 다른 표면형으로 본다.

## 연구 배경

메시지 중심 언어 계열은 이미 선례가 있다.

- Smalltalk는 메시지를 unary, binary, keyword 세 종류로 구분한다.
- Self는 표현식을 메시지 전송 중심으로 설명한다.
- Newspeak는 메시지 전송을 언어 핵심 연산으로 둔다.
- Io는 연산자 자체를 메시지 트리로 변환한다.
- Objective-C는 dot notation이 accessor sugar일 뿐이고, 핵심은 동적 메시지 전송이라고 설명한다.

지음은 이 계열을 그대로 복제하려는 것이 아니라, 한국어 조사와 어순을 활용해 메시지 표면 문법을 재설계하려는 것이다.

## 지음에 맞는 메시지 분류

### 1. Unary noun message

가장 먼저 검토할 형태다.

```txt
과일들의 길이
사용자의 이름
문장의 첫글자
```

권장 해석:

- `<표현식>의 <명사>`는 unary message send
- selector는 명사형으로 고정한다

초기에는 아래처럼 제한하는 것이 안전하다.

- 레코드 키 조회
- 목록/문자열의 `길이`
- 목록의 `첫째`, `마지막`

즉 `사람의 이름`과 `과일들의 길이`를 같은 문법 범주로 읽고, dispatch에서만 차이를 둔다.

### 2. Binary word message

산술과 비교 일부를 한국어형으로 옮길 때 쓸 수 있다.

```txt
7 더하기 8
점수 크다 10
나이 같다 20
```

하지만 이 범주는 가장 조심해야 한다.

- 우선순위와 결합성 정의가 필요하다
- ordinary identifier와 operator word가 충돌하기 쉽다
- 모든 단어를 infix로 풀어주면 parser가 빠르게 흔들린다

초기 연구 범위는 매우 좁게 잡는다.

- `더하기`
- `빼기`
- `곱하기`
- `나누기`

즉 binary word message는 당분간 built-in 작은 집합으로만 취급한다.

### 3. Keyword verb message

가장 한국어답지만 가장 비싼 축이다.

```txt
과일들에 "감" 추가
문장에 ","로 나누기
파일에 내용 쓰기
```

이 형태는 단순 함수 호출 sugar가 아니라 동사별 인수 틀을 언어가 알아야 한다. 지음에서는 이를 임시로 "조사 프레임"이라 부른다.

예:

```txt
<수신자>에 <값> 추가
<수신자>를 <값>로 바꾸기
<수신자>에서 <값> 찾기
```

이 축은 자유 자연어로 가면 안 된다. v0.2 연구에서는 아래처럼 제한한다.

- 어순은 canonical form만 허용
- 조사 생략 금지
- built-in 메시지 몇 개만 허용
- 사용자 정의 verb message는 후순위

## 의미 모델

메시지 중심으로 재구성하면 값 모델은 "무엇이 무엇을 이해하는가"로 정리된다.

예시:

- 숫자: `더하기`, `빼기`, `곱하기`, `나누기`
- 문자열: `길이`, `나누기`
- 목록: `길이`, `추가`, `첫째`, `마지막`
- 레코드: 키 조회용 unary message
- 함수: `호출` 가능한 값 또는 별도 callable protocol

즉 "리스트에 `추가` 메시지를 보낸다"와 "숫자에 `더하기` 메시지를 보낸다"를 같은 dispatch 계층으로 다룬다.

## AST 방향

v0.1 AST는 `Property`, `Call`, `Binary`를 따로 둔다. 메시지 중심 모델을 진지하게 검토한다면 v0.2 이상에서는 아래와 같은 통합 노드가 더 적합하다.

```txt
Expr =
  ...
  Send {
    receiver: Expr,
    selector: Selector,
    args: [Expr]
  }
```

`Selector`는 표면 문법 분류를 반영할 수 있다.

```txt
Selector =
  UnaryNoun(name)
  BinaryWord(name)
  KeywordFrame(parts)
```

이렇게 하면:

- `사람의 이름`
- `7 더하기 8`
- `과일들에 "감" 추가`

를 같은 의미 노드로 내릴 수 있다.

## 파싱 전략

메시지 중심으로 가더라도 parser는 자연어 parser가 되면 안 된다. 권장 전략은 아래와 같다.

1. unary noun message를 가장 먼저 확장한다
2. binary word message는 기호 연산과 같은 precedence table에 넣는다
3. keyword verb message는 statement-level에서만 먼저 허용한다
4. free word order는 금지하고 canonical order만 허용한다

초기 우선순위 예시:

1. postfix: 호출, 인덱싱, unary noun message
2. unary prefix
3. multiplicative
4. additive
5. binary word message
6. 비교/동등성
7. 논리

실제 순서는 실험이 필요하다. 중요한 점은 "메시지 문법을 넣더라도 우선순위 표를 포기하지 않는다"는 것이다.

## 현재 구현과의 접점

현재 프로토타입 기준으로 보면 세 후보의 비용은 다르다.

- `과일들의 길이`: 이미 `의` property syntax가 있어 가장 싸다. 해석 계층만 넓히면 된다.
- `7 더하기 8`: expression parser에 word operator를 넣어야 한다.
- `과일들에 "감" 추가`: statement grammar와 built-in dispatch 체계를 함께 넓혀야 한다.

즉 연구 순서는 아래가 적절하다.

1. unary noun message
2. built-in binary word message
3. built-in keyword verb message
4. 사용자 정의 메시지 프로토콜

## 비목표

이 문서는 아래를 당장 목표로 하지 않는다.

- actor model 기반 비동기 메시지 시스템
- 클래스 중심 OOP 강제
- 자유 자연어 파싱
- 조사 생략 허용
- 모든 식별자의 infix/verb 사용 허용

즉 "메시지 중심"은 "모든 한국어 문장을 코드로 받는다"는 뜻이 아니다.

## 장단점

장점:

- 함수/속성/연산자를 하나의 의미축으로 묶을 수 있다
- 지음의 한국어 문장성에 더 잘 맞는다
- `목록의 길이`, `목록에 값 추가` 같은 문법을 ad hoc sugar보다 일관되게 설명할 수 있다

단점:

- selector namespace 설계가 중요해진다
- built-in과 user-defined message의 경계가 어려워진다
- 조사와 어순이 parser 복잡도를 밀어올린다
- 잘못 넓히면 controlled language가 아니라 자연어 parser 비슷한 것이 된다

## 권장 다음 단계

1. `목록의 길이`, `문장의 길이`를 unary message lowering 후보로 문서화
2. `더하기/빼기/곱하기/나누기`를 built-in binary word message 후보로 정리
3. `목록에 값 추가`를 built-in keyword message 시범 문법으로 설계
4. parser/AST 실험 브랜치에서 `Send` 노드 초안 검토

## 참고 자료

- GNU Smalltalk syntax: https://www.gnu.org/software/smalltalk/manual/html_node/The-syntax.html
- Self language reference: https://handbook.selflanguage.org/2024.1/langref.html
- Newspeak overview: https://newspeaklanguage.org/index.html
- Newspeak by Example: https://newspeaklanguage.org/samples/Literate/literate.html
- Io guide: https://iolanguage.org/guide/guide.html
- Apple Objective-C messaging: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjectiveC/Chapters/ocObjectsClasses.html
