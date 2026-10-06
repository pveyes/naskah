#![recursion_limit = "256"]

extern crate printer;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate yew;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use printer::transpile_checked;

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
    let t = transpile_checked(src);
    let pairs = |list: &[(usize, usize)]| {
        list.iter()
            .map(|(js, naskah)| format!("{}:{}", js, naskah))
            .collect::<Vec<_>>()
            .join(",")
    };
    let map = format!("{}|{}", pairs(&t.call_sites), pairs(&t.statement_lines));
    (t.js, map)
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

#[wasm_bindgen(start)]
pub fn run_app() {
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();
    if let Some(element) = doc.query_selector("#playground").expect("No matching id") {
        App::<Model>::new().mount(element);
    }
}
