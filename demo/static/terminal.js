// What a program shows in Keluaran: lines of output, and a question with a place to
// answer it. The playground and the tutorial draw the same thing.

/** Add a line to `output`; returns the row. `line` and `column` say where in the program (the file `file`) it came from. */
export function addLine(output, level, text, line, column, file = "naskah.nsk") {
  const row = document.createElement("div");
  row.className = "console-line console-" + level;

  const message = document.createElement("span");
  message.className = "console-text";
  message.textContent = text;
  row.appendChild(message);

  if (line) {
    row.dataset.site = line;
    const site = document.createElement("span");
    site.className = "console-site";
    site.textContent = file + ":" + line + (column ? ":" + column : "");
    row.appendChild(site);
  }
  output.appendChild(row);
  output.scrollTop = output.scrollHeight;
  return row;
}

/**
 * Print `question`, then a box to type the answer in. `reply(text)` is called when it is
 * sent, and the box turns into a line that shows what was typed. Returns a function that
 * takes the box away again, for when the program is stopped before it is answered.
 */
export function ask(output, question, reply) {
  addLine(output, "ask", question || "…");

  const form = document.createElement("form");
  form.className = "ask-form";

  const mark = document.createElement("span");
  mark.className = "ask-mark";
  mark.textContent = "›";
  mark.setAttribute("aria-hidden", "true");

  const input = document.createElement("input");
  input.className = "ask-input";
  input.type = "text";
  input.maxLength = 400;
  input.placeholder = "Ketik jawabanmu, lalu tekan Enter";
  input.autocomplete = "off";
  input.autocapitalize = "off";
  input.spellcheck = false;
  input.enterKeyHint = "send";
  input.setAttribute("aria-label", question || "Jawabanmu");

  const send = document.createElement("button");
  send.className = "ask-send";
  send.type = "submit";
  send.textContent = "Kirim";

  form.append(mark, input, send);
  output.appendChild(form);
  output.scrollTop = output.scrollHeight;
  input.focus({ preventScroll: true });
  form.scrollIntoView({ block: "nearest" });

  let finished = false;
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (finished) return;
    finished = true;
    const text = input.value;
    const answered = document.createElement("div");
    answered.className = "console-line console-answer";
    const shown = document.createElement("span");
    shown.className = "console-text";
    shown.textContent = text;
    answered.appendChild(shown);
    form.replaceWith(answered);
    output.scrollTop = output.scrollHeight;
    reply(text);
  });

  return () => {
    if (finished) return;
    finished = true;
    form.remove();
  };
}
