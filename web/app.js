import init, { run_source_web, ziium_version } from "./pkg/ziium.js";

const demos = {
  "text-hanoi": {
    title: "텍스트 하노이탑",
    summary: "가장 단순한 재귀 하노이탑 예제입니다. 이동 경로를 텍스트로만 출력합니다.",
    renderer: "text",
    source: `탑옮기기 함수는 원반수, 시작, 보조, 목표를 받아
  원반수 == 1이면
    시작 + " -> " + 목표를 출력한다.
  아니면
    탑옮기기(원반수 - 1, 시작, 목표, 보조)
    시작 + " -> " + 목표를 출력한다.
    탑옮기기(원반수 - 1, 보조, 시작, 목표)

탑옮기기(3, "A", "B", "C")
`,
  },
  "canvas-hanoi": {
    title: "캔버스 하노이탑",
    summary:
      "지음 코드가 `그림판에 {...}으로/로 ...` 내장을 직접 호출해서 하노이탑 프레임을 만듭니다.",
    renderer: "canvas",
    source: `원반색 함수는 원반을 받아
  원반 == 1이면
    "#d94841"을 돌려준다.
  아니면
    원반 == 2이면
      "#f08c00"을 돌려준다.
    아니면
      "#33658a"을 돌려준다.

기둥그리기 함수는 탑, 이름, 중심점을 받아
  그림판에 { 글: 이름, x: 중심점 - 8, y: 62, 색: "#443a30", 크기: 22 }로 글자쓰기.
  인덱스는 0이다.
  원반은 0이다.
  너비는 0이다.
  왼쪽은 0이다.
  세로값은 0이다.
  인덱스 < (탑)의 길이인 동안
    원반을 탑[인덱스]로 바꾼다.
    너비를 52 + 원반 * 34로 바꾼다.
    왼쪽을 중심점 - (너비 / 2)로 바꾼다.
    세로값을 342 - (인덱스 + 1) * 28로 바꾼다.
    그림판에 { x: 왼쪽, y: 세로값, 너비: 너비, 높이: 24, 색: 원반색(원반) }으로 사각형채우기.
    인덱스를 인덱스 + 1로 바꾼다.

장면그리기 함수는 아무것도 받지 않아
  그림판에 { 배경색: "#f8f1dc" }으로 지우기.
  그림판에 { x: 80, y: 374, 너비: 800, 높이: 12, 색: "#3c3328" }으로 사각형채우기.
  그림판에 { x: 205, y: 90, 너비: 10, 높이: 284, 색: "#3c3328" }으로 사각형채우기.
  그림판에 { x: 475, y: 90, 너비: 10, 높이: 284, 색: "#3c3328" }으로 사각형채우기.
  그림판에 { x: 745, y: 90, 너비: 10, 높이: 284, 색: "#3c3328" }으로 사각형채우기.
  기둥그리기(탑A, "A", 210)
  기둥그리기(탑B, "B", 480)
  기둥그리기(탑C, "C", 750)

탑옮기기 함수는 개수, 시작탑, 보조탑, 목표탑을 받아
  개수 == 1이면
    원반은 마지막꺼내기(시작탑)이다.
    목표탑에 원반 추가.
    장면그리기()
  아니면
    탑옮기기(개수 - 1, 시작탑, 목표탑, 보조탑)
    원반은 마지막꺼내기(시작탑)이다.
    목표탑에 원반 추가.
    장면그리기()
    탑옮기기(개수 - 1, 보조탑, 시작탑, 목표탑)

탑A는 [3, 2, 1]이다.
탑B는 []이다.
탑C는 []이다.

장면그리기()
탑옮기기(3, 탑A, 탑B, 탑C)
`,
  },
};

const editor = document.querySelector("#editor");
const output = document.querySelector("#output");
const demoTitle = document.querySelector("#demoTitle");
const demoSummary = document.querySelector("#demoSummary");
const statusText = document.querySelector("#statusText");
const versionBadge = document.querySelector("#versionBadge");
const canvasPanel = document.querySelector("#canvasPanel");
const canvasStatus = document.querySelector("#canvasStatus");
const canvas = document.querySelector("#hanoiCanvas");
const runButton = document.querySelector("#runButton");
const demoButtons = [...document.querySelectorAll("[data-demo]")];
const ctx = canvas.getContext("2d");

let currentDemoId = "text-hanoi";
let animationTimer = null;

async function main() {
  await init();
  versionBadge.textContent = `ziium ${ziium_version()}`;
  bindEvents();
  loadDemo(currentDemoId);
}

function bindEvents() {
  for (const button of demoButtons) {
    button.addEventListener("click", () => {
      loadDemo(button.dataset.demo);
    });
  }

  runButton.addEventListener("click", () => {
    runCurrentDemo();
  });
}

