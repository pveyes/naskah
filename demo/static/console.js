// Runs the transpiled JavaScript in a throwaway Web Worker (worker.js) and prints
// what it logs. A worker has no DOM access and can be killed, so an infinite
// `ulang { }` never freezes the page.

import { parseMap, runProgram, TIME_LIMIT_MS } from "./runner.js";

const DEBOUNCE_MS = 300;

const output = document.getElementById("console-output");
const status = document.getElementById("console-status");

let stopProgram = null;
let runId = 0;

function clear() {
  output.replaceChildren();
  hideBand();
}

function addLine(level, text, sourceLine, column) {
  const line = document.createElement("div");
  line.className = "console-line console-" + level;

  const message = document.createElement("span");
  message.className = "console-text";
  message.textContent = text;
  line.appendChild(message);

  if (sourceLine) {
    line.dataset.site = sourceLine;
    line.addEventListener("mouseenter", () => showBand(sourceLine));
    line.addEventListener("mouseleave", hideBand);

    const site = document.createElement("span");
    site.className = "console-site";
    site.textContent = "naskah.nsk:" + sourceLine + (column ? ":" + column : "");
    line.appendChild(site);
  }
  output.appendChild(line);
}

// Highlighting the Naskah line a log line came from.
// The editor is rendered by wasm, so bands are drawn in a separate layer
// instead of inside it.
const bands = document.getElementById("line-bands");

/** Client rects (one per visual row) covering `lineNumber` of `root`'s text. */
function lineRows(root, lineNumber) {
  const lines = root.textContent.split("\n");
  if (lineNumber < 1 || lineNumber > lines.length) return [];

  let start = 0;
  for (let i = 0; i < lineNumber - 1; i++) start += lines[i].length + 1;
  const end = start + lines[lineNumber - 1].length;
  if (end === start) return [];

  const range = document.createRange();
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let position = 0;
  let started = false;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const length = node.nodeValue.length;
    if (!started && start < position + length) {
      range.setStart(node, start - position);
      started = true;
    }
    if (started && end <= position + length) {
      range.setEnd(node, end - position);
      break;
    }
    position += length;
  }

  // a line made of several spans yields several rects per row; keep one per row
  const rows = new Map();
  for (const rect of range.getClientRects()) {
    const key = Math.round(rect.top);
    if (!rows.has(key)) rows.set(key, rect);
  }
  return [...rows.values()];
}

function showBand(naskahLine) {
  hideBand();
  const backdrop = document.querySelector("#playground .backdrop");
  const editor = backdrop?.closest(".editor");
  if (!backdrop || !editor) return;

  const origin = bands.getBoundingClientRect();
  const box = editor.getBoundingClientRect();
  const lineHeight = parseFloat(getComputedStyle(backdrop).lineHeight);

  for (const rect of lineRows(backdrop, naskahLine)) {
    const height = Number.isFinite(lineHeight) ? lineHeight : rect.height;
    const band = document.createElement("div");
    band.className = "line-band";
    band.style.top = rect.top + rect.height / 2 - height / 2 - origin.top + "px";
    band.style.left = box.left - origin.left + "px";
    band.style.width = box.width + "px";
    band.style.height = height + "px";
    bands.appendChild(band);
  }
}

function hideBand() {
  bands.replaceChildren();
}

function placeholder(text) {
  clear();
  addLine("note", text);
}

function stop() {
  if (stopProgram) stopProgram();
  stopProgram = null;
}

function run(js) {
  stop();
  const id = ++runId;
  clear();
  status.textContent = "Menjalankan…";

  let printed = 0;
  const { sites, lines } = parseMap(document.getElementById("sourcemap")?.textContent ?? "");

  stopProgram = runProgram(
    { js, sites, lines },
    {
      line(msg) {
        if (id !== runId) return;
        printed += 1;
        addLine(msg.level, msg.text, msg.line);
        output.scrollTop = output.scrollHeight;
      },
      done(msg) {
        if (id !== runId) return;
        if (printed === 0) {
          placeholder("Tidak ada keluaran. Gunakan tulis(...) untuk mencetak.");
        }
        status.textContent = "Selesai dalam " + Math.max(1, Math.round(msg.ms)) + " ms";
      },
      timeout() {
        if (id !== runId) return;
        addLine("error", "Dihentikan: kode berjalan lebih dari " + TIME_LIMIT_MS / 1000 + " detik.");
        status.textContent = "Dihentikan";
      },
      crash(message) {
        if (id !== runId) return;
        addLine("error", "Galat: " + message);
        status.textContent = "Galat";
      }
    }
  );
}

function update() {
  const js = document.getElementById("js")?.innerText ?? "";
  const mistake = /^\/\/ Salah sintaks di baris (\d+), kolom (\d+): (.*)/.exec(js);
  if (mistake) {
    // the student never sees the JavaScript pane, so the mistake is shown here
    stop();
    runId += 1;
    clear();
    addLine("error", "Salah tulis: " + mistake[3], Number(mistake[1]), Number(mistake[2]));
    status.textContent = "Ada yang salah";
    return;
  }
  if (js.trim() === "") {
    stop();
    runId += 1;
    placeholder("Tulis kode Naskah di atas untuk mulai.");
    status.textContent = "Menunggu";
    return;
  }
  run(js);
}

let pending = null;
const schedule = () => {
  clearTimeout(pending);
  pending = setTimeout(update, DEBOUNCE_MS);
};

// A link can open the playground with a program already in the editor: /#kode=...
// The editor is made by wasm a moment after this script runs, so wait for it.
function loadFromLink(tries = 100) {
  const code = new URLSearchParams(location.hash.slice(1)).get("kode");
  if (code === null) return;
  const box = document.querySelector("#playground textarea");
  if (!box) {
    if (tries > 0) setTimeout(() => loadFromLink(tries - 1), 50);
    return;
  }
  box.value = code;
  box.dispatchEvent(new Event("input", { bubbles: true }));
}
loadFromLink();
window.addEventListener("hashchange", () => loadFromLink());

// The generated JavaScript is hidden unless asked for.
const playground = document.getElementById("playground");
const jsToggle = document.getElementById("js-toggle");

function showJavaScript(visible) {
  playground.dataset.js = visible ? "shown" : "hidden";
  jsToggle.textContent = visible ? "Sembunyikan JavaScript" : "Lihat JavaScript";
  jsToggle.setAttribute("aria-pressed", String(visible));
  try {
    localStorage.setItem("naskah-javascript", visible ? "1" : "0");
  } catch {}
}

jsToggle.addEventListener("click", () => showJavaScript(playground.dataset.js === "hidden"));
try {
  if (localStorage.getItem("naskah-javascript") === "1") showJavaScript(true);
} catch {}

// The playground is rendered by wasm, so watch its output pane for changes.
new MutationObserver(schedule).observe(document.getElementById("playground"), {
  childList: true,
  subtree: true,
  characterData: true,
});
schedule();
