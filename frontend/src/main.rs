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
    draw_helper::Draw,
};

use drawille::Canvas;


fn draw_tetris<W: Write>(terminal: &mut RawTerminal<W>, tetris: &Tetris) {
    let mut canvas = Canvas::new(30, 30);

    match tetris {
        Tetris::Running(state) => {
            write!(terminal, "{}", termion::cursor::Goto(1 as u16, 1 as u16)).unwrap();

            Draw {
                grid: &state.grid,
                piece_grid: &state.piece.current_rotation(),
                piece_offset: (state.piece.x, state.piece.y),
                blit: |x, y, state| {
                    if state {
                        canvas.set(x as u32, y as u32);
                    } else {
                        canvas.unset(x as u32, y as u32);
                    }
                }
            }
            .draw_grid((0, 0), (4, 4));
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
