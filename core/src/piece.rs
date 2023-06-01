use crate::grid::Grid;
use alloc::{boxed::Box, vec};
use core::{clone::Clone, marker::Copy, prelude::rust_2024::derive};
use enum_map::{enum_map, Enum, EnumMap};
use rand::prelude::*;
use rand_derive::Rand;

#[derive(Rand)]
enum NextPiece {
    Line,
}

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
    rotations: Box<EnumMap<Rotation, Grid>>,
    current_rotation: Rotation,
    pub x: usize,
    pub y: usize,
}

impl Piece {
    pub fn line((x, y): (usize, usize)) -> Self {
        pub use Rotation::*;
        let rotations = Box::new(enum_map! {
            R0 => Grid::of_data((4, 1), vec![true, true, true, true]),
            R90 => Grid::of_data((1, 4), vec![true, true, true, true]),
            R180 => Grid::of_data((4, 1), vec![true, true, true, true]),
            R270 => Grid::of_data((1, 4), vec![true, true, true, true]),
        });
        Piece {
            x,
            y,
            rotations,
            current_rotation: Rotation::R0,
        }
    }

    pub fn random_piece<R: Rng>((x, y): (usize, usize), rng: &mut R) -> Self {
        match rng.gen() {
            NextPiece::Line => Self::line((x, y)),
        }
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

    use crate::piece::Piece;

    #[test]
    pub fn line_rotation_through_all_slots_works() {
        Piece::line((5, 5))
            .next()
            .next()
            .next()
            .next()
            .next()
            .next()
            .next()
            .next();
    }
}
