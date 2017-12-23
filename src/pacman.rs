
extern crate piston_window;
extern crate sdl2_window;

pub mod runtime;
pub mod arrow;
pub mod signal;
#[macro_use]
pub mod macros;

use piston_window::*;
use sdl2_window::Sdl2Window;
use std::result::{Result};
use arrow::{Arrow};
use arrow::prim::{identity,value,map,pause,fixpoint,product,fork};
use signal::{Signal};
use signal::prim::{PureSignal,ValueSignal,UniqSignal};

#[derive(Clone,Copy)]
struct Pos {
    x: usize,
    y: usize,
}

struct Settings {
    width: usize,
    height: usize,
    cell_size: usize,
    start_pos: Pos,
    animation_len: usize,
    invincible_len: usize,
    yellow_pearls : Vec<Pos>,
    phantoms : Vec<(Pos,Pos)>,
    walls : Vec<(Pos,Pos)>
}

#[derive(Clone,Copy,Debug)]
enum CellContent {
    WhitePearl,
    YellowPearl,
    Empty
}

#[derive(Clone,Copy)]
enum Directions {
    Up,
    Down,
    Left,
    Right
}

#[derive(Clone,Copy)]
struct BoardCell {
    wall_up:    bool,
    wall_down:  bool,
    wall_left:  bool,
    wall_right: bool
}

type Walls  = Vec<Vec<BoardCell>>;
type Pearls = Vec<Vec<CellContent>>;

fn init_board(settings: &Settings) -> (Walls,Pearls) {
    let mut walls:  Walls = Vec::new();
    let mut pearls: Pearls = Vec::new();
    let wn : usize = settings.width;
    let hn : usize = settings.height;
    for x in 0..wn {
        let mut wcol: Vec<BoardCell> = Vec::new();
        let mut pcol: Vec<CellContent> = Vec::new();
        for y in 0..hn {
            let cell: BoardCell = BoardCell {
                wall_up:    y <= 1,
                wall_down:  y == hn - 1 || y == 0,
                wall_left:  x == 0,
                wall_right: x == wn - 1
            };
            wcol.push(cell);
            pcol.push(if y == 0 { CellContent::Empty } else { CellContent::WhitePearl });
        }
        walls.push(wcol);
        pearls.push(pcol);
    };
    for w in &settings.walls {
        let &(p1,p2) = w;
        if p1.x < p2.x {
            walls[p1.x][p1.y].wall_right = true;
            walls[p2.x][p2.y].wall_left  = true;
        }
        if p1.x > p2.x {
            walls[p1.x][p1.y].wall_left  = true;
            walls[p2.x][p2.y].wall_right = true;
        }
        if p1.y < p2.y {
            walls[p1.x][p1.y].wall_down  = true;
            walls[p2.x][p2.y].wall_up    = true;
        }
        if p1.y < p2.y {
            walls[p1.x][p1.y].wall_up    = true;
            walls[p2.x][p2.y].wall_down  = true;
        }
    };
    for yellow in &settings.yellow_pearls {
        pearls[yellow.x][yellow.y] = CellContent::YellowPearl;
    };
    (walls,pearls)
}

#[derive(Clone)]
struct DrawingData {
    pacman:   Option<Pos>,
    phantoms: Vec<Pos>,
}

fn compress_ddata(a : DrawingData, b : DrawingData) -> DrawingData {
    DrawingData {
        pacman:   if let None = a.pacman { b.pacman } else { a.pacman },
        phantoms: [&a.phantoms[..], &b.phantoms[..]].concat(),
    }
}

fn draw_board(walls: &Walls, pearls: &Pearls, settings: &Settings, ddata: &DrawingData,
              window: &mut PistonWindow<Sdl2Window>, ev: &Event) {
    let cs = settings.cell_size;
    let div = 10;
    let wall_color = [0.2,0.2,1.0,1.0];
    window.draw_2d(ev, |context, graphics| {
        clear([0.0, 0.0, 0.1, 1.0], graphics);
        for x in 0..walls.len() {
            for y in 0..walls[x].len() {
                if walls[x][y].wall_up {
                    rectangle(wall_color,
                              [(x * cs) as f64, (y * cs) as f64, (cs) as f64, (cs / div) as f64],
                              context.transform,
                              graphics);
                }
                if walls[x][y].wall_down {
                    rectangle(wall_color,
                              [(x * cs) as f64, (y * cs + (div-1) * cs / div) as f64, (cs) as f64, (cs / div) as f64],
                              context.transform,
                              graphics);
                }
                if walls[x][y].wall_left {
                    rectangle(wall_color,
                              [(x * cs) as f64, (y * cs) as f64, (cs / div) as f64, (cs) as f64],
                              context.transform,
                              graphics);
                }
                if walls[x][y].wall_right {
                    rectangle(wall_color,
                              [(x * cs + (div-1) * cs / div) as f64, (y * cs) as f64, (cs / div) as f64, (cs) as f64],
                              context.transform,
                              graphics);
                }
                match pearls[x][y] {
                    CellContent::WhitePearl  =>
                        { ellipse([1.0,1.0,0.0,1.0],
                                  ellipse::circle((x*cs + cs/2) as f64, (y*cs + cs/2) as f64, (cs/10) as f64),
                                  context.transform,
                                  graphics);
                        },
                    CellContent::YellowPearl =>
                        { ellipse([1.0,1.0,0.0,1.0],
                                  ellipse::circle((x*cs + cs/2) as f64, (y*cs + cs/2) as f64, (cs/4) as f64),
                                  context.transform,
                                  graphics);
                        },
                    CellContent::Empty       => ()
                }

                if let &Some(pos) = &ddata.pacman {
                    ellipse([1.0,1.0,1.0,1.0],
                            ellipse::circle((pos.x*cs + cs/2) as f64,
                                            (pos.y*cs + cs/2) as f64,
                                            (cs/3) as f64),
                            context.transform,
                            graphics);
                }
            }
        }
    });
}

fn can_move(pos: &Pos, dir: &Directions, walls: &Walls) -> bool {
    match *dir {
        Directions::Up    => !walls[pos.x][pos.y].wall_up,
        Directions::Down  => !walls[pos.x][pos.y].wall_down,
        Directions::Left  => !walls[pos.x][pos.y].wall_left,
        Directions::Right => !walls[pos.x][pos.y].wall_right,
    }
}

fn main() {
    let gsettings : Settings = Settings {
        width: 20,
        height: 11,
        cell_size: 50,
        start_pos: Pos { x: 0, y:0 },
        animation_len: 100,
        invincible_len: 100,
        yellow_pearls: vec![Pos{ x:1, y:1 }],
        phantoms: Vec::new(),
        walls: Vec::new()
    };
    let ddata : DrawingData = DrawingData {
        pacman: Some(Pos{ x:0, y:0 }),
        phantoms: Vec::new()
    };
    let (walls, mut pearls) = init_board(&gsettings);

    let glutin_window = WindowSettings::new("ReactivePacman",
                                            (((gsettings.width+1) * gsettings.cell_size) as u32,
                                            ((gsettings.height+1) * gsettings.cell_size) as u32))
        .exit_on_esc(false).resizable(false).srgb(false)
        .build().unwrap();
    let mut window: PistonWindow<Sdl2Window> = PistonWindow::new(OpenGL::V3_2, 0, glutin_window);

    while let Some(event) = window.next() {
        draw_board(&walls, &pearls, &gsettings, &ddata, &mut window, &event);
    }
}

