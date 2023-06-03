use crate::grid::Grid;

pub struct Draw<'a, F: FnMut(usize, usize, bool)> {
    pub grid: &'a Grid,
    pub piece_grid: &'a Grid,
    pub piece_offset: (usize, usize),
    pub blit: F,
}

impl<F: FnMut(usize, usize, bool)> Draw<'_, F> {
    pub fn draw_grid(
        &mut self,
        (x_off, y_off): (usize, usize),
        (scale_x, scale_y): (usize, usize),
    ) {
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
                let (canvas_x, canvas_y) = (x + x_off, ((self.grid.height - y) + 1) + y_off);
                let (canvas_x, canvas_y) = (canvas_x * scale_x, canvas_y * scale_y);
                let (canvas_x, canvas_y) = (canvas_x, canvas_y);

                for x in 0..(scale_x) {
                    for y in 0..(scale_y) {
                        (self.blit)(canvas_x + x, canvas_y + y, is_set);
                    }
                }
            }
        }
    }
}
