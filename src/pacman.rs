
extern crate piston_window;
extern crate piston;
extern crate sdl2_window;

pub mod runtime;
pub mod arrow;
pub mod signal;
#[macro_use]
pub mod macros;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::result::{Result};

use piston_window::*;
use sdl2_window::Sdl2Window;

use arrow::{Arrow};
use arrow::prim::{identity,value,map,pause,fixpoint,product,fork};
use signal::{Signal};
use signal::prim::{PureSignal,ValueSignal,UniqSignal};

#[derive(Clone,Copy,Debug)]
struct Pos {
    x: usize,
    y: usize,
}

struct Settings {
    width:           usize,
    height:          usize,
    cell_size:       usize,
    start_pos:       Pos,
    animation_len:   usize,
    invincible_len:  usize,
    yellow_pearls :  Vec<Pos>,
    phantoms_start : Vec<Pos>,
    phantoms_end :   Vec<Pos>,
    walls :          Vec<(Pos,Pos)>
}

#[derive(Clone,Copy,Debug)]
enum CellContent {
    WhitePearl,
    YellowPearl,
    Empty
}

#[derive(Clone,Copy,Debug)]
enum Directions {
    Up,
    Down,
    Left,
    Right,
    None
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
        if p1.x == p2.x {
            for y in (p2.y+1)..(p1.y+1) {
                walls[p1.x-1][y].wall_right = true;
                walls[p1.x]  [y].wall_left  = true;
            }
        } else {
            for x in p1.x..p2.x {
                walls[x][p1.y].wall_down = true;
                walls[x][p1.y+1].wall_up = true;
            }
        }
    };
    for yellow in &settings.yellow_pearls {
        pearls[yellow.x][yellow.y] = CellContent::YellowPearl;
    };
    (walls,pearls)
}

#[derive(Clone,Debug)]
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
    let div = 20;
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
        _                 => true
    }
}

fn update_pos(p: &Pos, d: &Directions) -> Option<Pos> {
    match *d {
        Directions::Up    => Some(Pos { x: p.x, y: p.y - 1 }),
        Directions::Down  => Some(Pos { x: p.x, y: p.y + 1 }),
        Directions::Left  => Some(Pos { x: p.x - 1, y: p.y }),
        Directions::Right => Some(Pos { x: p.x + 1, y: p.y }),
        _                 => None
    }
}

