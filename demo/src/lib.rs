extern crate printer;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate yew;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use printer::to_js;

struct Model {
    link: ComponentLink<Self>,
    code: String,
    transpiled: String,
}

enum Msg {
    ChangeCode(String),
}

const EXAMPLE_CODE: &str = "misal x = 2 + 2;
misal y = x > 2;
jika y benar {
  x = x + 1;
  menang();
}
";

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            code: EXAMPLE_CODE.into(),
            transpiled: to_js(EXAMPLE_CODE.into()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeCode(v) => {
                self.transpiled = to_js(v.clone().into());
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
                <div>
                    <label for="input",>{"Masukan"}</label>
                    <textarea value={&self.code} oninput={self.link.callback(|e: InputData| Msg::ChangeCode(e.value))} />
                </div>
                <div>
                    <label>{"Keluaran (JavaScript):"}</label>
                    <pre id="js",>{&self.transpiled}</pre>
                </div>
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
