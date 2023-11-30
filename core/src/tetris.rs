use crate::grid::Grid;
use crate::piece::Piece;
use itertools::iproduct;
use rand::{SeedableRng, rngs::{SmallRng}};

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
    rng: SmallRng,
}

impl TetrisState {
    fn respawn_piece(&mut self) {
        core::mem::swap(&mut self.piece, &mut self.next_piece);
        self.next_piece = Piece::random_piece(PIECE_START_LOCATION, &mut self.rng);
    }

    /// Removes any cleared rows from the game grid after a piece has been placed down.
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

    /// Calls set_output (x + x_off, y + y_off, true|false) for every pixel in a scaled
    /// tetris grid.
    pub fn draw_game_grid<F: FnMut(usize, usize, bool)>(
        &self,
        mut set_output: F,
        (x_off, y_off): (usize, usize),
        (scale_x, scale_y): (usize, usize),
    ) {
        let (piece_x_offset, piece_y_offset) = (self.piece.x, self.piece.y);
        let piece_grid = self.piece.current_rotation();

        for (x, y) in iproduct!(0..self.grid.width, (0..self.grid.height).rev()) {
            let in_piece = {
                if x >= piece_x_offset
                    && (x - piece_x_offset < piece_grid.width)
                    && y >= piece_y_offset
                    && (y - piece_y_offset < piece_grid.height)
                {
                    piece_grid[(x - piece_x_offset, y - piece_y_offset)]
                } else {
                    false
                }
            };

            let is_set = self.grid[(x, y)] || in_piece;
            let (canvas_x, canvas_y) = (x, ((self.grid.height - 1- y)));
            let (canvas_x, canvas_y) = ((canvas_x * scale_x) + x_off, (canvas_y * scale_y) + y_off);

            for x in 0..(scale_x) {
                for y in 0..(scale_y) {
                    (set_output)(canvas_x + x, canvas_y + y, is_set);
                }
            }
        }
    }
}

pub enum Tetris {
    Running(TetrisState),
    Finished,
}

impl Tetris {
    pub fn new() -> Self {
        let mut rng = SmallRng::seed_from_u64(/* TODO: Supply with OS entropy when creating Tetris */ 31203103120);
        let piece = Piece::random_piece(PIECE_START_LOCATION, &mut rng);
        let next_piece = Piece::random_piece(PIECE_START_LOCATION, &mut rng);
        Self::Running(TetrisState {
            grid: Grid::new(GRID_SIZE),
            piece,
            next_piece,
            key_state: KeyState::default(),
            score: 0,
            rng,
        })
    }

    /// Set the current state of all inputs to the game, to be considered on all subsequent
    /// updates until the next call to set_key_state.
    pub fn set_key_state(&mut self, key_state: &KeyState) {
        match self {
            Self::Running(state) => state.key_state = *key_state,
            Self::Finished => {}
        }
    }

    /// Perform a single update of the game, first applying and input moves or rotations if legal,
    /// then attempting to lower the piece by one tile. If the lowered piece collides with an
    /// existing tile or the floor of the game grid then the piece is placed into the grid,
    /// complete rows are considered and the piece is respawned. Upon respawn if the piece
    /// immediately collides with the grid then the player has lost and the game is over.
    ///
    /// This function should be called with a frequency that matches your desired game speed,
    /// calling it more frequently will make the game faster and more difficult.
    pub fn update(&mut self) {
        match self {
            Self::Running(state) => {
                // Apply rotation if rotate key is pressed and the rotation would not collide with
                // the grid.
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
