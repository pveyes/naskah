use printer::to_js;
use yew::prelude::worker::*;

pub struct Worker {
  link: AgentLink<Worker>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
  Compile(String),
}

impl Transferable for Request {}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
  Compiled(String),
}

impl Transferable for Response {}

pub enum Msg {}

impl Agent for Worker {
  type Reach = Public;
  type Message = Msg;
  type Input = Request;
  type Output = Response;

  fn create(link: AgentLink<Self>) -> Self {
    Worker { link }
  }

  fn update(&mut self, _: Self::Message) {}

  fn handle(&mut self, msg: Self::Input, id: HandlerId) {
    match msg {
      Request::Compile(code) => {
        let compiled = to_js(code);
        self.link.response(id, Response::Compiled(compiled));
      }
    }
  }

  fn name_of_resource() -> &'static str {
    "worker/worker.js"
  }
}
