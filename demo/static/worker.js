// Runs one Naskah program, already translated to JavaScript, away from the page.
// A worker has no access to the page and can be stopped, so an endless loop in the
// student's program never freezes the playground.
//
// Everything a student can see comes out of here, so nothing in it may be English:
// values are shown the Indonesian way and errors are explained in Indonesian,
// with the line of the Naskah program they happened on.

const MAX_LINES = 200;

// ---------------------------------------------------------------- showing values

const show = (v, top, seen = []) => {
  if (typeof v === "string") return top ? v : JSON.stringify(v);
  if (typeof v === "number") {
    if (Number.isNaN(v)) return "bukan angka";
    if (v === Infinity) return "tak hingga";
    if (v === -Infinity) return "min tak hingga";
    return String(v).replace(".", ",");
  }
  if (v === null || v === undefined) return "kosong";
  if (v === true) return "benar";
  if (v === false) return "salah";
  if (typeof v === "function") return "[fungsi]";
  if (typeof v === "object") {
    if (seen.includes(v) || seen.length > 3) return "[...]";
    const next = seen.concat([v]);
    if (Array.isArray(v)) return "[" + v.map((x) => show(x, false, next)).join(", ") + "]";
    if (v instanceof Error) return explain(v);
    const keys = Object.keys(v);
    if (!keys.length) return "{}";
    return "{ " + keys.map((k) => indo(k) + ": " + show(v[k], false, next)).join(", ") + " }";
  }
  return String(v);
};

// JavaScript's names for things a student knows by their Naskah names.
const NAMES = {
  length: "panjang",
  push: "tambah",
  join: "gabung",
  reverse: "balik",
  map: "peta",
  filter: "saring",
  find: "cari",
  sort: "urut",
  message: "pesan",
  "console.log": "tulis",
  prompt: "tanya",
};
const indo = (name) =>
  String(name)
    .replace(/console\.log/g, "tulis")
    .replace(/\(intermediate value\)/g, "sesuatu")
    // `.nama` and `..nama` are how Naskah writes this.nama and super.nama
    .replace(/\bthis\./g, "\u0001")
    .replace(/\bsuper\./g, "\u0002")
    .split(".")
    .map((part) => NAMES[part] || part)
    .join(".")
    .replace(/\u0002/g, "..")
    .replace(/\u0001/g, ".");

// ---------------------------------------------------------------- explaining errors

const NATIVE_ERRORS = [TypeError, ReferenceError, RangeError, SyntaxError, EvalError, URIError];
const isNative = (e) => NATIVE_ERRORS.some((T) => e instanceof T);

const q = (name) => "`" + indo(name) + "`";

// What the JavaScript engine said (Chrome's wording) and what to tell a child instead.
const RULES = [
  [/^(.+) is not defined$/, (m) => q(m[1]) + " belum dibuat. Buat dulu dengan misal " + m[1] + " = ..."],
  [
    /^Cannot access '(.+)' before initialization$/,
    (m) => q(m[1]) + " dipakai sebelum dibuat. Tulis misal " + m[1] + " = ... di atasnya."
  ],
  [
    /^Cannot read propert(?:y|ies) of (?:undefined|null) \(reading '(.+)'\)$/,
    (m) => "Tidak bisa mengambil " + q(m[1]) + " karena nilainya kosong."
  ],
  [
    /^Cannot set propert(?:y|ies) of (?:undefined|null) \(setting '(.+)'\)$/,
    (m) => "Tidak bisa mengisi " + q(m[1]) + " karena nilainya kosong."
  ],
  [/^(.+) is not a function$/, (m) => q(m[1]) + " bukan fungsi, jadi tidak bisa dipanggil."],
  [/^(.+) is not a constructor$/, (m) => q(m[1]) + " bukan kelas, jadi tidak bisa dibuat objeknya."],
  [/^(.+) is not iterable$/, (m) => q(m[1]) + " bukan daftar, jadi tidak bisa diulang dengan untuk setiap."],
  [
    /^Assignment to constant variable\.?$/,
    () => "Nilai konstan tidak bisa diganti. Pakai misal kalau nilainya perlu berubah."
  ],
  [/^Maximum call stack size exceeded$/, () => "Fungsi memanggil dirinya sendiri terus-menerus tanpa berhenti."],
  [/^Invalid array length$/, () => "Panjang daftar tidak boleh negatif atau terlalu besar."],
  [/^Cannot convert undefined or null to object$/, () => "Nilainya kosong, jadi tidak bisa dipakai di sini."]
];

