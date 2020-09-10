#[macro_use]
mod utils;

extern crate js_sys;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    /// Get the dead and alive values of the entire Universe.
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }

    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbour_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                count += self.cells[idx] as u8;
            }
        }

        count
    }

    fn seed(bits: u32) -> Vec<u32> {
        let factor = 100_000_000_000_000_000.0;

        (0..(bits/32)).into_iter().map(|_|
            ((js_sys::Math::trunc(js_sys::Math::random() * factor) as u64) >> 32) as u32
        ).collect()
    }
}

// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    /// Set the width of the Universe
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = FixedBitSet::with_capacity((self.width * self.height) as usize);
    }

    /// Set the height of the Universe
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = FixedBitSet::with_capacity((self.height * self.width) as usize);
    }

    /// Toggles the state of a cell
    pub fn toggle(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells.toggle(idx);
    }

    /// Adds a glider centered on the specified cell
    pub fn glider(&mut self, row: u32, col: u32) {
        let cells = [(row, col), (row, col + 1), (row - 1, col - 1), (row + 1, col), (row + 1, col - 1)];

        self.set_cells(&cells);
    }

    /// Adds a pulsar centered on the specified cell
    pub fn pulsar(&mut self, row: u32, col: u32) {
        let cells = [
            (row - 1, col + 2), (row - 1, col + 3), (row - 1, col + 4),
            (row - 1, col - 2), (row - 1, col - 3), (row - 1, col - 4),
            (row - 2, col + 1), (row - 2, col + 6), (row - 2, col - 1), (row - 2, col - 6),
            (row - 3, col + 1), (row - 3, col + 6), (row - 3, col - 1), (row - 3, col - 6),
            (row - 4, col + 1), (row - 4, col + 6), (row - 4, col - 1), (row - 4, col - 6),
            (row - 6, col + 2), (row - 6, col + 3), (row - 6, col + 4),
            (row - 6, col - 2), (row - 6, col - 3), (row - 6, col - 4),
            (row + 1, col + 2), (row + 1, col + 3), (row + 1, col + 4),
            (row + 1, col - 2), (row + 1, col - 3), (row + 1, col - 4),
            (row + 2, col + 1), (row + 2, col + 6), (row + 2, col - 1), (row + 2, col - 6),
            (row + 3, col + 1), (row + 3, col + 6), (row + 3, col - 1), (row + 3, col - 6),
            (row + 4, col + 1), (row + 4, col + 6), (row + 4, col - 1), (row + 4, col - 6),
            (row + 6, col + 2), (row + 6, col + 3), (row + 6, col + 4),
            (row + 6, col - 2), (row + 6, col - 3), (row + 6, col - 4)
        ];

        self.set_cells(&cells);
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbour_count(row, col);

                next.set(idx, match (cell, live_neighbours) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise
                });
            }
        }

        self.cells = next;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width: u32 = 64;
        let height: u32 = 64;
        let capacity = (width * height) as usize;

        let cells = FixedBitSet::with_capacity_and_blocks(capacity, Self::seed(width * height));

        Universe {
            width,
            height,
            cells
        }
    }

    pub fn reset(&mut self) {
        let capacity = (self.width * self.height) as usize;

        self.cells = FixedBitSet::with_capacity_and_blocks(capacity, Self::seed(self.width * self.height));
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}
