// Makes the examples of the tutorial come alive: coloured code, a button that runs the
// example right under it, and a link that opens it in the playground to be changed.

import init, { compile_program, highlight_html } from "./assets/wasm.js";
import { readCompiled, runProgram, TIME_LIMIT_MS } from "./runner.js";
import { addLine, ask } from "./terminal.js";

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

function enhance(pre) {
  const code = pre.textContent.replace(/^\n+|\s+$/g, "");

  const figure = element("figure", "example");
  const block = element("pre", "code");
  const inner = element("code");
  inner.innerHTML = highlight_html(code);
  block.appendChild(inner);

  const run = element("button", "run-button", "Jalankan");
  run.type = "button";
  const edit = element("a", "edit-link", "Ubah di Tempat Coba");
  edit.href = "./#kode=" + encodeURIComponent(code);
  const bar = element("div", "example-bar");
  bar.append(run, edit);

  const terminal = element("div", "console example-console");
  terminal.hidden = true;
  const output = element("div", "console-output");
  output.setAttribute("role", "log");
  terminal.appendChild(output);

  figure.append(block, bar, terminal);
  pre.replaceWith(figure);

  let stop = null;
  run.addEventListener("click", () => {
    if (stop) stop();
    stop = null;
    output.replaceChildren();
    terminal.hidden = false;

    const compiled = readCompiled(compile_program(code));
    if (compiled.error) {
      const { message, line, column } = compiled.error;
      addLine(output, "error", "Salah tulis: " + message, line, column);
      return;
    }

    let printed = 0;
    stop = runProgram(compiled, {
      line(msg) {
        printed += 1;
        addLine(output, msg.level, msg.text, msg.line);
      },
      ask(question, reply) {
        printed += 1;
        return ask(output, question, reply);
      },
      done() {
        if (printed === 0) addLine(output, "note", "Tidak ada keluaran. Gunakan tulis(...) untuk mencetak.");
      },
      timeout() {
        addLine(output, "error", "Dihentikan: kode berjalan lebih dari " + TIME_LIMIT_MS / 1000 + " detik.");
      },
      crash(message) {
        addLine(output, "error", "Galat: " + message);
      }
    });
  });
}

// The lessons are shown one at a time, like pages, instead of one long scroll. The address
// says which one (#pelajaran-3), so the back button and shared links work. Without this
// script the whole page is simply shown in one piece.
function paginate() {
  const lessons = [...document.querySelectorAll(".lesson")];
  const entries = [...document.querySelectorAll(".toc li")];
  const body = document.body;

  let visited = new Set();
  try {
    visited = new Set(JSON.parse(localStorage.getItem("naskah-pelajaran") || "[]"));
  } catch {}
  const remember = () => {
    try {
      localStorage.setItem("naskah-pelajaran", JSON.stringify([...visited]));
    } catch {}
  };

  lessons.forEach((lesson, i) => {
    const title = lesson.querySelector("h2").textContent;

    const back = element("a", "lesson-back", "Semua pelajaran");
    back.href = "#";
    const progress = element("div", "progress");
    progress.setAttribute("role", "progressbar");
    progress.setAttribute("aria-label", "Pelajaran " + (i + 1) + " dari " + lessons.length);
    const filled = element("span");
    filled.style.width = ((i + 1) / lessons.length) * 100 + "%";
    progress.appendChild(filled);
    const where = element("p", "lesson-where", "Pelajaran " + (i + 1) + " dari " + lessons.length);
    lesson.prepend(back, where, progress);

    const nav = element("nav", "lesson-nav");
    nav.setAttribute("aria-label", "Pindah pelajaran");
    const previous = element("a", "", i > 0 ? "Sebelumnya" : "Semua pelajaran");
    previous.href = i > 0 ? "#" + lessons[i - 1].id : "#";
    const next = element("a", "primary", i < lessons.length - 1 ? "Berikutnya" : "Selesai, ayo mencoba");
    next.href = i < lessons.length - 1 ? "#" + lessons[i + 1].id : "./";
    nav.append(previous, next);
    lesson.appendChild(nav);

    entries[i].dataset.title = title;
  });

  function show() {
    const index = lessons.findIndex((l) => l.id === location.hash.slice(1));
    body.classList.toggle("home-view", index === -1);
    lessons.forEach((lesson, i) => lesson.classList.toggle("current", i === index));

    if (index === -1) {
      document.title = "Belajar Naskah - Pelajaran pemrograman dalam Bahasa Indonesia";
    } else {
      visited.add(index);
      remember();
      document.title = lessons[index].querySelector("h2").textContent + " - Belajar Naskah";
      lessons[index].querySelector("h2").setAttribute("tabindex", "-1");
      lessons[index].querySelector("h2").focus({ preventScroll: true });
    }
    entries.forEach((entry, i) => entry.classList.toggle("done", visited.has(i)));
    window.scrollTo(0, 0);
  }

  body.classList.add("paged");
  window.addEventListener("hashchange", show);
  show();
}

await init();
for (const pre of document.querySelectorAll("pre.naskah")) enhance(pre);
paginate();
