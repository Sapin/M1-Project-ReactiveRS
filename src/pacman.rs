
extern crate piston_window;

pub mod runtime;
pub mod arrow;
pub mod signal;
#[macro_use]
pub mod macros;

use piston_window::*;
use std::result::{Result};
use arrow::{Arrow};
use arrow::prim::{identity,value,map,pause,fixpoint,product,fork};
use signal::{Signal};
use signal::prim::{PureSignal,ValueSignal,UniqSignal};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640,480])
        .exit_on_esc(true).build().unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([1.0; 4], graphics);
            rectangle([1.0, 0.0, 0.0, 1.0],
                      [0.0, 0.0, 100.0, 100.0],
                      context.transform,
                      graphics);
        });
    }
}

