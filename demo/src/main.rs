#[macro_use]
extern crate yew;
extern crate printer;
extern crate stdweb;

use printer::to_js;
use stdweb::web::{document, IParentNode};
use yew::prelude::*;

struct Model {
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
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            code: EXAMPLE_CODE.into(),
            transpiled: to_js(EXAMPLE_CODE.into()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeCode(code) => {
                self.code = code.clone();
                self.transpiled = to_js(code.clone());
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <div>
                    <label for="input",>{"Masukan"}</label>
                    <textarea
                        id="input",
                        value=&self.code,
                        oninput=|e| Msg::ChangeCode(e.value),>
                    </textarea>
                </div>
                <div>
                    <label>{"Keluaran (JavaScript):"}</label>
                    <pre id="js",>{&self.transpiled}</pre>
                </div>
            </>
        }
    }
}

fn main() {
    let element = document().query_selector("#playground").unwrap().unwrap();

    yew::initialize();
    App::<Model>::new().mount(element);
    yew::run_loop();
}
