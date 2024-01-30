mod cell;
use cell::Cell;

use rand::{prelude::SliceRandom, thread_rng};
use std::fmt::{Debug, Display, Formatter};

/// The enum represents all the variants of what can possibly go wrong when working with fields.
#[derive(Debug, PartialEq)]
pub enum FieldError {
    /// Used when the user tries to create a field with less than two cells total.
    NotEnoughCells,
    /// Used when the required number of mines is less than 1 or is more than the total number of cells minus 1 (there
    /// should always be at least one mine and at least one cell without a mine).
    ///
    /// The value represents the maximum allowed number of mines for the field with the given dimensions.
    InvalidMinesAmount(u16),
    /// Used when the user tries to populate the field with mines and tells it to except some cell, but that cell's
    /// position is incorrect (i.e., the row's and/or the column's indices are out of the field's dimensions).
    ///
    /// The value represents the requested-to-except cell's row and column indices respectively.
    InvalidExceptedCellPosition((u8, u8)),
    /// Used when trying to populate with mines a field that has already been populated with them.
    ///
    /// The restriction is implied to avoid accidentally re-distributing the mines of a field of an ongoing game.
    MinesAlreadyExist,
}

/// The field representation.
///
/// The field is basically a grid (a 2D vector) of cells with a known number of mines.
#[derive(Default, PartialEq, Eq)]
pub struct Field {
    /// The grid of cells of the field. A 2D vector, where the top level represents rows, and the nested vector of each
    /// row represents a cell.
    grid: Vec<Vec<Cell>>,
    /// The total number of mined cells.
    mines_amount: u16,
}

impl Field {
    /// Creates a new field with the provided dimensions and number of mines.
    ///
    /// Even though the method accepts the desired mines amount, it doesn't populate the field with them. The reason for
    /// that is that most of the time we don't want the player to click on a mined cell as their first move (i.e., we
    /// want a certain cell to be excepted from holding a mine), and the position of the cell to except would only be
    /// known after the field has been created and the user has opened their first cell.
    ///
    /// On the other hand, the configuration of the field, and, namely, the number of mines required, happens at the
    /// same time as when configuring the field's dimensions. Therefore, it's better to know if the number of mines
    /// required is too small or too large right when creating a new field instead of when it has already been created,
    /// so that an error (if any) could be shown to the player at the configuration stage, rather than after they
    /// actually start playing.
    ///
    /// The method might fail with `FieldError::NotEnoughCells` in case the total requested field's size is less than
    /// two cells or with `FieldError::InvalidMinesAmount` in case the requested mines amount is less than 1 or is more
    /// than the total number of cells minus 1.
    pub fn new(rows_amount: u8, columns_amount: u8, mines_amount: u16) -> Result<Self, FieldError> {
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        if cells_amount < 2 {
            // Return an error if there are less than 2 cells total.
            Err(FieldError::NotEnoughCells)
        } else if mines_amount < 1 || mines_amount > (cells_amount - 1) {
            // Return an error if the requested number of mines is incorrect, specifying the maximum possible number of
            // mines that would be correct for a field with the same dimensions.
            Err(FieldError::InvalidMinesAmount(cells_amount - 1))
        } else {
            let grid = (0..rows_amount)
                .map(|row_index| {
                    (0..columns_amount)
                        .map(|column_index| Cell::new((row_index, column_index)))
                        .collect()
                })
                .collect();

            Ok(Field { grid, mines_amount })
        }
    }

