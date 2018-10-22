extern crate naskah_demo;
extern crate yew;

use naskah_demo::worker;
use yew::prelude::*;

fn main() {
  yew::initialize();
  worker::Worker::register();
  yew::run_loop();
}