fn main() {
    let gsettings : Settings = Settings {
        width:          10,
        height:         6,
        cell_size:      50,
        start_pos:      Pos { x: 1, y: 4 },
        animation_len:  100,
        invincible_len: 100,
        yellow_pearls:  vec![Pos{ x:3, y:4 }, Pos { x:6, y:4 }, Pos { x:3, y:2 }, Pos { x:6, y:2 }],
        phantoms_start: vec![Pos{ x:9, y:3 }, Pos { x:5, y:3 }],
        phantoms_end:   vec![Pos{ x:0, y:0 }, Pos { x:1, y:0 }],
        walls:          vec![
            (Pos { x:2, y:4 }, Pos { x:5, y:4 }), (Pos { x:1, y:4 }, Pos { x:1, y:1 }), (Pos { x:6, y:4 }, Pos { x:9, y:4 }),
            (Pos { x:1, y:1 }, Pos { x:2, y:1 }), (Pos { x:9, y:3 }, Pos { x:10, y:3 }), (Pos { x:9, y:3 }, Pos { x:9, y:1 }), (Pos { x:8, y:3 }, Pos { x:8, y:1 }),
            (Pos { x:4, y:3 }, Pos { x:7, y:3 }), (Pos { x:2, y:3 }, Pos { x:3, y:3 }), (Pos { x:2, y:3 }, Pos { x:2, y:2 }),
            (Pos { x:3, y:2 }, Pos { x:3, y:1 }), (Pos { x:3, y:1 }, Pos { x:6, y:1 }), (Pos { x:6, y:2 }, Pos { x:6, y:1 }),
            (Pos { x:7, y:2 }, Pos { x:7, y:0 }), (Pos { x:3, y:2 }, Pos { x:4, y:2 }), (Pos { x:5, y:3 }, Pos { x:5, y:2 }),
        ]
    };
    let (walls, mut pearls) = init_board(&gsettings);
    let walls = Arc::new(walls);
    let walls_thread = walls.clone();

    let glutin_window = WindowSettings::new("ReactivePacman",
                                            (((gsettings.width+1) * gsettings.cell_size) as u32,
                                            ((gsettings.height+1) * gsettings.cell_size) as u32))
        .exit_on_esc(false).resizable(false).srgb(false)
        .build().unwrap();
    let mut window: PistonWindow<Sdl2Window> = PistonWindow::new(OpenGL::V3_2, 0, glutin_window);

    let (drawing_tx,drawing_rx) = mpsc::channel();
    let drawing_tx = Arc::new(Mutex::new(drawing_tx));
    let (action_tx,action_rx) = mpsc::channel();
    let action_rx = Arc::new(Mutex::new(action_rx));

    let mut ddata : DrawingData = DrawingData {
        pacman:   Some(gsettings.start_pos.clone()),
        phantoms: gsettings.phantoms_start.clone()
    };

    thread::spawn(move || {
        let drawing = drawing_tx.clone();
        let action  = action_rx.clone();
        let walls   = walls_thread;

        // The previous position of pacman in tiles, and the directionnal order
        let pacman_order = ValueSignal::new(Box::new(
                |a : (Pos,Directions), b : (Pos,Directions)| -> (Pos,Directions) { a }));
        // The new position in tiles of pacman, or None if pacman doesn't move
        let pacman_position = ValueSignal::new(Box::new(
                |a : Option<Pos>, b : Option<Pos>| -> Option<Pos> { if let None = a { b } else { a } }));

        let control_process = fixpoint::<(),(),_>(arrow!(
            mv x => {
                let action = action.clone();
                let action = action.lock().unwrap();
                let mut v = (Pos { x: 0, y: 0 }, Directions::None);
                if let Result::Ok(v2) = action.try_recv() {
                    /* Clean buffer */
                    while let Result::Ok(v) = action.try_recv() { };
                    v = v2;
                };
                v
            };
            emit pacman_order;
            pause;
            ret Result::Ok(())
        ));

        let pacman_process = fixpoint::<(),(),_>(arrow!(
            await pacman_order;
            mv x => {
                let walls = walls.clone();
                let (p,d) = x;
                if can_move(&p, &d, &walls) {
                    update_pos(&p, &d)
                } else {
                    Some(p)
                }
            };
            emit pacman_position;
            ret Result::Ok(())
        ));

        let draw_process = fixpoint::<(),(),_>(arrow!(
            await pacman_position;
            mv p => {
                if let None = p { () } else {
                    let drawing = drawing.clone();
                    let drawing = drawing.lock().unwrap();
                    let ddata = DrawingData {
                        pacman: p,
                        phantoms: Vec::new()
                    };
                    drawing.send(ddata).unwrap();
                }
            };
            ret Result::Ok(())
        ));

        arrow!(
            || control_process;
            || pacman_process;
            || draw_process
        ).execute_seq(());

    });
    let walls = walls.clone();

    while let Some(event) = window.next() {
        draw_board(&walls, &pearls, &gsettings, &ddata, &mut window, &event);

        event.press(|b| {
            match b {
                Button::Keyboard(k) =>
                    match k {
                        Key::Z => action_tx.send((ddata.clone().pacman.unwrap(),
                                                  Directions::Up)               ).unwrap(),
                        Key::Q => action_tx.send((ddata.clone().pacman.unwrap(),
                                                  Directions::Left)             ).unwrap(),
                        Key::S => action_tx.send((ddata.clone().pacman.unwrap(),
                                                  Directions::Down)             ).unwrap(),
                        Key::D => action_tx.send((ddata.clone().pacman.unwrap(),
                                                  Directions::Right)            ).unwrap(),
                        _      => ()
                    },
                _ => ()
            }
        });

        while let Result::Ok(data) = drawing_rx.try_recv() {
            match data.pacman {
                None    => (),
                Some(_) => ddata = data
            }
        }
    }
}

