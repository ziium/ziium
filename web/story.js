import init, { run_source_web_with_choices } from "./pkg/ziium.js";

const SOURCE = `# 숲속의 용사 — 인터랙티브 텍스트 어드벤처
# 한국어 비교(X가 Y보다 크면), 상대적 변화(X를 N만큼 줄인다),
# 선택 프레임(에서 고른 것이다)을 보여주는 샘플이다.

체력은 100이다.
장면은 1이다.

# 장면 1: 마을 출발
마을 함수는 아무것도 받지 않아
  "=== 마을 ==="을 출력한다.
  "용사여, 모험을 떠날 시간이다."를 출력한다.
  "체력:"을 출력한다.
  체력을 출력한다.
  ""을 출력한다.
  방향은 ["숲으로", "강으로"]에서 고른 것이다.
  방향이 "숲으로"보다 같으면
    장면을 2로 바꾼다.
  아니면
    장면을 4로 바꾼다.

# 장면 2: 숲 입구
숲 함수는 아무것도 받지 않아
  ""을 출력한다.
  "=== 숲 입구 ==="을 출력한다.
  "어두운 숲에 들어왔다. 앞에 늑대가 나타났다!"를 출력한다.
  ""을 출력한다.
  행동은 ["공격", "도망"]에서 고른 것이다.
  행동이 "공격"보다 같으면
    장면을 3으로 바꾼다.
  아니면
    "도망쳤다! 체력을 조금 잃었다."를 출력한다.
    체력을 10만큼 줄인다.
    장면을 4로 바꾼다.

# 장면 3: 전투
전투 함수는 아무것도 받지 않아
  ""을 출력한다.
  "=== 전투 ==="을 출력한다.
  "늑대와 싸운다!"를 출력한다.
  체력을 25만큼 줄인다.
  "늑대를 물리쳤다! 하지만 체력을 25 잃었다."를 출력한다.
  "체력:"을 출력한다.
  체력을 출력한다.
  체력이 40보다 작으면
    "경고: 체력이 위험하다!"를 출력한다.
  장면을 5로 바꾼다.

# 장면 4: 강가
강가 함수는 아무것도 받지 않아
  ""을 출력한다.
  "=== 강가 ==="을 출력한다.
  "넓은 강이 흐르고 있다. 낡은 다리가 보인다."를 출력한다.
  ""을 출력한다.
  행동은 ["다리를 건넌다", "돌아간다"]에서 고른 것이다.
  행동이 "다리를 건넌다"보다 같으면
    "다리가 흔들린다... 간신히 건넜다!"를 출력한다.
    체력을 15만큼 줄인다.
    장면을 5로 바꾼다.
  아니면
    "마을로 돌아왔다."를 출력한다.
    체력을 5만큼 늘린다.
    장면을 1로 바꾼다.

# 장면 5: 보물
보물 함수는 아무것도 받지 않아
  ""을 출력한다.
  "=== 보물 발견 ==="을 출력한다.
  "빛나는 보물 상자를 발견했다!"를 출력한다.
  장면을 6으로 바꾼다.

# 장면 6: 결말
결말 함수는 아무것도 받지 않아
  ""을 출력한다.
  "=== 결말 ==="을 출력한다.
  "최종 체력:"을 출력한다.
  체력을 출력한다.
  체력이 50보다 크면
    "용사는 당당히 마을로 돌아왔다. 승리!"를 출력한다.
  아니면
    "용사는 간신히 살아 돌아왔다..."를 출력한다.
  장면을 0으로 바꾼다.

# 게임 루프
장면 > 0인 동안
  장면 == 1이면
    마을()
  아니면
    장면 == 2이면
      숲()
    아니면
      장면 == 3이면
        전투()
      아니면
        장면 == 4이면
          강가()
        아니면
          장면 == 5이면
            보물()
          아니면
            결말()
`;

let choicesMade = [];

const storyOutput = document.getElementById("storyOutput");
const choicesArea = document.getElementById("choicesArea");
const statusMsg = document.getElementById("statusMsg");
const codeView = document.getElementById("codeView");

function renderOutput(events) {
  let html = "";
  let sceneCount = 0;
  for (const ev of events) {
    if (ev.kind === "Output") {
      const text = ev.text;
      if (text.startsWith("===") && text.endsWith("===")) {
        sceneCount++;
        html += `<span class="scene-title" id="scene-${sceneCount}">${escapeHtml(text)}</span>\n`;
      } else {
        html += escapeHtml(text) + "\n";
      }
    }
  }
  return { html, sceneCount };
}

function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function highlightCode(source) {
  return escapeHtml(source);
}

function runStory() {
  const choicesJson = JSON.stringify(choicesMade);
  const result = run_source_web_with_choices(SOURCE, choicesJson);

  const events = JSON.parse(result.execution_events_json || "[]");
  const choices = result.choices_json ? JSON.parse(result.choices_json) : [];

  const { html, sceneCount } = renderOutput(events);
  storyOutput.innerHTML = html;
  choicesArea.innerHTML = "";

  if (sceneCount > 0) {
    const scrollBox = document.getElementById("storyScroll");
    const anchor = document.getElementById(`scene-${sceneCount}`);
    if (scrollBox && anchor) {
      requestAnimationFrame(() => {
        scrollBox.scrollTop = anchor.offsetTop;
      });
    }
  }

  if (choices.length > 0) {
    statusMsg.textContent = "선택하세요:";
    for (const opt of choices) {
      const btn = document.createElement("button");
      btn.className = "choice-btn";
      btn.textContent = opt;
      btn.addEventListener("click", () => {
        choicesMade.push(opt);
        runStory();
      });
      choicesArea.appendChild(btn);
    }
  } else if (result.ok) {
    statusMsg.textContent = "이야기가 끝났습니다.";
    const btn = document.createElement("button");
    btn.className = "restart-btn";
    btn.textContent = "다시 시작";
    btn.addEventListener("click", () => {
      choicesMade = [];
      runStory();
    });
    choicesArea.appendChild(btn);
  } else {
    statusMsg.textContent = "오류: " + result.error;
  }
}

async function main() {
  await init();
  codeView.textContent = SOURCE;
  runStory();
}

main();
