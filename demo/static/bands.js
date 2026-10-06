// Highlight bands over the Naskah line a line of output came from. Code is rendered by
// wasm, so a band is drawn in a layer of its own on top instead of inside the code.
// The playground and the tutorial share this.

/** Client rects (one per visual row) covering `lineNumber` of `root`'s text. */
export function lineRows(root, lineNumber) {
  const lines = root.textContent.split("\n");
  if (lineNumber < 1 || lineNumber > lines.length) return [];

  let start = 0;
  for (let i = 0; i < lineNumber - 1; i++) start += lines[i].length + 1;
  return rangeRects(root, start, start + lines[lineNumber - 1].length);
}

/** Client rects (one per visual row) covering characters `start` to `end` of `root`'s text. */
export function rangeRects(root, start, end) {
  if (end <= start) return [];

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

/**
 * Draw a band over line `lineNumber` of `root` into `layer`. The band is as wide as `frame`
 * and `layer` must be positioned over (or around) it.
 */
export function drawBand(layer, root, frame, lineNumber) {
  const origin = layer.getBoundingClientRect();
  const box = frame.getBoundingClientRect();
  const lineHeight = parseFloat(getComputedStyle(root).lineHeight);

  for (const rect of lineRows(root, lineNumber)) {
    const height = Number.isFinite(lineHeight) ? lineHeight : rect.height;
    const band = document.createElement("div");
    band.className = "line-band";
    band.style.top = rect.top + rect.height / 2 - height / 2 - origin.top + "px";
    band.style.left = box.left - origin.left + "px";
    band.style.width = box.width + "px";
    band.style.height = height + "px";
    layer.appendChild(band);
  }
}
