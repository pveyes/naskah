extern crate naskah_demo;
extern crate stdweb;
extern crate yew;

use naskah_demo::Model;
use stdweb::web::{document, IParentNode};
use yew::prelude::*;

fn main() {
  let element = document().query_selector("#playground").unwrap().unwrap();

  yew::initialize();
  App::<Model>::new().mount(element);
  yew::run_loop();
}
