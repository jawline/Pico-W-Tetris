use crate::grid::Grid;
use alloc::vec;
use core::{clone::Clone, marker::Copy, prelude::rust_2024::derive};
use enum_iterator::Sequence;
use enum_map::{enum_map, Enum, EnumMap};
use rand::prelude::*;
use rand_derive::Rand;

#[derive(Clone, Copy, Enum)]
pub enum Rotation {
    R0 = 0,
    R90 = 1,
    R180 = 2,
    R270 = 3,
}

impl Rotation {
    pub fn next(&self) -> Self {
        pub use Rotation::*;
        match self {
            R0 => R90,
            R90 => R180,
            R180 => R270,
            R270 => R0,
        }
    }
}

pub struct Piece {
    rotations: EnumMap<Rotation, Grid>,
    current_rotation: Rotation,
    pub x: usize,
    pub y: usize,
}

#[derive(Rand, Sequence)]
enum NextPiece {
    Line,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl NextPiece {
    fn to_piece(&self, (x, y): (usize, usize)) -> Piece {
        pub use Rotation::*;
        let rotations = match self {
            NextPiece::Line => {
                enum_map! {
                    R0 => Grid::of_data((4, 1), vec![true, true, true, true]),
                    R90 => Grid::of_data((1, 4), vec![true, true, true, true]),
                    R180 => Grid::of_data((4, 1), vec![true, true, true, true]),
                    R270 => Grid::of_data((1, 4), vec![true, true, true, true]),
                }
            }
            NextPiece::J => {
                enum_map! {
                    R0 => Grid::of_data((4, 2), vec![true, false, false, false, true, true, true, true]),
                    R90 => Grid::of_data((2, 4), vec![true, true, true, false, true, false, true, false]),
                    R180 => Grid::of_data((4, 2), vec![true, true, true, true, false, false, false, true]),
                    R270 => Grid::of_data((2, 4), vec![true, false, true, false, true, false, true, true]),
                }
            }

            NextPiece::L => {
                enum_map! {
                    R0 => Grid::of_data((4, 2), vec![false, false, false, true, true, true, true, true]),
                    R90 => Grid::of_data((2, 4), vec![true, true, false, true, false, true, false, true]),
                    R180 => Grid::of_data((4, 2), vec![true, true, true, true, true, false, false, false]),
                    R270 => Grid::of_data((2, 4), vec![false, true, false, true, false, true, true, true]),
                }
            }
            NextPiece::O => {
                enum_map! {
                    R0 => Grid::of_data((2, 2), vec![true, true, true, true]),
                    R90 => Grid::of_data((2, 2),vec![true, true, true, true]  ),
                    R180 => Grid::of_data((2, 2), vec![true, true, true, true] ),
                    R270 => Grid::of_data((2, 2), vec![true, true, true, true] ),
                }
            }
            NextPiece::S => {
                enum_map! {
                    R0 => Grid::of_data((3, 2), vec![false, true, true, true, true, false]),
                    R90 => Grid::of_data((2, 3), vec![true, false, true, true, false, true]),
                    R180 => Grid::of_data((3, 2), vec![true, true, false, false, true, true]),
                    R270 => Grid::of_data((2, 3), vec![false, true, true, true, true, false]),
                }
            }
            NextPiece::T => {
                enum_map! {
                    R0 => Grid::of_data((3, 2), vec![false, true, false, true, true, true]),
                    R90 => Grid::of_data((2, 3), vec![true, false, true, true, true, false]),
                    R180 => Grid::of_data((3, 2), vec![true, true, true, false, true, false]),
                    R270 => Grid::of_data((2, 3), vec![false, true, true, true, false, true]),
                }
            }
            NextPiece::Z => {
                enum_map! {
                    R0 => Grid::of_data((3, 2), vec![true, true, false, false, true, true]),
                    R90 => Grid::of_data((2, 3), vec![false, true, true, true, true, false]),
                    R180 => Grid::of_data((3, 2), vec![true, true, false, false, true, true]),
                    R270 => Grid::of_data((2, 3), vec![false, true, true, true, true, false]),
                }
            }
        };

        Piece {
            x,
            y,
            rotations,
            current_rotation: Rotation::R0,
        }
    }
}

impl Piece {
    pub fn random_piece<R: Rng>(offset: (usize, usize), rng: &mut R) -> Self {
        rng.gen::<NextPiece>().to_piece(offset)
    }

    pub fn next_rotation(&mut self) {
        self.current_rotation = self.current_rotation.next();
    }

    pub fn current_rotation(&self) -> &Grid {
        &self.rotations[self.current_rotation]
    }

    pub fn peek_next_rotation(&self) -> &Grid {
        &self.rotations[self.current_rotation.next()]
    }
}

#[cfg(test)]
mod test {
    use crate::piece::NextPiece;
    use enum_iterator::all;

    #[test]
    pub fn construct_and_rotate_every_piece() {
        for next_piece in all::<NextPiece>() {
            let mut piece = next_piece.to_piece((5, 5));
            for _ in 0..8 {
                piece.next_rotation();
            }
        }
    }
}
