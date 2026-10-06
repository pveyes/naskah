// Runs the transpiled JavaScript in a throwaway Web Worker (worker.js) and prints
// what it logs. A worker has no DOM access and can be killed, so an infinite
// `ulang { }` never freezes the page.

import { parseMap, runProgram, TIME_LIMIT_MS } from "./runner.js";
import { addLine as addTerminalLine, ask } from "./terminal.js";
import { setupExamples } from "./contoh.js";

const DEBOUNCE_MS = 300;

const output = document.getElementById("console-output");
const status = document.getElementById("console-status");
const runButton = document.getElementById("console-run");

let stopProgram = null;
let runId = 0;

function clear() {
  output.replaceChildren();
  hideBand();
}

// the program is a file, and its name is on every line it prints
let currentFile = "naskah.nsk";

function addLine(level, text, sourceLine, column) {
  const row = addTerminalLine(output, level, text, sourceLine, column, currentFile);
  if (sourceLine) {
    row.addEventListener("mouseenter", () => showBand(sourceLine));
    row.addEventListener("mouseleave", hideBand);
  }
  return row;
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
      ask(question, reply) {
        if (id !== runId) return;
        status.textContent = "Menunggu jawabanmu";
        return ask(output, question, (text) => {
          status.textContent = "Menjalankan…";
          reply(text);
        });
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

function update(manual = false) {
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
  // a program that asks questions would ask them again at every change, so it waits to be started
  if (!manual && /\bprompt\(/.test(js)) {
    stop();
    runId += 1;
    placeholder("Program ini bertanya. Tekan Jalankan untuk memulai.");
    status.textContent = "Menunggu";
    return;
  }
  run(js);
}

let pending = null;
const schedule = () => {
  clearTimeout(pending);
  pending = setTimeout(() => update(), DEBOUNCE_MS);
};

runButton.addEventListener("click", () => {
  clearTimeout(pending);
  update(true);
});

// Put a program into the editor. The editor is made by wasm a moment after this script
// runs, so wait for it.
function setProgram(code, file, tries = 100) {
  const box = document.querySelector("#playground textarea");
  if (!box) {
    if (tries > 0) setTimeout(() => setProgram(code, file, tries - 1), 50);
    return;
  }
  currentFile = file;
  box.value = code;
  box.dispatchEvent(new Event("input", { bubbles: true }));
}

const examples = setupExamples(document.getElementById("examples"), (example) => {
  setProgram(example.kode, example.file);
});
// the editor starts with the first example
if (examples.first) {
  currentFile = examples.first.file;
  examples.markFirst();
  // the file tabs take over the job of the "Naskah" title above the editor
  document.getElementById("playground").dataset.tabs = "";
}

// A link can open the playground with a program already in the editor: /#kode=...
function loadFromLink() {
  const code = new URLSearchParams(location.hash.slice(1)).get("kode");
  if (code === null) return;
  examples.clear();
  setProgram(code, "naskah.nsk");
}
loadFromLink();
window.addEventListener("hashchange", () => loadFromLink());

// The generated JavaScript is hidden. A teacher can ask for it with /#javascript (the link on
// the page for teachers), which shows it beside the code.
const playground = document.getElementById("playground");

function showJavaScript() {
  const flags = new URLSearchParams(location.hash.slice(1));
  playground.dataset.js = flags.has("javascript") ? "shown" : "hidden";
}
showJavaScript();
window.addEventListener("hashchange", showJavaScript);

// The playground is rendered by wasm, so watch its output pane for changes.
new MutationObserver(schedule).observe(document.getElementById("playground"), {
  childList: true,
  subtree: true,
  characterData: true,
});
schedule();
