use crate::grid::Grid;
use crate::piece::Piece;
use rand::{rngs::ThreadRng, thread_rng};

const GRID_SIZE: (usize, usize) = (10, 20);
const PIECE_START_LOCATION: (usize, usize) = (5, 19);

// The minimum score for removing a single grid piece
const BASE_SCORE_UNIT: usize = 1000;

#[derive(Clone, Copy, Default)]
pub struct KeyState {
    pub left: bool,
    pub right: bool,
    pub rotate: bool,
}

pub struct TetrisState {
    pub piece: Piece,
    pub next_piece: Piece,
    pub grid: Grid,
    pub key_state: KeyState,
    pub score: usize,
    rng: ThreadRng,
}

impl TetrisState {
    fn respawn_piece(&mut self) {
        core::mem::swap(&mut self.piece, &mut self.next_piece);
        self.next_piece = Piece::random_piece(PIECE_START_LOCATION, &mut self.rng);
    }

    fn remove_complete_rows(&mut self) {
        let mut rows_cleared = 0;

        for y in 0..self.grid.height {
            if (0..self.grid.width).find(|&x| !self.grid[(x, y)]).is_none() {
                (0..self.grid.width).for_each(|x| self.grid[(x, y)] = false);
                rows_cleared += 1;
            }
        }

        // Combo by squaring rows_cleared, you double the base row score for each additional
        // row you clear.
        self.score += (rows_cleared * rows_cleared) * self.grid.width * BASE_SCORE_UNIT;
    }
}

pub enum Tetris {
    Running(TetrisState),
    Finished,
}

impl Tetris {
    pub fn new() -> Self {
        let mut rng =  thread_rng();
        let piece = Piece::random_piece(PIECE_START_LOCATION, &mut rng);
        let next_piece = Piece::random_piece(PIECE_START_LOCATION, &mut rng);
        Self::Running(TetrisState {
            grid: Grid::new(GRID_SIZE),
            piece,
            next_piece,
            key_state: KeyState::default(),
            score: 0,
            rng
        })
    }

    pub fn set_key_state(&mut self, key_state: &KeyState) {
        match self {
            Self::Running(state) => state.key_state = *key_state,
            Self::Finished => {}
        }
    }

    pub fn update(&mut self) {
        match self {
            Self::Running(state) => {
                if state.key_state.rotate {
                    let rotated_grid = state.piece.peek_next_rotation();
                    if !rotated_grid.collides(&state.grid, (state.piece.x, state.piece.y)) {
                        state.piece.next_rotation();
                    }
                }

                // Apply any left / right move before lowering y. Do not do the move if it creates
                // a collision.
                match (state.key_state.left, state.key_state.right) {
                    (false, false) | (true, true) => {
                        // We do nothing if both keys are pushed as they net out.
                    }
                    (true, false) => {
                        if state.piece.x > 0
                            && !state
                                .piece
                                .current_rotation()
                                .collides(&state.grid, (state.piece.x - 1, state.piece.y))
                        {
                            state.piece.x -= 1;
                        }
                    }
                    (false, true) => {
                        if (state.piece.x + state.piece.current_rotation().width) < state.grid.width
                            && !state
                                .piece
                                .current_rotation()
                                .collides(&state.grid, (state.piece.x + 1, state.piece.y))
                        {
                            state.piece.x += 1;
                        }
                    }
                }

                if state.piece.y == 0
                    || state
                        .piece
                        .current_rotation()
                        .collides(&state.grid, (state.piece.x, state.piece.y - 1))
                {
                    state
                        .piece
                        .current_rotation()
                        .copy_into(&mut state.grid, (state.piece.x, state.piece.y));

                    state.remove_complete_rows();

                    state.respawn_piece();

                    // If a spawned piece immediately collides with the world then the game is lost
                    if state
                        .piece
                        .current_rotation()
                        .collides(&state.grid, (state.piece.x, state.piece.y))
                    {
                        *self = Self::Finished;
                    }
                } else {
                    state.piece.y -= 1;
                }
            }
            Self::Finished => {}
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Running(_) => false,
            Self::Finished => true,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tetris::Tetris;

    #[test]
    fn new_tetris_instance() {
        let _tetris = Tetris::new();
    }

    #[test]
    fn doing_nothing_for_a_million_updates_creates_a_finished_game() {
        let mut tetris = Tetris::new();

        for _ in 0..100_000 {
            tetris.update();
        }

        assert!(tetris.is_finished());
    }
}
