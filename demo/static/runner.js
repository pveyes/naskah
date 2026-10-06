// Runs a compiled Naskah program in a throwaway worker (worker.js) and reports what it
// prints. A worker has no access to the page and can be stopped, so an endless
// `ulang { }` never freezes anything. Used by the playground and by the tutorial.

export const TIME_LIMIT_MS = 2000;

// room for the answer to tanya(...), in letters; worker.js has the same number
const ANSWER_MAX = 400;

// "3:12,7:20" -> [[3, 12], [7, 20]] (JS line, Naskah line)
function parsePairs(text) {
  return text
    .split(",")
    .map((pair) => pair.split(":").map(Number))
    .filter(([jsLine, naskahLine]) => jsLine && naskahLine);
}

/** The playground keeps "sites|lines" in the page; sites are the tulis(...) calls and
 *  lines are the first JS line of every statement. */
export function parseMap(text) {
  const [sitesText = "", linesText = ""] = text.split("|");
  return { sites: new Map(parsePairs(sitesText)), lines: parsePairs(linesText) };
}

/** What compile_program() answers, as an object: { js, sites, lines } or { error }. */
export function readCompiled(answer) {
  const [kind, first, second, third] = answer.split("\u0001");
  if (kind === "error") {
    return { error: { message: first, line: Number(second), column: Number(third) } };
  }
  return { js: first, ...parseMap(second + "|" + third) };
}

// Route each tulis(...) through __log(naskahLine, ...) so a printed line knows where it came from.
export function instrument(js, sites) {
  return js
    .split("\n")
    .map((text, index) => {
      const naskahLine = sites.get(index + 1);
      return naskahLine ? text.replace("console.log(", "__log(" + naskahLine + ", ") : text;
    })
    .join("\n");
}

/**
 * Run `compiled` ({ js, sites, lines }). `on.line({ level, text, line })` is called for
 * everything the program prints, then `on.done({ ms })`, or `on.timeout()` when it runs
 * too long, or `on.crash(message)` if the worker itself fails.
 *
 * When the program asks a question with tanya(...), `on.ask(question, reply)` is called;
 * `reply(text)` hands the answer back and may be called once. It can return a function
 * that is called if the program is stopped first. Waiting for a person does not count
 * towards the time limit. Without `on.ask`, or on a page that cannot share memory with
 * the worker, tanya(...) just answers kosong.
 *
 * Returns a function that stops the program.
 */
export function runProgram(compiled, on) {
  const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  let finished = false;
  let timer = null;
  let cancelAsk = null;

  const stop = () => {
    finished = true;
    clearTimeout(timer);
    if (cancelAsk) cancelAsk();
    cancelAsk = null;
    worker.terminate();
  };

  const startClock = () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (finished) return;
      stop();
      on.timeout();
    }, TIME_LIMIT_MS);
  };
  startClock();

  // the answer goes back through memory that both sides can see
  const canAsk = Boolean(on.ask) && typeof SharedArrayBuffer === "function" && self.crossOriginIsolated;
  const answers = canAsk ? new SharedArrayBuffer(8 + 2 * ANSWER_MAX) : null;
  const control = answers ? new Int32Array(answers, 0, 2) : null;
  const letters = answers ? new Uint16Array(answers, 8, ANSWER_MAX) : null;

  const reply = (text) => {
    if (finished) return;
    cancelAsk = null;
    const given = String(text).slice(0, ANSWER_MAX);
    for (let i = 0; i < given.length; i++) letters[i] = given.charCodeAt(i);
    Atomics.store(control, 1, given.length);
    Atomics.store(control, 0, 1);
    Atomics.notify(control, 0);
    startClock();
  };

  worker.onmessage = (event) => {
    if (finished) return;
    const msg = event.data;
    if (msg.type === "line") {
      on.line(msg);
    } else if (msg.type === "ask") {
      clearTimeout(timer);
      const cancel = on.ask(msg.question, reply);
      cancelAsk = typeof cancel === "function" ? cancel : null;
    } else if (msg.type === "done") {
      clearTimeout(timer);
      on.done(msg);
      // a promise nobody waited for may still print something
      setTimeout(stop, 100);
    }
  };
  worker.onerror = (event) => {
    if (finished) return;
    event.preventDefault();
    stop();
    on.crash(event.message);
  };

  worker.postMessage({ code: instrument(compiled.js, compiled.sites), lines: compiled.lines, answers });
  return stop;
}