function loadDemo(demoId) {
  currentDemoId = demoId;
  const demo = demos[demoId];
  editor.value = demo.source;
  demoTitle.textContent = demo.title;
  demoSummary.textContent = demo.summary;
  statusText.textContent = "실행 버튼을 누르면 결과가 여기에 나옵니다.";
  output.textContent = demo.renderer === "canvas"
    ? "이 데모는 출력보다 캔버스 프레임이 핵심입니다."
    : "아직 실행하지 않았습니다.";
  canvasPanel.classList.toggle("is-hidden", demo.renderer !== "canvas");

  for (const button of demoButtons) {
    button.classList.toggle("is-active", button.dataset.demo === demoId);
  }

  stopAnimation();
  clearCanvas();
  canvasStatus.textContent =
    demo.renderer === "canvas"
      ? "지음 코드가 만든 캔버스 프레임을 기다리는 중입니다."
      : "캔버스 데모가 선택되면 여기에서 하노이탑을 그립니다.";
}

function runCurrentDemo() {
  stopAnimation();
  const demo = demos[currentDemoId];
  const result = run_source_web(editor.value);

  if (!result.ok) {
    output.textContent = result.error;
    statusText.textContent = "실행 오류";
    clearCanvas();
    if (demo.renderer === "canvas") {
      canvasStatus.textContent = "오류 때문에 캔버스를 그리지 못했습니다.";
    }
    return;
  }

  const text = result.output.trim();
  const frames = JSON.parse(result.canvas_frames_json || "[]");

  if (demo.renderer === "canvas") {
    output.textContent =
      text === ""
        ? `출력 없음\n캔버스 프레임 ${frames.length}개 생성`
        : `${text}\n\n캔버스 프레임 ${frames.length}개 생성`;
    statusText.textContent = `실행 성공: 캔버스 프레임 ${frames.length}개`;
    renderCanvasFrames(frames);
    return;
  }

  output.textContent = text === "" ? "(출력 없음)" : text;
  statusText.textContent = `실행 성공: ${countNonEmptyLines(result.output)}줄 출력`;
}

function renderCanvasFrames(frames) {
  if (!Array.isArray(frames) || frames.length === 0) {
    clearCanvas();
    canvasStatus.textContent = "생성된 캔버스 프레임이 없습니다.";
    return;
  }

  let step = 0;
  drawFrame(frames[0]);
  canvasStatus.textContent = `1 / ${frames.length} 프레임`;

  animationTimer = window.setInterval(() => {
    step += 1;
    if (step >= frames.length) {
      stopAnimation();
      canvasStatus.textContent = `완료: ${frames.length} 프레임`;
      return;
    }

    drawFrame(frames[step]);
    canvasStatus.textContent = `${step + 1} / ${frames.length} 프레임`;
  }, 650);
}

function drawFrame(frame) {
  clearCanvas();
  for (const command of frame.commands) {
    drawCommand(command);
  }
}

function drawCommand(command) {
  switch (command.kind) {
    case "Clear": {
      ctx.fillStyle = command.background;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      return;
    }
    case "FillRect": {
      ctx.fillStyle = command.color;
      if (command.height <= 26) {
        roundRect(ctx, command.x, command.y, command.width, command.height, 10);
        ctx.fill();
      } else {
        ctx.fillRect(command.x, command.y, command.width, command.height);
      }
      return;
    }
    case "FillText": {
      ctx.fillStyle = command.color;
      ctx.textAlign = "left";
      ctx.textBaseline = "alphabetic";
      ctx.font = `700 ${command.size}px "IBM Plex Sans KR", "Apple SD Gothic Neo", sans-serif`;
      ctx.fillText(command.text, command.x, command.y);
      return;
    }
    default:
      return;
  }
}

function roundRect(context, x, y, width, height, radius) {
  context.beginPath();
  context.moveTo(x + radius, y);
  context.lineTo(x + width - radius, y);
  context.quadraticCurveTo(x + width, y, x + width, y + radius);
  context.lineTo(x + width, y + height - radius);
  context.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
  context.lineTo(x + radius, y + height);
  context.quadraticCurveTo(x, y + height, x, y + height - radius);
  context.lineTo(x, y + radius);
  context.quadraticCurveTo(x, y, x + radius, y);
  context.closePath();
}

function clearCanvas() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
}

function stopAnimation() {
  if (animationTimer != null) {
    window.clearInterval(animationTimer);
    animationTimer = null;
  }
}

function countNonEmptyLines(text) {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean).length;
}

main().catch((error) => {
  statusText.textContent = "초기화 실패";
  output.textContent = String(error);
});
