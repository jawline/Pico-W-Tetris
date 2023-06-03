use std::{
    io::{stdin, stdout, Write},
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use termion::{
    clear,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};
use tetris_core::{
    grid::Grid,
    tetris::{KeyState, Tetris},
};

use drawille::Canvas;

struct OverlayView<'a> {
    grid: &'a Grid,
    piece_grid: &'a Grid,
    piece_offset: (usize, usize),
}

impl OverlayView<'_> {
    fn draw_grid(&self, canvas: &mut Canvas, (x_off, y_off): (usize, usize), (scale_x, scale_y) : (usize, usize)) {
        let (piece_x_offset, piece_y_offset) = self.piece_offset;
        for y in (0..self.grid.height).rev() {
            for x in 0..self.grid.width {
                let in_piece = {
                    if x >= piece_x_offset
                        && (x - piece_x_offset < self.piece_grid.width)
                        && y >= piece_y_offset
                        && (y - piece_y_offset < self.piece_grid.height)
                    {
                        self.piece_grid[(x - piece_x_offset, y - piece_y_offset)]
                    } else {
                        false
                    }
                };

                let is_set = self.grid[(x, y)] || in_piece;
                let (canvas_x, canvas_y) = (
                    x + x_off,
                    ((self.grid.height - y) + 1) + y_off,
                );
                let (canvas_x, canvas_y) = (canvas_x * scale_x, canvas_y * scale_y);
                let (canvas_x, canvas_y) = (canvas_x as u32, canvas_y as u32);

                    for x in 0..( scale_x as u32) {
                        for y in 0..(scale_y as u32) {

                if is_set {
                    canvas.set(canvas_x + x, canvas_y + y);
                } else {
                    canvas.unset(canvas_x + x, canvas_y + y);
                }
                        }
                    }
            }
        }
    }
}

fn draw_tetris<W: Write>(terminal: &mut RawTerminal<W>, tetris: &Tetris) {
    let mut canvas = Canvas::new(30, 30);

    match tetris {
        Tetris::Running(state) => {
            write!(terminal, "{}", termion::cursor::Goto(1 as u16, 1 as u16)).unwrap();

            OverlayView {
                grid: &state.grid,
                piece_grid: &state.piece.current_rotation(),
                piece_offset: (state.piece.x, state.piece.y),
            }
            .draw_grid(&mut canvas, (0, 0), (4, 4));
            for (idx, line) in canvas.frame().lines().enumerate() {
                write!(terminal, "{}", termion::cursor::Goto(1 as u16, 1 + idx as u16)).unwrap();
                write!(
                    terminal,
                    "{}\n",
                    line,
                ).unwrap();
            }
        }
        Tetris::Finished => write!(terminal, "Finished").unwrap(),
    }
}

fn main() {
    let mut terminal = stdout().into_raw_mode().unwrap();
    let mut tetris = Tetris::new();

    let (key_tx, key_rx) = channel();

    thread::spawn(move || {
        let keys = stdin().keys();
        for key in keys {
            key_tx.send(key.unwrap()).unwrap();
        }
    });

    'game_loop: while !tetris.is_finished() {
        write!(&mut terminal, "{}", clear::All).unwrap();

        while let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('a') => {
                    tetris.set_key_state(&KeyState {
                        left: true,
                        right: false,
                        rotate: false,
                    });
                }
                Key::Char('d') => {
                    tetris.set_key_state(&KeyState {
                        left: false,
                        right: true,
                        rotate: false,
                    });
                }
                Key::Char(' ') => {
                    tetris.set_key_state(&KeyState {
                        left: false,
                        right: false,
                        rotate: true,
                    });
                }
                Key::Ctrl('c') => {
                    println!("Exit on SIGINT");
                    break 'game_loop;
                }
                _ => {}
            }
        }

        draw_tetris(&mut terminal, &tetris);
        tetris.update();
        tetris.set_key_state(&KeyState {
            left: false,
            right: false,
            rotate: false,
        });

        thread::sleep(Duration::from_millis(250));
    }

    println!("END");
}
