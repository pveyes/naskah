#![recursion_limit = "256"]

extern crate printer;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate yew;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use printer::{transpile_checked, Transpiled};

mod highlight;
use highlight::{tokenize, Kind, Lang};

struct Model {
    link: ComponentLink<Self>,
    code: String,
    transpiled: String,
    /// `js_line:naskah_line` pairs, read by console.js
    call_sites: String,
}

enum Msg {
    ChangeCode(String),
}

const EXAMPLE_CODE: &str = r#"// Hewan peliharaan
Hewan(nama) {
  .nama = nama

  suara() {
    hasilkan "..."
  }
}

Kucing(nama) turunan Hewan(nama) {
  suara() {
    hasilkan "meong, bukan {..suara()}"
  }
}

misal daftar = [Hewan("Burung"), Kucing("Tom")]
untuk setiap h dalam daftar {
  tulis("{h.nama} bilang {h.suara()}")
}

fungsi ambil() {
  tunggu tunda(300)
  lempar Galat("jaringan putus")
}

coba {
  tunggu ambil()
} tangkap galat {
  tulis("Gagal: {galat.pesan}")
}
"#;

/// The JavaScript, plus `sites|lines` for console.js: "js:naskah" pairs for every
/// `tulis` call and for the first line of every statement.
fn compile(src: &str) -> (String, String) {
    compile_from(&transpile_checked(src))
}

fn compile_from(t: &Transpiled) -> (String, String) {
    let pairs = |list: &[(usize, usize)]| {
        list.iter()
            .map(|(js, naskah)| format!("{}:{}", js, naskah))
            .collect::<Vec<_>>()
            .join(",")
    };
    let map = format!("{}|{}", pairs(&t.call_sites), pairs(&t.statement_lines));
    (t.js.clone(), map)
}

fn highlighted(lang: Lang, src: &str) -> Html {
    html! {
        <>
            { for tokenize(lang, src).into_iter().map(|(kind, text)| match kind {
                Kind::Plain => html! { {text} },
                _ => html! { <span class=kind.class()>{text}</span> },
            }) }
        </>
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let (transpiled, call_sites) = compile(EXAMPLE_CODE);
        Self {
            link,
            code: EXAMPLE_CODE.into(),
            transpiled,
            call_sites,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeCode(v) => {
                let (transpiled, call_sites) = compile(&v);
                self.transpiled = transpiled;
                self.call_sites = call_sites;
                self.code = v;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <div class="pane">
                    <label class="pane-title" for="input">{"Naskah"}</label>
                    <div class="editor">
                        <pre class="backdrop" aria-hidden="true">{highlighted(Lang::Naskah, &self.code)}{"\n"}</pre>
                        <textarea id="input" spellcheck="false" value={&self.code} oninput={self.link.callback(|e: InputData| Msg::ChangeCode(e.value))} />
                    </div>
                </div>
                <div class="pane">
                    <span class="pane-title">{"JavaScript"}</span>
                    <pre class="output" id="js" aria-live="polite">{highlighted(Lang::JavaScript, &self.transpiled)}</pre>
                </div>
                <span id="sourcemap" class="sourcemap" aria-hidden="true">{&self.call_sites}</span>
            </>
        }
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Naskah code as HTML, coloured like the playground, for pages that only show code.
#[wasm_bindgen]
pub fn highlight_html(src: &str) -> String {
    let mut html = String::new();
    for (kind, text) in tokenize(Lang::Naskah, src) {
        match kind {
            Kind::Plain => html.push_str(&escape_html(text)),
            _ => html.push_str(&format!("<span class=\"{}\">{}</span>", kind.class(), escape_html(text))),
        }
    }
    html
}

/// Check and translate a program for a page that runs it itself. The answer is
/// `ok`, the JavaScript, the `tulis` sites and the statement lines, or `error`, the
/// message, the line and the column, all joined with the character U+0001.
#[wasm_bindgen]
pub fn compile_program(src: &str) -> String {
    let t = transpile_checked(src);
    if let Some(e) = t.error {
        return ["error".to_string(), e.message, e.line.to_string(), e.col.to_string()].join("\u{1}");
    }
    let (js, map) = compile_from(&t);
    let mut parts = map.splitn(2, '|');
    let (sites, lines) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    ["ok", js.as_str(), sites, lines].join("\u{1}")
}

#[wasm_bindgen(start)]
pub fn run_app() {
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();
    if let Some(element) = doc.query_selector("#playground").expect("No matching id") {
        App::<Model>::new().mount(element);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn highlighted_code_is_html() {
        let html = highlight_html("jika a < 3 { tulis(\"x & y\") }");
        assert!(html.contains("<span class=\"tok-keyword\">jika</span>"), "{}", html);
        assert!(html.contains("<span class=\"tok-operator\">&lt;</span>"), "{}", html);
        assert!(html.contains("x &amp; y"), "{}", html);
        assert!(!html.contains("> < "), "{}", html);
    }

    /// Every `<pre class="naskah">` example of the tutorial, as (code, attributes).
    fn lesson_examples() -> Vec<(String, String)> {
        const PAGE: &str = include_str!("../static/belajar.html");
        let unescape = |s: &str| {
            s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
        };
        let mut examples = Vec::new();
        let mut rest = PAGE;
        while let Some(start) = rest.find("<pre class=\"naskah\"") {
            let from = &rest[start..];
            let tag_end = from.find('>').unwrap();
            let end = from.find("</pre>").unwrap();
            examples.push((unescape(&from[tag_end + 1..end]), from[..tag_end].to_string()));
            rest = &from[end..];
        }
        examples
    }

    #[test]
    fn every_example_in_the_tutorial_compiles() {
        let examples = lesson_examples();
        assert!(examples.len() >= 30, "found only {} examples", examples.len());

        for (code, attributes) in examples {
            let answer = compile_program(&code);
            let failed = answer.starts_with("error");
            // data-salah marks the examples that show a mistake in the writing on purpose
            if attributes.contains("data-salah") {
                assert!(failed, "should have been a mistake:\n{}", code);
            } else {
                assert!(!failed, "{}\n--- in:\n{}", answer.replace('\u{1}', " | "), code);
            }
        }
    }

    #[test]
    fn the_tutorial_marks_the_mistakes_it_teaches() {
        let examples = lesson_examples();
        let on_purpose: Vec<_> = examples
            .iter()
            .filter(|(_, a)| a.contains("data-salah") || a.contains("data-galat"))
            .collect();
        assert_eq!(on_purpose.len(), 3);
    }

    #[test]
    fn compiling_a_program_answers_with_one_string() {
        let ok = compile_program("tulis(1)\ntulis(2)");
        let parts: Vec<&str> = ok.split('\u{1}').collect();
        assert_eq!(parts[0], "ok");
        assert_eq!(parts[1], "console.log(1);\nconsole.log(2);\n");
        assert_eq!(parts[2], "1:1,2:2");
        assert_eq!(parts[3], "1:1,2:2");

        let bad = compile_program("tulis(nmaa)");
        let parts: Vec<&str> = bad.split('\u{1}').collect();
        assert_eq!(parts, vec!["error", "`nmaa` belum dibuat. Buat dulu dengan misal nmaa = ...", "1", "7"]);
    }
}