    /// Populates the field with randomly distributed mines, the total amount of which is known from the time when the
    /// field was created.
    ///
    /// The method also accepts an optional parameter of a cell position to except. The excepted cell is a one that is
    /// guaranteed not to be mined.
    ///
    /// The method is guaranteed to place exactly the pre-configured number of mines, even after (if) excepting a
    /// particular cell.
    ///
    /// As a side effect, it also increments the numerical values of the mined cells' adjacent cells, which represent
    /// the number of mines around an adjacent cell.
    pub fn populate_with_mines(
        &mut self,
        excepted_cell_position: Option<(u8, u8)>, // `(row_index, column_index)`
    ) -> Result<(), FieldError> {
        // Get the number of rows and the width of a single row.
        let (rows_amount, columns_amount, _) = self.get_size();

        // Return an error if the provided excepted cell's position goes beyond the field's dimensions.
        if let Some((row_index, column_index)) = excepted_cell_position {
            if (row_index > rows_amount - 1) || (column_index > columns_amount - 1) {
                return Err(FieldError::InvalidExceptedCellPosition((
                    row_index,
                    column_index,
                )));
            }
        }

        // Flatten the field for an easier interaction with it.
        let mut flattened_field = self.grid.iter_mut().flatten().collect::<Vec<&mut Cell>>();

        // Return an error if there are mines already: can't populate with mines a field that's already been populated.
        if flattened_field.iter().any(|cell| cell.is_mined()) {
            return Err(FieldError::MinesAlreadyExist);
        }

        // Remove the reference to the excepted cell (if any) to avoid marking it as mined.
        if let Some((row_index, column_index)) = excepted_cell_position {
            // `row_index * columns_amount + column_index` is a formula of getting a 1D flattened-vector's index based
            // on the original 2D vector's coordinates.
            flattened_field.remove((row_index * columns_amount + column_index) as usize);
        }

        // Shuffle the mutable borrowings to randomly distribute the mines.
        let mut rng = thread_rng();
        flattened_field.shuffle(&mut rng);

        // Fill the first `number_of_mines` cells with mines and store them in a vector.
        let cells_with_mines = flattened_field
            .into_iter()
            .take(self.mines_amount as usize)
            .map(|cell| {
                cell.mine();
                cell
            })
            .collect::<Vec<&mut Cell>>();

        // Get a flat vector of all the mined cells' adjacent cells.
        let adjacent_cells_positions = cells_with_mines
            .into_iter()
            // Get a mined cell's adjacent cells' positions.
            .flat_map(|cell| cell.get_adjacent_cells_positions())
            .collect::<Vec<(u8, u8)>>();

        // For each of the adjacent cells, increment their numerical value, representing the quantity of mines around
        // them.
        adjacent_cells_positions
            .into_iter()
            .for_each(|(row_index, column_index)| {
                if let Some(cell) = self.get_cell_mut((row_index, column_index)) {
                    cell.increment_mines_around_amount();
                }
            });

        Ok(())
    }

    /// Returns the field's height (the number of rows), width (the number of columns) and the two values multiplied,
    /// which is effectively the total number of cells.
    pub fn get_size(&self) -> (u8, u8, u16) {
        let rows_amount = self.grid.len() as u8;
        let columns_amount = self.grid.first().map(|row| row.len()).unwrap_or(0) as u8;
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        (rows_amount, columns_amount, cells_amount)
    }

    /// Returns a read-only cell reference by its position or `None` if there's no cell at the given position.
    pub fn get_cell(&self, (row_index, column_index): (u8, u8)) -> Option<&Cell> {
        self.grid
            .get(row_index as usize)
            .and_then(|r| r.get(column_index as usize))
    }

    /// Returns a mutable cell reference by its position or `None` if there's no cell at the given position.
    pub fn get_cell_mut(&mut self, (row_index, column_index): (u8, u8)) -> Option<&mut Cell> {
        self.grid
            .get_mut(row_index as usize)
            .and_then(|r| r.get_mut(column_index as usize))
    }

