// Runs a compiled Naskah program in a throwaway worker (worker.js) and reports what it
// prints. A worker has no access to the page and can be stopped, so an endless
// `ulang { }` never freezes anything. Used by the playground and by the tutorial.

export const TIME_LIMIT_MS = 2000;

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
 * too long, or `on.crash(message)` if the worker itself fails. Returns a function that
 * stops the program.
 */
export function runProgram(compiled, on) {
  const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  let finished = false;

  const stop = () => {
    finished = true;
    clearTimeout(timer);
    worker.terminate();
  };

  const timer = setTimeout(() => {
    if (finished) return;
    stop();
    on.timeout();
  }, TIME_LIMIT_MS);

  worker.onmessage = (event) => {
    if (finished) return;
    const msg = event.data;
    if (msg.type === "line") {
      on.line(msg);
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

  worker.postMessage({ code: instrument(compiled.js, compiled.sites), lines: compiled.lines });
  return stop;
}
