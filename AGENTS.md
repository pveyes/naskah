# AGENTS.md

Guidance for anyone (human or AI agent) changing this repository.

## What Naskah is for

Naskah is a programming language for **primary school students in Indonesia who know no English**. That goal outranks everything else:

- Efficiency and programmer convenience are **not** goals. Readability for a child is.
- JavaScript is only the target that is easiest to run in a browser. It is an implementation detail: do not minify it, and do not design the language around it.
- The student should never need to read, or even see, JavaScript. The JS pane in the playground is hidden, and only shown when a teacher opens `/#javascript` (linked from `guru.html`). There is no button for it in the editor.

## Language design rules

These were decided together with the project owner. Follow them, and ask before bending one.

- **Words, not symbols.** No shortcut syntax: no ternary, `++`/`--`, `+=` and other compound assignment, `??`, `?.`, destructuring, spread/rest, default parameters, arrow functions. If something is only a shorter way to write what already exists, leave it out.
- **Everything the student reads is Indonesian**: keywords, built-in names, error messages. Do not let English leak: JavaScript error messages, `console`, `Math`, `Date`, `JSON`, `Error` and the like must be reachable only through Indonesian names (`tulis`, `Galat`, `tunda`, ...). New built-ins get Indonesian names.
- **Errors are for children.** Say what went wrong in plain Indonesian, point at the Naskah line and column, and suggest the fix. Never show a JS line number, JS stack trace or JS wording, not even as a "technical detail". Show **one mistake at a time**: several errors at once overwhelm a child, and recovery guesses cascade into noise.
- **Catch mistakes before running.** The playground uses `transpile_checked`, which also refuses names that were never made (with a "Maksudmu `nama`?" suggestion). Plain `transpile` skips that check, which keeps the tests short.
- **Numbers use the Indonesian format**: decimals are written `1,5`, not `1.5`. A comma between two digits with no space is a decimal point; a separator comma needs a space after it (`[1, 2, 3]`, `f(1, 5)`).
- **Capital letters mean class.** A name starting with a capital letter is a class and calling it always builds an object (`Hewan("Tom")`). Variables, functions, parameters and methods start with a lowercase letter.
- **Semicolons are optional.** A line break, `}` or the end of the code ends a statement; a token that starts a line never continues the previous line.
- **`.nama` is the current object and `..nama` is the parent's version.** Both only inside classes.
- **A function is async when its body uses `tunggu`.** There is no `async` marker.
- JavaScript reserved words are fine as Naskah names (`misal delete = 1`): the printer writes them with a `$` on the end, and so for names starting with `__`. Property names, methods and object keys are left alone.
- Keep the vocabulary small. Prefer one obvious way to do a thing.

## Known gaps

Things that break the rules above today. Fix them rather than copying them.

- Runtime-error explanations match Chrome's wording. Other browsers fall back to a generic Indonesian sentence about the kind of error, which is safe but less specific.
- Built-ins that are still English: `Math`, `Date`, `JSON`, `Promise`, the string and number methods. They are allowed by `GLOBALS` in `parser/src/syntax.rs` so nothing breaks, but they need Indonesian names.
- `.gabung(",")` and other JavaScript methods still turn numbers into text with a dot.
- The link to `guru.html` in the footer, that page, and the `#javascript` view are the only places that say "JavaScript" (besides the tutorial's own sentence about English); keep it that way. `guru.html` is where the Naskah-to-JavaScript equivalents live, for teachers and developers.

## How the pieces fit

- `parser/` is a hand-written lexer and recursive-descent parser, with positioned errors in Indonesian.
- `printer/` turns the AST into readable JavaScript, one statement per line, and returns two `(js_line, naskah_line)` maps: one for `tulis` calls and one for every statement. The playground's line labels, hover highlight and runtime-error lines all rely on that one-statement-per-line output, which is why the JS must never be minified.
- The generated JS calls a few helpers that the playground's worker provides: `__tambah` (`+` that joins text with Indonesian numbers), `__teks` (a value in a text) and `__pesan` (an error's message, explained in Indonesian). They are not defined anywhere else, so the JS is not standalone.
- `demo/` is the playground: a Yew (Rust to wasm) app in `demo/src`, plus the static site in `demo/static`. `console.js` runs the generated JS in a Web Worker (`worker.js`) with a time limit. `worker.js` also turns JavaScript's own errors into Indonesian and finds the Naskah line they came from.
- `tanya(...)` waits for a person in the middle of a program. The worker sleeps on a `SharedArrayBuffer` (`runner.js` hands it over, `worker.js` waits on it) until the answer box in `terminal.js` fills it, and that time is not counted against the 2-second limit. Browsers only allow shared memory on pages sent with the cross-origin isolation headers, which is what `demo/static/public/_headers` does; the Google Fonts links still load under them. Without isolation `tanya` answers `kosong` and says so. A program that uses `tanya` waits for the Jalankan button instead of re-running on every edit.
- The home page has at most five example tabs (`demo/static/contoh.json`), each a short program in a file like `halo.nsk`, ordered from simple to hard; the file name also labels the lines it prints. The first one is what the editor starts with (`EXAMPLE_CODE` in `demo/src/lib.rs`, kept equal by a test), and a test compiles all of them.
- `demo/static/belajar.html` is the tutorial for children: twelve lessons shown one page at a time by `belajar.js`, each example with a Jalankan button (it uses the same `runner.js` and the wasm exports `compile_program` and `highlight_html` as the playground). The HTML is the source of truth. A test in `demo/src/lib.rs` compiles every `<pre class="naskah">` example, so a lesson cannot contain code that does not work. An example that shows a mistake on purpose carries `data-salah` (a mistake in the writing) or `data-galat` (only fails while running). Write lessons the way the page speaks: short Indonesian sentences and no English words. The playground is called "Tempat Coba" for children.
- `SYNTAX.md` is the language reference for users and is written in Indonesian. Update it with every language change, together with the vocabulary table in `demo/static/index.html`, the highlighter in `demo/src/highlight.rs`, and the tests.

## Working in the repo

```sh
cargo test --workspace   # parser, printer and highlighter tests
./scripts/dev.sh         # build the wasm demo and serve it locally with cf dev
./scripts/deploy.sh      # build and deploy to Cloudflare Workers with cf deploy
```

- **Always use the `cf` CLI for anything that touches Cloudflare** (auth, deploys, secrets, workers). Find commands with `cf cli search "<task>"` and keep the query anonymous: no names, emails, domains, account IDs or tokens. Do not use `wrangler` directly.
- The site is deployed as the Worker `naskah` (static assets, no Worker code). CI deploys on pushes to `master` and needs the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` repository secrets.
- Add a test with every language change. Parser and printer tests compare exact positions and output, so keep error messages and columns stable.
- Never commit local tool state (`.wrangler/`, `.cloudflare/`, `node_modules/`, `target/`). Check `git status` before `git add -A`.
- Writing: user-facing text and docs are Indonesian; code, comments and commit messages are English.
- Do not force-push, rewrite history, or delete repository secrets without being asked.
