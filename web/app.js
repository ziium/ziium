import init, { run_source_web } from "./pkg/ziium.js";

const demos = {
  "text-hanoi": {
    title: "텍스트 하노이탑",
    summary: "가장 단순한 재귀 하노이탑 예제입니다. 이동 경로를 텍스트로만 출력합니다.",
    renderer: "text",
    source: `탑옮기기 함수는 원반수, 시작, 보조, 목표를 받아
  원반수 == 1이면
    시작 + " -> " + 목표를 출력한다.
    0.5초 쉬기.
  아니면
    탑옮기기(원반수 - 1, 시작, 목표, 보조)
    시작 + " -> " + 목표를 출력한다.
    0.5초 쉬기.
    탑옮기기(원반수 - 1, 보조, 시작, 목표)

탑옮기기(3, "A", "B", "C")
`,
  },
  "canvas-hanoi": {
    title: "캔버스 하노이탑",
    summary:
      "지음 코드가 `그림판에 {...}으로/로 ...` 내장을 직접 호출해서 하노이탑 프레임을 만듭니다.",
    renderer: "canvas",
    source: `배경색은 "#f8f1dc"이다.
기둥색은 "#3c3328"이다.
글자색은 "#443a30"이다.
빨강은 "#d94841"이다.
주황은 "#f08c00"이다.
파랑은 "#33658a"이다.

원반색 함수는 원반을 받아
  원반 == 1이면
    빨강을 돌려준다.
  아니면
    원반 == 2이면
      주황을 돌려준다.
    아니면
      파랑을 돌려준다.

기둥그리기 함수는 탑, 이름, 중심점을 받아
  그림판에 { 글: 이름, x: 중심점 - 8, y: 62, 색: 글자색, 크기: 22 }로 글자쓰기.
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
  그림판에 { 배경색: 배경색 }으로 지우기.
  그림판에 { x: 80, y: 374, 너비: 800, 높이: 12, 색: 기둥색 }으로 사각형채우기.
  그림판에 { x: 205, y: 90, 너비: 10, 높이: 284, 색: 기둥색 }으로 사각형채우기.
  그림판에 { x: 475, y: 90, 너비: 10, 높이: 284, 색: 기둥색 }으로 사각형채우기.
  그림판에 { x: 745, y: 90, 너비: 10, 높이: 284, 색: 기둥색 }으로 사각형채우기.
  기둥그리기(탑A, "A", 210)
  기둥그리기(탑B, "B", 480)
  기둥그리기(탑C, "C", 750)
  0.5초 쉬기.

탑옮기기 함수는 개수, 시작탑, 보조탑, 목표탑을 받아
  개수 == 1이면
    원반은 시작탑에서 맨위 원반을 빼낸 것이다.
    목표탑에 원반 추가.
    장면그리기()
  아니면
    탑옮기기(개수 - 1, 시작탑, 목표탑, 보조탑)
    원반은 시작탑에서 맨위 원반을 빼낸 것이다.
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
const outputPanel = document.querySelector(".panel-output");
const demoTitle = document.querySelector("#demoTitle");
const demoSummary = document.querySelector("#demoSummary");
const statusText = document.querySelector("#statusText");
const canvasPanel = document.querySelector("#canvasPanel");
const canvasStatus = document.querySelector("#canvasStatus");
const canvasInfo = document.querySelector("#canvasInfo");
const canvas = document.querySelector("#hanoiCanvas");
const runButton = document.querySelector("#runButton");
const demoButtons = [...document.querySelectorAll("[data-demo]")];
const ctx = canvas.getContext("2d");

let currentDemoId = "text-hanoi";
let animationTimer = null;
let playbackGeneration = 0;

async function main() {
  await init();
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
    void runCurrentDemo();
  });
}

function loadDemo(demoId) {
  currentDemoId = demoId;
  const demo = demos[demoId];
  editor.value = demo.source;
  demoTitle.textContent = demo.title;
  demoSummary.textContent = demo.summary;
  output.textContent = "아직 실행하지 않았습니다.";
  canvasInfo.textContent = "아직 실행하지 않았습니다.";
  setRightPanel(demo.renderer);

  if (demo.renderer === "canvas") {
    canvasStatus.textContent = "실행 버튼을 누르면 캔버스 재생이 시작됩니다.";
  } else {
    statusText.textContent = "실행 버튼을 누르면 결과가 여기에 나옵니다.";
  }

  for (const button of demoButtons) {
    button.classList.toggle("is-active", button.dataset.demo === demoId);
  }

  stopAnimation();
  clearCanvas();
}

async function runCurrentDemo() {
  stopAnimation();
  const playbackId = playbackGeneration;
  const demo = demos[currentDemoId];
  const result = run_source_web(editor.value);

  if (!result.ok) {
    output.textContent = result.error;
    clearCanvas();
    if (demo.renderer === "canvas") {
      canvasStatus.textContent = "실행 오류";
      canvasInfo.textContent = result.error;
    } else {
      statusText.textContent = "실행 오류";
    }
    return;
  }

  const events = JSON.parse(result.execution_events_json || "[]");

  if (demo.renderer === "canvas") {
    await replayCanvasEvents(events, playbackId);
    return;
  }

  await replayTextEvents(events, playbackId);
}

function setRightPanel(renderer) {
  const showCanvas = renderer === "canvas";
  outputPanel.classList.toggle("is-hidden", showCanvas);
  canvasPanel.classList.toggle("is-hidden", !showCanvas);
}

async function replayTextEvents(events, playbackId) {
  const lines = [];
  output.textContent = "";
  statusText.textContent = "실행 중...";

  for (const event of events) {
    if (!isCurrentPlayback(playbackId)) {
      return;
    }

    if (event.kind === "Output") {
      lines.push(event.text);
      output.textContent = lines.join("\n");
      statusText.textContent = `실행 중: ${lines.length}줄 출력`;
      continue;
    }

    if (event.kind === "Sleep") {
      const stillCurrent = await waitForSeconds(event.seconds, playbackId);
      if (!stillCurrent) {
        return;
      }
    }
  }

  if (!isCurrentPlayback(playbackId)) {
    return;
  }

  output.textContent = lines.length === 0 ? "(출력 없음)" : lines.join("\n");
  statusText.textContent = `실행 성공: ${lines.length}줄 출력`;
}

async function replayCanvasEvents(events, playbackId) {
  const lines = [];
  const totalFrames = countCanvasFrames(events);
  let shownFrames = 0;

  clearCanvas();
  updateCanvasInfo(lines, totalFrames);
  canvasStatus.textContent =
    totalFrames === 0 ? "실행 성공: 프레임 없음" : `재생 중: 0 / ${totalFrames} 프레임`;

  for (const event of events) {
    if (!isCurrentPlayback(playbackId)) {
      return;
    }

    if (event.kind === "Output") {
      lines.push(event.text);
      updateCanvasInfo(lines, totalFrames);
      continue;
    }

    if (event.kind === "CanvasFrame") {
      drawFrame(event.frame);
      shownFrames += 1;
      canvasStatus.textContent = `재생 중: ${shownFrames} / ${totalFrames} 프레임`;
      updateCanvasInfo(lines, totalFrames);
      continue;
    }

    if (event.kind === "Sleep") {
      const stillCurrent = await waitForSeconds(event.seconds, playbackId);
      if (!stillCurrent) {
        return;
      }
    }
  }

  if (!isCurrentPlayback(playbackId)) {
    return;
  }

  canvasStatus.textContent =
    totalFrames === 0 ? "실행 성공: 프레임 없음" : `완료: ${totalFrames} 프레임`;
  updateCanvasInfo(lines, totalFrames);
}

function updateCanvasInfo(lines, totalFrames) {
  const outputText = lines.length === 0 ? "출력 없음" : lines.join("\n");
  canvasInfo.textContent = `${outputText}\n\n캔버스 프레임 ${totalFrames}개 생성`;
}

function countCanvasFrames(events) {
  return events.filter((event) => event.kind === "CanvasFrame").length;
}

function isCurrentPlayback(playbackId) {
  return playbackId === playbackGeneration;
}

function waitForSeconds(seconds, playbackId) {
  const ms = Math.max(0, seconds) * 1000;
  if (ms === 0) {
    return Promise.resolve(isCurrentPlayback(playbackId));
  }

  return new Promise((resolve) => {
    animationTimer = window.setTimeout(() => {
      animationTimer = null;
      resolve(isCurrentPlayback(playbackId));
    }, ms);
  });
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
  playbackGeneration += 1;
  if (animationTimer != null) {
    window.clearTimeout(animationTimer);
    animationTimer = null;
  }
}

main().catch((error) => {
  statusText.textContent = "초기화 실패";
  output.textContent = String(error);
  canvasStatus.textContent = "초기화 실패";
  canvasInfo.textContent = String(error);
});
