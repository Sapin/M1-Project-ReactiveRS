
extern crate piston_window;
extern crate sdl2_window;

pub mod runtime;
pub mod arrow;
pub mod signal;
#[macro_use]
pub mod macros;

use piston_window::*;
use sdl2_window::Sdl2Window;
// use std::result::{Result};
// use arrow::{Arrow};
// use arrow::prim::{identity,value,map,pause,fixpoint,product,fork};
// use signal::{Signal};
// use signal::prim::{PureSignal,ValueSignal,UniqSignal};
// 
fn main() {
    let glutin_window = WindowSettings::new("ReactivePacman", (640,480))
        .exit_on_esc(false).resizable(false).srgb(false)
        .build().unwrap();
    let mut window: PistonWindow<Sdl2Window> = PistonWindow::new(OpenGL::V3_2, 0, glutin_window);
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

