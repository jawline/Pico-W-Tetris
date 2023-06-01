use alloc::{format, string::String, vec::Vec};
use core::{
    assert,
    ops::{Index, IndexMut},
    result::Result,
};
use itertools::iproduct;

#[derive(PartialEq, Eq)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub data: Vec<bool>,
}

impl Grid {
    pub fn of_data((width, height): (usize, usize), data: Vec<bool>) -> Self {
        assert!(width * height == data.len());
        Grid {
            width,
            height,
            data,
        }
    }
    pub fn new((width, height): (usize, usize)) -> Self {
        let mut data = Vec::new();
        data.resize(width * height, false);
        Self::of_data((width, height), data)
    }

    fn offset(&self, x: usize, y: usize) -> Result<usize, String> {
        if x < self.width && y < self.height {
            Ok((self.width * y) + x)
        } else {
            Err(format!(
                "Selected grid cell ({}, {}) exceed grid size (width={}, height={})",
                x, y, self.width, self.height
            ))
        }
    }

    pub fn get(&mut self, x: usize, y: usize) -> Result<bool, String> {
        let offset = self.offset(x, y)?;
        Ok(self.data[offset])
    }

    /**
     * Returns true of a grid at a given offset collides with another grid given the applied offset
     * to grid positions.
     *
     * E.g, \[1,0\].collides(\[0, 1\], (0, 0)) == false but \[1, 0\].collides(\[0, 1\], (1, 0)) == true.
     *
     * If one grid is larger than the other or the offset makes them not overlap, then only the
     * overlapping sections will be tested for collision.
     */
    pub fn collides(&self, other: &Self, (offset_x, offset_y): (usize, usize)) -> bool {
        for (x, y) in iproduct!(0..self.width, 0..self.height) {
            let (target_x, target_y) = (x + offset_x, y + offset_y);
            if target_x < other.width
                && target_y < other.height
                && self[(x, y)]
                && other[(target_x, target_y)]
            {
                return true;
            }
        }
        return false;
    }

    /**
     * This copies one grid into another but does not set existing true bricks at the location to
     * false. For example, if we copy [1, 0] into [1, 1] we will end up with [1, 1] rather than [1,
     * 0] but if we copy [1, 0] into [0, 1] we will also end up with [1, 1].
     *
     * If the address being copied to is not within the (width, height) of the target grid then it
     * will be ignored.
     */
    pub fn copy_into(&self, other: &mut Self, (offset_x, offset_y): (usize, usize)) {
        for (x, y) in iproduct!(0..self.width, 0..self.height) {
            let (target_x, target_y) = (x + offset_x, y + offset_y);
            if target_x < other.width && target_y < other.height && self[(x, y)] {
                other[(target_x, target_y)] = true;
            }
        }
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = bool;

    fn index(&self, (x, y): (usize, usize)) -> &bool {
        let offset = self.offset(x, y).unwrap();
        &self.data[offset]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut bool {
        let offset = self.offset(x, y).unwrap();
        &mut self.data[offset]
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::Grid;
    use alloc::vec;
    use core::assert;

    #[test]
    fn create_empty_grid() {
        Grid::new((50, 20));
    }

    #[test]
    fn fetch() {
        let _ = Grid::new((10, 10)).get(0, 0).unwrap();

        if Grid::new((10, 10)).get(11, 0).is_ok() {
            panic!("Expected this lookup to fail")
        }

        if Grid::new((10, 10)).get(0, 11).is_ok() {
            panic!("Expected this lookup to fail")
        }
    }

    #[test]
    fn set_and_get_using_index() {
        let mut grid = Grid::new((10, 10));
        grid[(0, 0)] = true;
        assert!(grid[(0, 0)] == true);
        assert!(grid[(0, 1)] == false);
        assert!(grid[(1, 0)] == false);
        grid[(0, 0)] = false;
        grid[(1, 0)] = true;
        assert!(grid[(0, 0)] == false);
        assert!(grid[(0, 1)] == false);
        assert!(grid[(1, 0)] == true);
    }

    #[test]
    fn grid_collides() {
        let all_empty = Grid::of_data((2, 2), vec![false, false, false, false]);
        let all_full = Grid::of_data((2, 2), vec![true, true, true, true]);
        let first_set = Grid::of_data((2, 2), vec![true, false, false, false]);
        let second_set = Grid::of_data((2, 2), vec![false, true, false, false]);

        // Test that all_empty never collides with all_full
        assert!(!all_empty.collides(&all_full, (0, 0)));
        assert!(!all_empty.collides(&all_full, (1, 0)));
        assert!(!all_empty.collides(&all_full, (2, 0)));
        assert!(!all_empty.collides(&all_full, (3, 0)));
        assert!(!all_empty.collides(&all_full, (0, 0)));
        assert!(!all_empty.collides(&all_full, (0, 1)));
        assert!(!all_empty.collides(&all_full, (0, 2)));
        assert!(!all_empty.collides(&all_full, (0, 3)));

        // Test that all_full, first_set, and second_set collide with themselves and that and that
        // all_full does not collide with itself if offset by it's width
        assert!(!all_empty.collides(&all_empty, (0, 0)));
        assert!(all_full.collides(&all_full, (0, 0)));
        assert!(first_set.collides(&first_set, (0, 0)));
        assert!(second_set.collides(&second_set, (0, 0)));
        assert!(!all_full.collides(&all_full, (2, 0)));
        assert!(!first_set.collides(&first_set, (1, 0)));
        assert!(!second_set.collides(&first_set, (1, 0)));

        // Test that first set can collide with second set if offset but not if unoffset
        assert!(!first_set.collides(&second_set, (0, 0)));
        assert!(first_set.collides(&second_set, (1, 0)));
    }

    #[test]
    fn grid_copy_into() {
        let mut a = Grid::new((4, 4));
        let mut b = Grid::new((4, 4));

        b[(0, 0)] = true;
        b[(1, 1)] = true;

        b.copy_into(&mut a, (0, 0));

        // Test offset by 1 in the x dim
        let mut a = Grid::new((4, 4));
        b.copy_into(&mut a, (1, 0));

        assert!(
            a == Grid::of_data(
                (4, 4),
                vec![
                    false, true, false, false, false, false, true, false, false, false, false,
                    false, false, false, false, false
                ]
            )
        );

        // Test offset by 1 in the y dim
        let mut a = Grid::new((4, 4));
        b.copy_into(&mut a, (0, 1));

        assert!(
            a == Grid::of_data(
                (4, 4),
                vec![
                    false, false, false, false, true, false, false, false, false, true, false,
                    false, false, false, false, false
                ]
            )
        );

        // Test offset off the edge of a
        let mut a = Grid::new((4, 4));
        b.copy_into(&mut a, (3, 0));

        assert!(
            a == Grid::of_data(
                (4, 4),
                vec![
                    false, false, false, true, false, false, false, false, false, false, false,
                    false, false, false, false, false
                ]
            )
        );
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_panics() {
        let grid = Grid::new((10, 10));
        assert!(grid[(11, 0)] == true);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_panics() {
        let mut grid = Grid::new((10, 10));
        grid[(11, 0)] = true;
    }
}
