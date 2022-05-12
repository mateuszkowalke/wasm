mod utils;

use js_sys::Math;
use std::fmt;
use wasm_bindgen::prelude::*;

extern crate fixedbitset;
extern crate web_sys;
use fixedbitset::FixedBitSet;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    temp_cells: FixedBitSet,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        }
    }
}

impl Universe {
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn get_row_index(&self, row: u32) -> usize {
        (row * self.width) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };
        let south = if row == self.height - 1 { 0 } else { row + 1 };
        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };
        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;
        let n = self.get_index(north, column);
        count += self.cells[n] as u8;
        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;
        let w = self.get_index(row, west);
        count += self.cells[w] as u8;
        let e = self.get_index(row, east);
        count += self.cells[e] as u8;
        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;
        let s = self.get_index(south, column);
        count += self.cells[s] as u8;
        let se = self.get_index(south, east);
        count += self.cells[se] as u8;

        count
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> Vec<Cell> {
        let mut res = vec![];
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let alive = if self.cells[idx] {
                    Cell::Alive
                } else {
                    Cell::Dead
                };
                res.push(alive);
            }
        }
        res
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        self.temp_cells.set_range(.., false);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                // log!(
                //     "cell[{}, {}] is initially {:?} and has {} live neighbors",
                //     row,
                //     col,
                //     cell,
                //     live_neighbors
                // );
                self.temp_cells.set(
                    idx,
                    match (cell, live_neighbors) {
                        (true, x) if x < 2 => false,
                        (true, 2) | (true, 3) => true,
                        (true, x) if x > 3 => false,
                        (false, 3) => true,
                        (otherwise, _) => otherwise,
                    },
                );
                // log!("    it becomes {:?}", next[idx]);
            }
        }
        self.cells.clone_from(&self.temp_cells);
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 256;
        let height = 256;
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        let temp_cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, i % 2 == 0 || i % 7 == 0);
        }
        Universe {
            width,
            height,
            cells,
            temp_cells
        }
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        cells.set_range(.., false);
        self.cells = cells;
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size = (self.width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        cells.set_range(.., false);
        self.cells = cells;
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        let cell_state = self.cells[idx];
        self.cells.set(idx, !cell_state);
    }

    pub fn reset_clear(&mut self) {
        self.cells.set_range(.., false);
    }

    pub fn reset_random(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                self.cells
                    .set(idx, if Math::random() < 0.5 { true } else { false });
            }
        }
    }

    pub fn insert_glider_at_pos(&mut self, row: u32, col: u32) {
        for d_row in [self.height - 1, 0, 1].iter().cloned() {
            for d_col in [self.width - 1, 0, 1].iter().cloned() {
                let cell_row = (d_row + row) % self.height;
                let cell_col = (d_col + col) % self.width;
                let idx = self.get_index(cell_row, cell_col);
                let is_alive = if (d_row == self.height - 1 && d_col == 0)
                    || (d_row == 0 && d_col == self.width - 1)
                    || d_row == 1
                {
                    true
                } else {
                    false
                };
                self.cells.set(idx, is_alive);
            }
        }
    }

    pub fn insert_pulsar_at_pos(&mut self, row: u32, col: u32) {
        let hor_row = [
            false, false, true, true, true, false, false, false, true, true, true, false, false,
        ];
        let ver_row = [
            true, false, false, false, false, true, false, true, false, false, false, false, true,
        ];
        let empty_row = [false; 13];
        let rows = [
            hor_row, empty_row, ver_row, ver_row, ver_row, hor_row, empty_row, hor_row, ver_row,
            ver_row, ver_row, empty_row, hor_row,
        ];
        for (d_row, row_cells) in rows.iter().cloned().enumerate() {
            let cell_row =
                (((d_row as u32 + self.height - 6) % self.height) as u32 + row) % self.height;
            let row_idx = self.get_row_index(cell_row) + col as usize;
            for idx in 0..13 {
                self.cells.set(row_idx + idx - 6, row_cells[idx])
            }
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
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

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