    /// Opens the cell by its position.
    ///
    /// As a side effect, it also recursively opens all the adjacent cells to the given one if its numerical value is 0
    /// (if the target cell has no mines in it, to put it simpler).
    pub fn open_cell(&mut self, (row_index, column_index): (u8, u8)) {
        if let Some(cell) = self.get_cell_mut((row_index, column_index)) {
            if !cell.is_open() && !cell.is_flagged() {
                cell.open();
            } else {
                return;
            }

            if let Some(0) = cell.get_mines_around_amount() {
                cell.get_adjacent_cells_positions()
                    .into_iter()
                    .for_each(|cell_position| self.open_cell(cell_position));
            }
        }
    }

    /// Opens all the cells surrounding the target one.
    ///
    /// Is an equivalent of the middle-click in the original
    /// implementation.
    ///
    /// Won't produce any effect if the target cell is closed or flagged or if its numerical value is not the same
    /// as the number of flags placed around it.
    pub fn open_surrounding_cells(&mut self, (row_index, column_index): (u8, u8)) {
        // get the width and the height of the field

        if let Some(target_cell) = self.get_cell((row_index, column_index)) {
            let adjacent_cells_indices = target_cell.get_adjacent_cells_positions();

            let flagged_adjacent_cells_amount = adjacent_cells_indices
                .into_iter()
                .filter_map(|(row_index, column_index)| self.get_cell((row_index, column_index)))
                .filter(|adjacent_cell| adjacent_cell.is_flagged())
                .collect::<Vec<&Cell>>()
                .len() as u8;

            if target_cell.is_open()
                && target_cell.get_mines_around_amount().is_some()
                && flagged_adjacent_cells_amount == target_cell.get_mines_around_amount().unwrap()
            {
                adjacent_cells_indices
                    .into_iter()
                    .for_each(|adjacent_cell_position| {
                        self.open_cell(adjacent_cell_position);
                    });
            };
        }
    }

    /// Toggles flag for the cell (if any) with the given position.
    pub fn toggle_flag_for_cell(&mut self, (row_index, columns_index): (u8, u8)) {
        if let Some(cell) = self.get_cell_mut((row_index, columns_index)) {
            cell.toggle_flag();
        }
    }

    /// The method returns the total number of all the currently flagged cells in the field.
    ///
    /// A use case might be displaying the in-game statistics.
    pub fn get_flagged_cells_amount(&self) -> u16 {
        self.grid
            .iter()
            .flatten()
            .filter(|cell| cell.is_flagged())
            .map(|_| 1)
            .sum()
    }

    /// Checks that there exists at least one mined cell which is open.
    ///
    /// This is effectively the loss-condition for the game.
    pub fn check_open_mines_exist(&self) -> bool {
        self.grid
            .iter()
            .flatten()
            .any(|cell| cell.is_open() && cell.is_mined())
    }

    /// Checks that all the empty cells are open.
    ///
    /// This is effectively the win-condition for the game.
    pub fn check_all_non_mines_open(&self) -> bool {
        self.grid
            .iter()
            .flatten()
            .filter(|cell| !cell.is_mined())
            .all(|cell| cell.is_open())
    }

    /// Opens all the yet-not-flagged cells with mines.
    ///
    /// The method should be called when the game is already lost to reveal the real positions of mines.
    pub fn open_missed_mines(&mut self) {
        self.grid
            .iter_mut()
            .flatten()
            .filter(|cell| cell.is_mined() && !cell.is_flagged())
            .for_each(|cell| {
                cell.open();
            });
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.grid.iter() {
            for cell in row {
                write!(f, "{:?} ", cell)?;
            }

            writeln!(f)?;
        }

        write!(f, "")
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.grid[0].iter().enumerate().for_each(|(i, _)| {
            write!(
                f,
                "{:^3}",
                if i == 0 {
                    "    0 ".to_string()
                } else {
                    i.to_string()
                }
            )
            .unwrap();
        });

        writeln!(f)?;

        for (i, row) in self.grid.iter().enumerate() {
            write!(f, "{:^3}", i).unwrap();

            for cell in row {
                write!(f, "{} ", cell)?;
            }

            writeln!(f)?;
        }

        write!(f, "")
    }
}
