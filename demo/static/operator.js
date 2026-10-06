// Typing `kali` or `bagi` between two values turns the word into the school symbol, × or ÷,
// as soon as the space after it is typed. The playground editor is drawn by wasm, so this
// listens on the page and only acts on the textarea with id "input".

const SYMBOLS = { kali: "×", bagi: "÷" };

// words after which `kali` or `bagi` is a name being made, not an operator
const DECLARES = new Set(["misal", "konstan", "fungsi", "untuk", "ulang", "selama", "sampai", "dari"]);

/**
 * The edit to make after a space was typed at `caret`, as `{ from, to, text }`, or null.
 * `kali` and `bagi` only count when a value comes before them, and not inside a text or a note.
 */
export function operatorEdit(text, caret) {
  const line = text.slice(text.lastIndexOf("\n", caret - 2) + 1, caret - 1);
  const match = /(?:^|[^\p{L}\p{N}_])(kali|bagi)$/u.exec(line);
  if (!match) return null;
  const word = match[1];

  const before = line.slice(0, line.length - word.length).trimEnd();
  if (before === "" || before.includes("//") || (before.match(/"/g) ?? []).length % 2 === 1) return null;
  if (!/[\p{L}\p{N}_)\]"]$/u.test(before)) return null;
  const previous = /([\p{L}\p{N}_]+)$/u.exec(before);
  if (previous && DECLARES.has(previous[1])) return null;

  const to = caret - 1;
  return { from: to - word.length, to, text: SYMBOLS[word] };
}

document.addEventListener("input", (event) => {
  const box = event.target;
  if (box.id !== "input" || event.inputType !== "insertText" || event.data !== " ") return;
  const edit = operatorEdit(box.value, box.selectionStart);
  if (!edit) return;

  box.setSelectionRange(edit.from, edit.to);
  // insertText keeps the undo history; setRangeText is the fallback
  if (!document.execCommand("insertText", false, edit.text)) {
    box.setRangeText(edit.text, edit.from, edit.to, "end");
    box.dispatchEvent(new Event("input", { bubbles: true }));
  }
  // the caret was after the space, and the space is still there after the symbol
  box.setSelectionRange(edit.from + 2, edit.from + 2);
});