const FALLBACKS = [
  [ReferenceError, "Ada nama yang belum dibuat."],
  [TypeError, "Ada nilai yang dipakai dengan cara yang tidak cocok."],
  [RangeError, "Angkanya di luar batas yang bisa dipakai."],
  [SyntaxError, "Ada tulisan yang tidak bisa dijalankan."]
];

function explain(e) {
  if (!(e instanceof Error)) return show(e, true);
  // an error the student threw themselves with Galat("...") says what they wrote
  if (!isNative(e)) return e.message;

  let text;
  for (const [pattern, say] of RULES) {
    const m = pattern.exec(e.message);
    if (m) {
      text = say(m);
      break;
    }
  }
  if (text) return text;
  return (FALLBACKS.find(([T]) => e instanceof T) || [0, "Terjadi kesalahan saat program berjalan."])[1];
}

// ---------------------------------------------------------------- where did it happen?

const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;

let lineMap = []; // [jsLine, naskahLine] pairs sorted by jsLine, one per statement
let frameOffset = null; // lines the engine adds in front of the program in a stack trace

const FRAME = /(?:<anonymous>|Function|eval):(\d+):\d+/;

const firstFrameLine = (error) => {
  for (const frame of String((error && error.stack) || "").split("\n")) {
    const m = FRAME.exec(frame);
    if (m) return Number(m[1]);
  }
  return null;
};

// Engines count the lines of a generated function differently, so find out how.
async function calibrate() {
  try {
    await new AsyncFunction('"use strict";\nthrow new Error("probe")')();
  } catch (e) {
    const line = firstFrameLine(e);
    frameOffset = line === null ? null : line - 2;
  }
}

const naskahLineOf = (error) => {
  const reported = firstFrameLine(error);
  if (reported === null || frameOffset === null) return undefined;
  const jsLine = reported - frameOffset - 1; // one more for the "use strict" line we add
  let found;
  for (const [js, naskah] of lineMap) {
    if (js <= jsLine) found = naskah;
    else break;
  }
  return found;
};

// ---------------------------------------------------------------- the program's world

let printed = 0;
const post = (level, text, line) => {
  printed += 1;
  if (printed <= MAX_LINES) postMessage({ type: "line", level, text, line });
  else if (printed === MAX_LINES + 1) {
    postMessage({ type: "line", level: "note", text: "Keluaran dipotong setelah " + MAX_LINES + " baris." });
  }
};

const format = (args) => args.map((a) => show(a, true)).join(" ");
const print = (level) => (...args) => post(level, format(args));

const sandbox = { log: print("log"), error: print("error"), warn: print("error") };

// tulis(...) calls are rewritten to __log(naskahLine, ...) before the program runs
const __log = (line, ...args) => post("log", format(args), line);
// what students see of a caught error: `galat.pesan`
const __pesan = (e) => (e instanceof Error ? explain(e) : e && e.message !== undefined ? e.message : show(e, true));
// a value as text, the Indonesian way
const __teks = (v) => show(v, true);
// `+` that joins text the Indonesian way when either side is text
const __tambah = (a, b) => (typeof a === "string" || typeof b === "string" ? __teks(a) + __teks(b) : a + b);

const prompt = () => {
  post("error", "tanya() belum didukung di sini dan selalu mengembalikan kosong.");
  return null;
};

function report(e) {
  post("error", "Galat: " + explain(e), naskahLineOf(e));
}

self.addEventListener("unhandledrejection", (event) => report(event.reason));

self.onmessage = async (event) => {
  lineMap = event.data.lines || [];
  await calibrate();

  const started = performance.now();
  try {
    const program = new AsyncFunction("console", "prompt", "__log", "__pesan", "__teks", "__tambah", '"use strict";\n' + event.data.code);
    await program(sandbox, prompt, __log, __pesan, __teks, __tambah);
  } catch (e) {
    report(e);
  }
  postMessage({ type: "done", ms: performance.now() - started });
};
