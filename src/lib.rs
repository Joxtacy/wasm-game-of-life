mod utils;

use js_sys;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    buffer_cells: Vec<Cell>,
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
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
        count += self.buffer_cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.buffer_cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.buffer_cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.buffer_cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.buffer_cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.buffer_cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.buffer_cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.buffer_cells[se] as u8;

        count
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
            self.buffer_cells[idx] = Cell::Alive;
        }
    }
}

fn generate_cells_static(i: u32) -> Cell {
    if i % 2 == 0 || i % 7 == 0 {
        Cell::Alive
    } else {
        Cell::Dead
    }
}

fn generate_cells_random(_i: u32) -> Cell {
    if js_sys::Math::random() < 0.5 {
        Cell::Alive
    } else {
        Cell::Dead
    }
}

fn generate_cells_dead(_i: u32) -> Cell {
    Cell::Dead
}

#[wasm_bindgen]
impl Universe {
    pub fn new(height: u32, width: u32) -> Universe {
        // utils::set_panic_hook();

        if height == 0 || width == 0 {
            panic!("Cannot create universe with 0 size");
        }

        let cells: Vec<Cell> = (0..width * height).map(generate_cells_static).collect();
        let buffer_cells = cells.clone();

        Universe {
            width,
            height,
            cells,
            buffer_cells,
        }
    }

    pub fn new_random(height: u32, width: u32) -> Universe {
        utils::set_panic_hook();

        if height == 0 || width == 0 {
            panic!("Cannot create universe with 0 size");
        }

        let cells: Vec<Cell> = (0..width * height).map(generate_cells_random).collect();
        let buffer_cells = cells.clone();

        Universe {
            width,
            height,
            cells,
            buffer_cells,
        }
    }

    pub fn new_dead(height: u32, width: u32) -> Universe {
        utils::set_panic_hook();

        if height == 0 || width == 0 {
            panic!("Cannot create universe with 0 size");
        }

        let cells: Vec<Cell> = (0..width * height).map(generate_cells_dead).collect();
        let buffer_cells = cells.clone();

        Universe {
            width,
            height,
            cells,
            buffer_cells,
        }
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        let mut coords: Vec<(u32, u32)> = Vec::new();

        coords.push((row - 1, column - 1));
        coords.push((row, column));
        coords.push((row, column + 1));
        coords.push((row + 1, column - 1));
        coords.push((row + 1, column));

        coords = coords
            .iter()
            .map(|coord| (coord.0 % self.height, coord.1 % self.width))
            .collect();
        self.set_cells(&coords);
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        let mut coords: Vec<(u32, u32)> = Vec::new();

        coords.push((row - 6, column - 4));
        coords.push((row - 6, column - 3));
        coords.push((row - 6, column - 2));
        coords.push((row - 6, column + 4));
        coords.push((row - 6, column + 3));
        coords.push((row - 6, column + 2));

        coords.push((row + 6, column - 4));
        coords.push((row + 6, column - 3));
        coords.push((row + 6, column - 2));
        coords.push((row + 6, column + 4));
        coords.push((row + 6, column + 3));
        coords.push((row + 6, column + 2));

        coords.push((row - 1, column - 4));
        coords.push((row - 1, column - 3));
        coords.push((row - 1, column - 2));
        coords.push((row - 1, column + 4));
        coords.push((row - 1, column + 3));
        coords.push((row - 1, column + 2));

        coords.push((row + 1, column - 4));
        coords.push((row + 1, column - 3));
        coords.push((row + 1, column - 2));
        coords.push((row + 1, column + 4));
        coords.push((row + 1, column + 3));
        coords.push((row + 1, column + 2));

        coords.push((row - 4, column - 6));
        coords.push((row - 3, column - 6));
        coords.push((row - 2, column - 6));
        coords.push((row + 4, column - 6));
        coords.push((row + 3, column - 6));
        coords.push((row + 2, column - 6));

        coords.push((row - 4, column + 6));
        coords.push((row - 3, column + 6));
        coords.push((row - 2, column + 6));
        coords.push((row + 4, column + 6));
        coords.push((row + 3, column + 6));
        coords.push((row + 2, column + 6));

        coords.push((row - 4, column - 1));
        coords.push((row - 3, column - 1));
        coords.push((row - 2, column - 1));
        coords.push((row + 4, column - 1));
        coords.push((row + 3, column - 1));
        coords.push((row + 2, column - 1));

        coords.push((row - 4, column + 1));
        coords.push((row - 3, column + 1));
        coords.push((row - 2, column + 1));
        coords.push((row + 4, column + 1));
        coords.push((row + 3, column + 1));
        coords.push((row + 2, column + 1));

        coords = coords
            .iter()
            .map(|coord| (coord.0 % self.height, coord.1 % self.width))
            .collect();
        self.set_cells(&coords);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead cell state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..self.width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead cell state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");


        {
            let _timer = Timer::new("new generation");

            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.buffer_cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    /*
                    log!(
                    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    cell,
                    live_neighbors
                    );
                    */

                    let next_cell = match (cell, live_neighbors) {
                        // Rule 1: Any live cell with fewer than two live neighbours
                        // dies, as if caused by underpopulation.
                        (Cell::Alive, x) if x < 2 => Cell::Dead,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                        // Rule 3: Any live cell with more than three live
                        // neighbours dies, as if by overpopulation.
                        (Cell::Alive, x) if x > 3 => Cell::Dead,
                        // Rule 4: Any dead cell with exactly three live neighbours
                        // becomes a live cell, as if by reproduction.
                        (Cell::Dead, 3) => Cell::Alive,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    };

                    // log!("    it becomes {:?}", next_cell);

                    self.cells[idx] = next_cell;
                }
            }

            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let next_cell = match cell {
                        Cell::Alive => Cell::Alive,
                        Cell::Dead => Cell::Dead,
                    };
                    self.buffer_cells[idx] = next_cell;
                }
            }
        }

        let _timer = Timer::new("free old cells");
    }
}
