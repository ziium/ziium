import init, { run_source_web, ziium_version } from "./pkg/ziium.js";

const demos = {
  "text-hanoi": {
    title: "텍스트 하노이탑",
    summary: "가장 단순한 재귀 하노이탑 예제입니다. 출력에는 이동 경로만 나옵니다.",
    renderer: "text",
    discCount: 3,
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
      "지음다운 문장형 스타일로 같은 하노이탑을 실행하고, 출력된 이동 경로를 캔버스로 그립니다.",
    renderer: "canvas",
    discCount: 3,
    source: `이동문장만들기 함수는 이동객체를 받아
  이동객체의 시작 + " -> " + 이동객체의 목표를 돌려준다.

탑옮기기 함수는 원반수, 시작, 보조, 목표를 받아
  원반수 == 1이면
    이동객체는 { 시작: 시작, 목표: 목표 }이다.
    이동객체로 이동문장만들기를 출력한다.
  아니면
    남은원반수는 원반수 빼기 1이다.
    탑옮기기(남은원반수, 시작, 목표, 보조)
    이동객체는 { 시작: 시작, 목표: 목표 }이다.
    이동객체로 이동문장만들기를 출력한다.
    탑옮기기(남은원반수, 보조, 시작, 목표)

탑옮기기(3, "A", "B", "C")
`,
  },
};

const pegNames = ["A", "B", "C"];

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
  runCurrentDemo();
}

function bindEvents() {
  for (const button of demoButtons) {
    button.addEventListener("click", () => {
      loadDemo(button.dataset.demo);
      runCurrentDemo();
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
  canvasPanel.classList.toggle("is-hidden", demo.renderer !== "canvas");

  for (const button of demoButtons) {
    button.classList.toggle("is-active", button.dataset.demo === demoId);
  }

  if (demo.renderer !== "canvas") {
    stopAnimation();
    clearCanvas();
    canvasStatus.textContent = "캔버스 데모가 선택되면 여기에서 하노이탑을 그립니다.";
  }
}

function runCurrentDemo() {
  stopAnimation();
  const demo = demos[currentDemoId];
  const result = run_source_web(editor.value);

  if (result.ok) {
    const text = result.output.trim();
    output.textContent = text === "" ? "(출력 없음)" : text;
    statusText.textContent = `실행 성공: ${countNonEmptyLines(result.output)}줄 출력`;

    if (demo.renderer === "canvas") {
      renderHanoiFromOutput(result.output, demo.discCount);
    }
    return;
  }

  output.textContent = result.error;
  statusText.textContent = "실행 오류";
  clearCanvas();
  if (demo.renderer === "canvas") {
    canvasStatus.textContent = "오류 때문에 캔버스를 그리지 못했습니다.";
  }
}

function renderHanoiFromOutput(outputText, discCount) {
  const moves = parseMoves(outputText);
  if (moves.length === 0) {
    clearCanvas();
    canvasStatus.textContent = "이동 경로를 찾지 못했습니다.";
    return;
  }

  const towers = initialTowers(discCount);
  drawTowers(towers, 0, moves.length);
  canvasStatus.textContent = `0 / ${moves.length} 이동`;

  let step = 0;
  animationTimer = window.setInterval(() => {
    if (step >= moves.length) {
      stopAnimation();
      canvasStatus.textContent = `완료: ${moves.length}번 이동`;
      return;
    }

    applyMove(towers, moves[step]);
    step += 1;
    drawTowers(towers, step, moves.length);
    canvasStatus.textContent = `${step} / ${moves.length} 이동`;
  }, 650);
}

function parseMoves(outputText) {
  return outputText
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const match = /^([ABC])\s*->\s*([ABC])$/.exec(line);
      if (!match) {
        return null;
      }

      return {
        from: pegNames.indexOf(match[1]),
        to: pegNames.indexOf(match[2]),
      };
    })
    .filter(Boolean);
}

function initialTowers(discCount) {
  return [
    Array.from({ length: discCount }, (_, index) => discCount - index),
    [],
    [],
  ];
}

function applyMove(towers, move) {
  const disc = towers[move.from].pop();
  if (disc == null) {
    throw new Error("빈 기둥에서 원반을 꺼내려 했습니다.");
  }
  towers[move.to].push(disc);
}

function drawTowers(towers, step, totalSteps) {
  const width = canvas.width;
  const height = canvas.height;
  ctx.clearRect(0, 0, width, height);

  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "#f8f1dc");
  gradient.addColorStop(1, "#d9e4ff");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);

  ctx.fillStyle = "#3c3328";
  ctx.fillRect(80, height - 46, width - 160, 12);

  const pegXs = [width * 0.22, width * 0.5, width * 0.78];
  const baseY = height - 46;
  const discHeight = 26;
  const maxDiscWidth = 160;
  const minDiscWidth = 62;
  const palette = ["#d94841", "#f08c00", "#33658a", "#5c7cfa", "#7f5539"];
  const maxDisc = towers.reduce(
    (largest, tower) => Math.max(largest, ...tower, 0),
    0,
  );

  ctx.textAlign = "center";
  ctx.font = '600 18px "Iowan Old Style", "AppleMyungjo", serif';
  ctx.fillStyle = "#443a30";

  pegXs.forEach((x, index) => {
    ctx.fillRect(x - 5, height * 0.16, 10, baseY - height * 0.16);
    ctx.fillText(pegNames[index], x, height * 0.12);
  });

  towers.forEach((tower, towerIndex) => {
    tower.forEach((disc, discIndex) => {
      const y = baseY - discHeight * (tower.length - discIndex);
      const widthRatio = maxDisc === 1 ? 1 : (disc - 1) / (maxDisc - 1);
      const discWidth = minDiscWidth + widthRatio * (maxDiscWidth - minDiscWidth);
      const x = pegXs[towerIndex] - discWidth / 2;

      ctx.fillStyle = palette[(disc - 1) % palette.length];
      roundRect(ctx, x, y, discWidth, discHeight - 4, 10);
      ctx.fill();

      ctx.fillStyle = "#fffaf0";
      ctx.font = '700 14px "IBM Plex Sans KR", "Apple SD Gothic Neo", sans-serif';
      ctx.fillText(String(disc), pegXs[towerIndex], y + 15);
    });
  });

  ctx.textAlign = "left";
  ctx.font = '600 16px "IBM Plex Sans KR", "Apple SD Gothic Neo", sans-serif';
  ctx.fillStyle = "#3b2f2f";
  ctx.fillText(`하노이탑 진행: ${step} / ${totalSteps}`, 26, 32);
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
