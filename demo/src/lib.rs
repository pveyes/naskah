#[macro_use]
extern crate yew;
extern crate printer;
extern crate stdweb;
#[macro_use]
extern crate serde_derive;

pub mod worker;

use printer::to_js;
use yew::prelude::*;

pub struct Model {
    code: String,
    transpiled: String,
    worker: Box<Bridge<worker::Worker>>,
}

pub enum Msg {
    ChangeCode(String),
    CompilationFinished(String),
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

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|re| match re {
            worker::Response::Compiled(t) => Msg::CompilationFinished(t),
        });
        let worker = worker::Worker::bridge(callback);

        Model {
            code: EXAMPLE_CODE.into(),
            transpiled: to_js(EXAMPLE_CODE.into()),
            worker,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeCode(code) => {
                self.code = code.clone();
                self.worker.send(worker::Request::Compile(code));
            }
            Msg::CompilationFinished(t) => {
                self.transpiled = t;
            }
        };
        true
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
