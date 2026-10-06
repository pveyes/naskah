// Runs the transpiled JavaScript in a throwaway Web Worker and prints what it
// logs. A worker has no DOM access and can be killed, so an infinite `ulang { }`
// never freezes the page.

const TIME_LIMIT_MS = 2000;
const MAX_LINES = 200;
const DEBOUNCE_MS = 300;

const WORKER_SOURCE = `
  const show = (v, top, seen = []) => {
    if (typeof v === "string") return top ? v : JSON.stringify(v);
    if (v === null || v === undefined) return "kosong";
    if (v === true) return "benar";
    if (v === false) return "salah";
    if (typeof v === "function") return "[fungsi]";
    if (typeof v === "object") {
      if (seen.includes(v) || seen.length > 3) return "[...]";
      const next = seen.concat([v]);
      if (Array.isArray(v)) return "[" + v.map((x) => show(x, false, next)).join(", ") + "]";
      const keys = Object.keys(v);
      if (!keys.length) return "{}";
      return "{ " + keys.map((k) => k + ": " + show(v[k], false, next)).join(", ") + " }";
    }
    return String(v);
  };

  let lines = 0;
  const post = (level, text, line) => {
    lines += 1;
    if (lines <= ${MAX_LINES}) postMessage({ type: "line", level, text, line });
    else if (lines === ${MAX_LINES} + 1) postMessage({ type: "line", level: "note", text: "Keluaran dipotong setelah ${MAX_LINES} baris." });
  };
  const format = (args) => args.map((a) => show(a, true)).join(" ");
  const print = (level) => (...args) => post(level, format(args));
  const sandbox = { log: print("log"), error: print("error"), warn: print("error") };
  // tulis(...) calls are rewritten to __log(naskahLine, ...) before running
  const __log = (line, ...args) => post("log", format(args), line);
  const prompt = () => {
    post("error", "tanya() belum didukung di sini dan selalu mengembalikan kosong.");
    return null;
  };

  const describe = (e) => {
    if (e instanceof ReferenceError) {
      const m = /^(\\S+) is not defined/.exec(e.message);
      if (m) return "\`" + m[1] + "\` belum dibuat. Buat dulu dengan misal " + m[1] + " = ...;";
    }
    return e && e.message ? e.message : show(e, true);
  };

  // top-level tunggu needs an async function around the program
  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;

  self.addEventListener("unhandledrejection", (event) => {
    post("error", "Galat: " + describe(event.reason));
  });

  onmessage = async (event) => {
    const started = performance.now();
    try {
      const program = new AsyncFunction("console", "prompt", "__log", '"use strict";\\n' + event.data);
      await program(sandbox, prompt, __log);
    } catch (e) {
      post("error", "Galat: " + describe(e));
    }
    postMessage({ type: "done", ms: performance.now() - started });
  };
`;

const workerUrl = URL.createObjectURL(new Blob([WORKER_SOURCE], { type: "text/javascript" }));

const output = document.getElementById("console-output");
const status = document.getElementById("console-status");

let worker = null;
let timer = null;
let runId = 0;

function clear() {
  output.replaceChildren();
  hideBand();
}

function addLine(level, text, sourceLine) {
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
    site.textContent = "naskah.nsk:" + sourceLine;
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

// "3:12,7:20" -> Map { 3 => 12, 7 => 20 } (JS line -> Naskah line)
function readCallSites() {
  const text = document.getElementById("sourcemap")?.textContent ?? "";
  const sites = new Map();
  for (const pair of text.split(",")) {
    const [jsLine, naskahLine] = pair.split(":").map(Number);
    if (jsLine && naskahLine) sites.set(jsLine, naskahLine);
  }
  return sites;
}

// Route each tulis(...) through __log(naskahLine, ...) so a log line knows where it came from.
function instrument(js, sites) {
  return js
    .split("\n")
    .map((text, index) => {
      const naskahLine = sites.get(index + 1);
      return naskahLine ? text.replace("console.log(", "__log(" + naskahLine + ", ") : text;
    })
    .join("\n");
}

function placeholder(text) {
  clear();
  addLine("note", text);
}

function stop() {
  if (worker) worker.terminate();
  worker = null;
  clearTimeout(timer);
}

function run(js) {
  stop();
  const id = ++runId;
  clear();
  status.textContent = "Menjalankan…";

  worker = new Worker(workerUrl);
  const current = worker;
  let printed = 0;

  timer = setTimeout(() => {
    if (id !== runId) return;
    stop();
    addLine("error", "Dihentikan: kode berjalan lebih dari " + TIME_LIMIT_MS / 1000 + " detik.");
    status.textContent = "Dihentikan";
  }, TIME_LIMIT_MS);

  current.onmessage = (event) => {
    if (id !== runId) return;
    const msg = event.data;
    if (msg.type === "line") {
      printed += 1;
      addLine(msg.level, msg.text, msg.line);
      output.scrollTop = output.scrollHeight;
    } else if (msg.type === "done") {
      clearTimeout(timer);
      if (printed === 0) {
        placeholder("Tidak ada keluaran. Gunakan tulis(...) untuk mencetak.");
      }
      status.textContent = "Selesai dalam " + Math.max(1, Math.round(msg.ms)) + " ms";
    }
  };
  current.onerror = (event) => {
    if (id !== runId) return;
    event.preventDefault();
    addLine("error", "Galat: " + event.message);
    status.textContent = "Galat";
  };
  current.postMessage(instrument(js, readCallSites()));
}

function update() {
  const js = document.getElementById("js")?.innerText ?? "";
  if (js.startsWith("// Salah sintaks")) {
    stop();
    runId += 1;
    placeholder("Perbaiki kesalahan sintaks dulu, lalu kode akan dijalankan.");
    status.textContent = "Menunggu";
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

// The playground is rendered by wasm, so watch its output pane for changes.
new MutationObserver(schedule).observe(document.getElementById("playground"), {
  childList: true,
  subtree: true,
  characterData: true,
});
schedule();
