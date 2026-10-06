// The tabs above the editor. Each one is a short program kept in a file with a name like
// a real one (halo.nsk). The name also labels the lines it prints, so a child sees
// where each line came from.

/** Fill `tabs` from contoh.json. `choose({ file, kode })` is called when one is picked. */
export async function setupExamples(tabs, choose) {
  let examples = [];
  try {
    examples = await (await fetch("./contoh.json")).json();
  } catch {
    return { first: null, clear() {} };
  }

  const buttons = examples.map((example, index) => {
    const tab = document.createElement("button");
    tab.type = "button";
    tab.className = "tab";
    tab.setAttribute("role", "tab");
    tab.setAttribute("aria-selected", "false");
    tab.textContent = example.file;
    tab.addEventListener("click", () => {
      mark(index);
      choose(example);
    });
    tabs.appendChild(tab);
    return tab;
  });

  function mark(selected) {
    buttons.forEach((tab, index) => {
      tab.setAttribute("aria-selected", String(index === selected));
      tab.tabIndex = index === selected ? 0 : -1;
    });
  }

  // arrow keys move between the tabs, like any set of tabs
  tabs.addEventListener("keydown", (event) => {
    const current = buttons.findIndex((tab) => tab === document.activeElement);
    const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
    if (current === -1 || !step) return;
    event.preventDefault();
    buttons[(current + step + buttons.length) % buttons.length].focus();
  });

  mark(-1);
  return {
    /** The first example, which is also what the editor starts with. */
    first: examples[0],
    markFirst: () => {
      mark(0);
      buttons[0].tabIndex = 0;
    },
    /** Nothing is selected, for a program that came from somewhere else. */
    clear: () => {
      mark(-1);
      if (buttons[0]) buttons[0].tabIndex = 0;
    }
  };
}
