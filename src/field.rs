pub mod cell;

use cell::Cell;
use rand::{prelude::SliceRandom, thread_rng};
use std::fmt::{Debug, Display, Formatter};

/// The enum represents all the variants of what can possibly go wrong when working with fields.
#[derive(Debug, PartialEq)]
pub enum FieldError {
    /// Used when the user tries to create a field with less than 2 cells total.
    NotEnoughCells,
    /// Used when the required amount of mines is less than 1 or is more than the total amount of cells minus 1 (there
    /// should always be at least one mine and at least one cell without a mine).
    ///
    /// The value represents the maximum allowed amount of mines for the field with the given dimensions.
    InvalidMinesAmount(u16),
    /// Used when the user tries to populate the field with mines and tells it to except some cell, but that cell's
    /// position is incorrect (i.e. the row's and/or the column's indices are out of the field's dimensions).
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
/// The field is basically a grid (a 2D vector) of cells with a known amount of mines.
#[derive(Default, PartialEq, Eq)]
pub struct Field {
    /// The grid of cells of the field. A 2D vector, where the top level represents rows and the nested vector of each
    /// row is a cell.
    grid: Vec<Vec<Cell>>,
    /// The total amount of mined cells.
    mines_amount: u16,
}

impl Field {
    /// Creates a new field with the provided dimensions and mines amount.
    ///
    /// Even though the method accepts the desired mines amount, it doesn't populate the field with them. The reason for
    /// that is that most of the time we don't want the player to click on a mined cell as their first move (i.e., we
    /// want a certain cell to be excepted from holding a mine), and the position of the cell to except would only be
    /// known after the field has been created and the user has opened their first cell.
    ///
    /// On the other hand, the configuration of the field, and, namely, the amount of mines required, happens at the
    /// same time as when configuring the field's dimensions. Therefore, it's better to know if the number of mines
    /// required is too small or too large right when creating a new field instead of when it has already been created,
    /// so that an error (if any) could be shown to the player at the configuration stage, rather than after they
    /// actually start playing.
    ///
    /// The method might fail with `FieldError::NotEnoughCells` in case the total requested field's size is less than 2
    /// cells or with `FieldError::InvalidMinesAmount` in case the requested mines amount is less than 1 or is more than
    /// the total amount of cells minus 1.
    pub fn new(rows_amount: u8, columns_amount: u8, mines_amount: u16) -> Result<Self, FieldError> {
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        if cells_amount < 2 {
            // Return an error if there are less than 2 cells total.
            Err(FieldError::NotEnoughCells)
        } else if mines_amount < 1 || mines_amount > (cells_amount - 1) {
            // Return an error if the requested amount of mines is incorrect, specifying the maximum possible amount of
            // mines that would be correct.
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

    /// Returns the field's height (rows_amount), width (columns amount) and the 2 values multiplied, which is
    /// effectively the total number of cells.
    pub fn get_size(&self) -> (u8, u8, u16) {
        let rows_amount = self.grid.len() as u8;
        let columns_amount = self.grid.first().map(|row| row.len()).unwrap_or(0) as u8;
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        (rows_amount, columns_amount, cells_amount)
    }

    /// Populates the field with randomly-distributed mines, the total amount of which is known from the time when the
    /// field was created.
    ///
    /// The method also accepts an optional parameter of a cell position to except. The excepted cell is a one that is
    /// guaranteed not to be mined.
    ///
    /// The method is guaranteed to place exactly the pre-configured amount of mines, even after (if) excepting a
    /// particular cell.
    ///
    /// As a side effect, it also increments the numerical values of the cell's adjacent cells, which represent the
    /// number of mines around an adjacent cell.
    pub fn populate_with_mines(
        &mut self,
        excepted_cell_position: Option<(u8, u8)>, // `(row_index, column_index)`
    ) -> Result<(), FieldError> {
        // Get the amount of rows, the width of a single row and the total amount of cells.
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
            .iter_mut()
            .take(self.mines_amount as usize)
            .map(|cell| cell.mine())
            .collect::<Vec<Cell>>();

        // Increment the number of mines around a cell for all the cells which are adjacent to those with mines.
        cells_with_mines
            .iter()
            // Get a mined cell's adjacent cells' positions.
            .flat_map(|cell| cell.get_adjacent_cells_positions())
            .for_each(|(row_index, column_index)| {
                if let Some(cell) = self
                    .grid
                    .get_mut(row_index as usize)
                    .and_then(|row| row.get_mut(column_index as usize))
                {
                    cell.increment_mines_around_amount();
                }
            });

        Ok(())
    }

    /// Returns a cell by its position or `None` if there's no cell at the given position.
    pub fn get_cell(&self, (row_index, column_index): (u8, u8)) -> Option<&Cell> {
        self.grid
            .get(row_index as usize)
            .and_then(|r| r.get(column_index as usize))
    }

    /// Opens the cell by its position.
    ///
    /// As a side effect, it also recursively opens all the adjacent cells to the given one if its numerical value is 0
    /// (in other words, if it has no mines in it).
    ///
    /// /// The vector of the cell's adjacent cells' positions is used to recursively open the cell's adjacent cells when
    //     /// applicable. Namely, only in the case when the cell is empty and the number of its adjacent mined cells is 0.
    //     ///
    //     /// Returning an empty vector here allows to remove all the additional checks on the field level. In other words,
    //     /// if there's something in the returned vector, then it's guaranteed that the field can safely open all the cells
    //     /// by the positions in the vector.
    //     ///
    //     /// The positions in the returned vector are guaranteed to only include the positions of the cells which do exist
    //     /// in the field.
    pub fn open_cell(&mut self, (row_index, column_index): (u8, u8)) {
        if let Some(cell) = self
            .grid
            .get_mut(row_index as usize)
            .and_then(|row| row.get_mut(column_index as usize))
        {
            // Only open the cell if it's closed and is not flagged.
            if !cell.is_open() && !cell.is_flagged() {
                cell.open();
                return;
            }

            // We only go here if the cell was opened above.
            if let Some(0) = cell.get_mines_around_amount() {
                // Return the indices of the cell's adjacent cells to recursively open them as well in the caller.
                let next_cells_to_open_positions = cell.get_adjacent_cells_positions();

                next_cells_to_open_positions
                    .iter()
                    .for_each(|cell_to_open_position| {
                        self.open_cell(*cell_to_open_position);
                    });
            }
        };
    }

    ///
    pub fn open_surrounding_cells(&mut self, (row_index, column_index): (u8, u8)) {
        // get the width and the height of the field

        if let Some(target_cell) = self.get_cell((row_index, column_index)) {
            let adjacent_cells_indices = target_cell.get_adjacent_cells_positions();

            let flagged_adjacent_cells_amount = adjacent_cells_indices
                .iter()
                .filter_map(|(row_index, column_index)| {
                    self.grid
                        .get(*row_index as usize)
                        .and_then(|row| row.get(*column_index as usize))
                })
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

    pub fn flag_cell(&mut self, (row_index, columns_index): (u8, u8)) {
        if let Some(cell) = self
            .grid
            .get_mut(row_index as usize)
            .and_then(|row| row.get_mut(columns_index as usize))
        {
            cell.toggle_flag();
        }
    }

    pub fn get_flagged_cells_amount(&self) -> u16 {
        self.grid
            .iter()
            .flatten()
            .filter(|cell| cell.is_flagged())
            .map(|_| 1)
            .sum()
    }

    pub fn check_open_mines_exist(&self) -> bool {
        self.grid
            .iter()
            .flatten()
            .any(|cell| cell.is_open() && cell.is_mined())
    }

    pub fn check_all_non_mines_open(&self) -> bool {
        self.grid
            .iter()
            .flatten()
            .filter(|cell| !cell.is_mined())
            .all(|cell| cell.is_open())
    }

    /// The method should be called when the game is lost to reveal the real positions of mines.
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

// #[cfg(test)]
// mod test {
//     use super::cell::Cell;
//     use super::{Field, FieldError};
//
//     #[test]
//     fn new_creates_a_field_with_specified_dimensions() {
//         let asymmetrical_field = Field::new(2, 3);
//
//         assert_eq!(
//             asymmetrical_field,
//             Ok(Field(vec![
//                 vec![Cell::new((0, 0)), Cell::new((0, 1)), Cell::new((0, 2))],
//                 vec![Cell::new((1, 0)), Cell::new((1, 1)), Cell::new((1, 2))],
//             ]))
//         );
//     }
//
//     #[test]
//     fn new_wont_create_a_field_with_the_size_less_than_two() {
//         let empty_field = Field::new(0, 0);
//         assert_eq!(empty_field, Err(FieldError::NotEnoughCells));
//
//         let no_rows_field = Field::new(1, 0);
//         assert_eq!(no_rows_field, Err(FieldError::NotEnoughCells));
//
//         let no_columns_field = Field::new(0, 1);
//         assert_eq!(no_columns_field, Err(FieldError::NotEnoughCells));
//
//         let one_cell_field = Field::new(1, 1);
//         assert_eq!(one_cell_field, Err(FieldError::NotEnoughCells));
//
//         let two_cells_field = Field::new(1, 2);
//         assert!(two_cells_field.is_ok());
//     }
//
//     #[test]
//     fn get_size_correctly_calculates_field_dimensions() {
//         let regular_field = Field::new(2, 3).unwrap();
//         assert_eq!(regular_field.get_size(), (2, 3, 2 * 3));
//
//         let another_field = Field::new(81, 241).unwrap();
//         assert_eq!(another_field.get_size(), (81, 241, 81 * 241));
//     }
//
//     #[test]
//     fn insert_mines_inserts_a_correct_amount_of_mines() {
//         let desired_amount_of_mines = 9;
//
//         let mut field = Field::new(9, 9).unwrap();
//         field.insert_mines(desired_amount_of_mines, None).unwrap();
//
//         let real_amount_of_mines = field
//             .0
//             .iter()
//             .flatten()
//             .map(|cell| if let true = cell.is_mined() { 1 } else { 0 })
//             .sum();
//
//         assert_eq!(desired_amount_of_mines, real_amount_of_mines);
//     }
//
//     #[test]
//     fn insert_mines_respects_the_excepted_cell() {
//         for _ in 0..1000 {
//             let mut field = Field::new(9, 9).unwrap();
//             field.insert_mines(80, Some((0, 0))).unwrap();
//
//             assert!(!field.0[0][0].is_mined());
//         }
//     }
//
//     #[test]
//     fn insert_mines_errors_when_excepted_index_is_out_of_bounds() {
//         let mut field = Field::new(9, 9).unwrap();
//         let result = field.insert_mines(9, Some((8, 9)));
//         assert_eq!(result, Err(FieldError::InvalidCellPosition((8, 9))));
//
//         let mut field = Field::new(9, 9).unwrap();
//         let result = field.insert_mines(9, Some((9, 8)));
//         assert_eq!(result, Err(FieldError::InvalidCellPosition((9, 8))));
//
//         let mut field = Field::new(9, 9).unwrap();
//         let result = field.insert_mines(9, Some((9, 9)));
//         assert_eq!(result, Err(FieldError::InvalidCellPosition((9, 9))));
//     }
//
//     #[test]
//     fn insert_mines_errors_when_less_than_one_mine_is_requested() {
//         let mut field = Field::new(9, 9).unwrap();
//         let result = field.insert_mines(0, None);
//         assert_eq!(result, Err(FieldError::InvalidMinesAmount));
//     }
//
//     #[test]
//     fn insert_mines_errors_when_too_many_mines_are_requested() {
//         let mut field = Field::new(9, 9).unwrap();
//         let result = field.insert_mines(81, None);
//         assert_eq!(result, Err(FieldError::InvalidMinesAmount));
//     }
//
//     #[test]
//     fn insert_mines_errors_on_double_invocation() {
//         let mut field = Field::new(9, 9).unwrap();
//         field.insert_mines(9, Some((0, 0))).unwrap();
//
//         let result = field.insert_mines(9, None);
//         assert_eq!(result, Err(FieldError::MinesAlreadyExist));
//     }
//
//     #[test]
//     fn insert_numbers_correctly_inserts_numbers() {
//         fn b(position: (u8, u8)) -> Cell {
//             // build a mined cell (a bomb)
//             let mut cell = Cell::new(position);
//             cell.mine();
//             cell
//         }
//
//         fn n(position: (u8, u8), number: u8) -> Cell {
//             // builds a cell with a number
//             let mut cell = Cell::new(position);
//             (0..number).for_each(|_| cell.increment_adjacent_mines_amount());
//             cell
//         }
//
//         let mut field_without_numbers = Field(vec![
//             vec![b((0, 0)), n((0, 1), 0), n((0, 2), 0)],
//             vec![n((1, 0), 0), n((1, 1), 0), b((1, 2))],
//             vec![n((2, 0), 0), n((2, 1), 0), n((2, 2), 0)],
//         ]);
//
//         field_without_numbers.insert_numbers(vec![
//             field_without_numbers.0[0][0],
//             field_without_numbers.0[1][2],
//         ]);
//
//         let reference = Field(vec![
//             vec![b((0, 0)), n((0, 1), 2), n((0, 2), 1)],
//             vec![n((1, 0), 1), n((1, 1), 2), b((1, 2))],
//             vec![n((2, 0), 0), n((2, 1), 1), n((2, 2), 1)],
//         ]);
//
//         assert_eq!(field_without_numbers, reference);
//     }
//
//     fn helper_assert_for_cells(
//         field: Field,
//         cells_positions: Vec<(u8, u8)>,
//         should_be_open: bool,
//     ) -> Field {
//         cells_positions
//             .iter()
//             .for_each(|(row_index, column_index)| {
//                 assert_eq!(
//                     field.0[*row_index as usize][*column_index as usize].is_open(),
//                     should_be_open
//                 );
//             });
//
//         field
//     }
//
//     #[test]
//     fn open_cell_correctly_opens_cell_when_the_target_cell_is_empty() {
//         // Such a field, when opening the (1, 1) cell
//         /*
//         ⬜  ⬜  ⬜  1️ 💣
//         ⬜  ⬜  ⬜  2️ 2️
//         ⬜  ⬜  ⬜  1️ 💣
//         ⬜  1️  1️  2️ 1️
//         ⬜  1️  💣  1️ ⬜
//          */
//
//         // must produce the following result
//         /*
//         ⬜  ⬜  ⬜ 1️  ⬛
//         ⬜  ⬜  ⬜ 2️  ⬛
//         ⬜  ⬜  ⬜ 1️  ⬛
//         ⬜  1️ 1️  2️  ⬛
//         ⬜  1️ ⬛  ⬛  ⬛
//          */
//
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][4].mine();
//         field.0[2][4].mine();
//         field.0[4][2].mine();
//         field.0[0][3].increment_adjacent_mines_amount(); // 1
//         field.0[1][3].increment_adjacent_mines_amount(); // 1...
//         field.0[1][3].increment_adjacent_mines_amount(); // 2
//         field.0[2][3].increment_adjacent_mines_amount(); // 1
//         field.0[3][1].increment_adjacent_mines_amount(); // 1
//         field.0[3][2].increment_adjacent_mines_amount(); // 1
//         field.0[3][3].increment_adjacent_mines_amount(); // 1...
//         field.0[3][3].increment_adjacent_mines_amount(); // 2
//         field.0[4][1].increment_adjacent_mines_amount(); // 1
//
//         field.open_cell((1, 1));
//
//         let cells_that_should_be_opened_positions = [
//             (0u8, 0u8),
//             (0, 1),
//             (0, 2),
//             (0, 3),
//             (1, 0),
//             (1, 1),
//             (1, 2),
//             (1, 3),
//             (2, 0),
//             (2, 1),
//             (2, 2),
//             (2, 3),
//             (3, 0),
//             (3, 1),
//             (3, 2),
//             (3, 3),
//             (4, 0),
//             (4, 1),
//         ];
//
//         field =
//             helper_assert_for_cells(field, cells_that_should_be_opened_positions.to_vec(), true);
//
//         let cells_that_should_be_closed_positions =
//             [(0, 4), (1, 4), (2, 4), (3, 4), (4, 2), (4, 3), (4, 4)];
//
//         assert_eq!(
//             cells_that_should_be_opened_positions.len()
//                 + cells_that_should_be_closed_positions.len(),
//             field.get_size().2 as usize
//         );
//
//         helper_assert_for_cells(field, cells_that_should_be_closed_positions.to_vec(), false);
//     }
//
//     #[test]
//     fn open_cell_correctly_opens_cell_when_the_target_cell_contains_a_number() {
//         // Such a field, when opening the (1, 1) cell
//         /*
//         ⬜  1️  💣  2️  💣
//         ⬜  1️  1️  2️  1️
//         ⬜  ⬜  ⬜  ⬜  ⬜
//         1️  1️  1️  ⬜  ⬜
//         1️  💣  1️  ⬜  ⬜
//          */
//
//         // must produce the following result
//         /*
//         ⬛  ⬛  ⬛  ⬛  ⬛
//         ⬛  1️  ⬛  ⬛  ⬛
//         ⬛  ⬛  ⬛  ⬛  ⬛
//         ⬛  ⬛  ⬛  ⬛  ⬛
//         ⬛  ⬛  ⬛  ⬛  ⬛
//          */
//
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][2].mine();
//         field.0[0][4].mine();
//         field.0[4][1].mine();
//         // we only care about the scenario when the target cell is a number
//         field.0[1][1].increment_adjacent_mines_amount(); // 1
//
//         field.open_cell((1, 1));
//
//         let cells_that_should_be_opened_positions = [(1, 1)];
//
//         field =
//             helper_assert_for_cells(field, cells_that_should_be_opened_positions.to_vec(), true);
//
//         let cells_that_should_be_closed_positions = [
//             (0, 0),
//             (0, 1),
//             (0, 2),
//             (0, 3),
//             (0, 4),
//             (1, 0),
//             (1, 2),
//             (1, 3),
//             (1, 4),
//             (2, 0),
//             (2, 1),
//             (2, 2),
//             (2, 3),
//             (2, 4),
//             (3, 0),
//             (3, 1),
//             (3, 2),
//             (3, 3),
//             (3, 4),
//             (4, 0),
//             (4, 1),
//             (4, 2),
//             (4, 3),
//             (4, 4),
//         ];
//
//         assert_eq!(
//             cells_that_should_be_opened_positions.len()
//                 + cells_that_should_be_closed_positions.len(),
//             field.get_size().2 as usize
//         );
//
//         helper_assert_for_cells(field, cells_that_should_be_closed_positions.to_vec(), false);
//     }
//
//     #[test]
//     fn open_surrounding_cells_correctly_opens_surrounding_cells() {
//         // let assume we have a 5*5 field where at the (0, 0) position is the only mine. Currently we're as the
//         // state after the the first move, where the user has just opened the cell (1, 0). Next, they flag the (0, 0)
//         // cell and open the surrounding cells for (1, 0). This should effectively open all the cells.
//
//         // create the filed with one mine a properly distributed numbers
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][0].mine();
//         field.insert_numbers(vec![field.0[0][0]]);
//
//         // open the (1, 0) cell
//         field.open_cell((1, 0));
//
//         // flag the mine
//         field.flag_cell((0, 0));
//
//         // open the surrounding cells for the opened (1, 0)
//         field.open_surrounding_cells((1, 0));
//
//         // all the cells except for the one with the mine should be opened now
//         let all_cells_opened = field.0.iter().flatten().skip(1).all(|cell| cell.is_open());
//
//         assert!(all_cells_opened);
//     }
//
//     #[test]
//     fn open_surrounding_cells_does_not_produce_no_effect_when_the_target_cell_is_closed() {
//         // the same field from the above, but this time we won't open the (1, 0) cells and will then try to open its
//         // surrounding cells. This should fail (by not producing any effect)
//
//         // create the filed with one mine a properly distributed numbers
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][0].mine();
//         field.insert_numbers(vec![field.0[0][0]]);
//
//         // don't open the (1, 0) cell
//         // field.open_cell((1, 0));
//
//         // flag the mine
//         field.flag_cell((0, 0));
//
//         // open the surrounding cells for the close (1, 0)
//         field.open_surrounding_cells((1, 0));
//
//         // all the cells should remain closed (the first one - the one with the mine - should remain `Flagged`)
//         let all_cells_closed = field.0.iter().flatten().skip(1).all(|cell| !cell.is_open());
//
//         assert!(all_cells_closed);
//     }
//
//     #[test]
//     fn open_surrounding_cells_does_not_produce_no_effect_when_the_target_cell_is_flagged() {
//         // the same field from the above, but this time we flag the mined (0, 0) cell and trying to open the surrounding
//         // cells for it. This attempt should fail by not producing any effect
//
//         // create the filed with one mine a properly distributed numbers
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][0].mine();
//         field.insert_numbers(vec![field.0[0][0]]);
//
//         // don't open the (1, 0) cell
//         // field.open_cell((1, 0));
//
//         // flag the mine
//         field.flag_cell((0, 0));
//
//         // open the surrounding cells for it
//         field.open_surrounding_cells((0, 0));
//
//         // all the cells should remain closed (the first one - the one with the mine - should remain `Flagged`)
//         let all_cells_closed = field.0.iter().flatten().skip(1).all(|cell| !cell.is_open());
//
//         assert!(all_cells_closed);
//     }
//
//     #[test]
//     fn open_surrounding_cells_does_not_produce_no_effect_when_the_flags_number_mismatch_the_mines_around_amount(
//     ) {
//         fn calc_num_of_closed_cells(field: &Field) -> usize {
//             field
//                 .0
//                 .iter()
//                 .flatten()
//                 .filter(|cell| !cell.is_open() && !cell.is_flagged())
//                 .collect::<Vec<&Cell>>()
//                 .len()
//         }
//
//         // the same field, but this time opening the surrounding cells should fail (not produce eny effect) because
//         // 1. For the first part of the test there won't be any flags at all
//         // 2. For the second part of the test there will be two flags instead of one
//
//         // create the filed with one mine a properly distributed numbers
//         let mut field = Field::new(5, 5).unwrap();
//         field.0[0][0].mine();
//         field.insert_numbers(vec![field.0[0][0]]);
//
//         // open the (1, 0) cell
//         field.open_cell((1, 0));
//
//         // don't flag the mine
//         // field.flag_cell((0, 0));
//
//         // open the surrounding cells for the opened (1, 0)
//         field.open_surrounding_cells((1, 0));
//
//         // all the cells except for the one with that was opened should be closed
//         let number_of_closed_cells = calc_num_of_closed_cells(&field);
//
//         assert_eq!(number_of_closed_cells, 24);
//
//         // SECOND PART //
//
//         // flag two cells
//         field.flag_cell((0, 0));
//         field.flag_cell((0, 1));
//
//         // open the surrounding cells for the opened (1, 0)
//         field.open_surrounding_cells((1, 0));
//
//         // all the cells except for the one with that was opened should be closed
//         let number_of_closed_cells = calc_num_of_closed_cells(&field);
//
//         // should be 22 because one cell has been opened and 2 were flagged: 25 - 1 - 2 is 22
//         assert_eq!(number_of_closed_cells, 22);
//     }
//
//     #[test]
//     fn flag_cell_correctly_toggles_the_flag() {
//         let mut field = Field::new(20, 20).unwrap();
//
//         field.flag_cell((0, 0));
//         assert!(field.0[0][0].is_flagged());
//
//         field.flag_cell((0, 0));
//         assert!(!field.0[0][0].is_open() && !field.0[0][0].is_flagged());
//     }
//
//     #[test]
//     fn check_open_mines_exist_correctly_indicates_the_presence_of_mines() {
//         let mut field = Field::new(20, 20).unwrap();
//         let mut mine = Cell::new((0, 0));
//         mine.mine();
//         field.0[0][0] = mine;
//         field.open_cell((0, 0));
//
//         assert!(field.check_open_mines_exist());
//     }
//
//     #[test]
//     fn check_open_mines_exist_does_not_false_trigger() {
//         let mut field = Field::new(20, 20).unwrap();
//         let mut mine = Cell::new((0, 0));
//         mine.mine();
//         field.0[0][0] = mine;
//         // field.open_cell((0, 0)); // don't open the mined cell
//
//         assert!(!field.check_open_mines_exist());
//     }
//
//     #[test]
//     fn check_all_non_mines_open_is_true_when_all_non_mines_are_open() {
//         let mut field = Field::new(20, 20).unwrap();
//         let mut mine = Cell::new((0, 0));
//         mine.mine();
//         field.0[0][0] = mine;
//         field.open_cell((19, 19)); // this opens all the non-mined cells consecutively
//
//         assert!(field.check_all_non_mines_open());
//     }
//
//     #[test]
//     fn check_all_non_mines_open_does_not_false_trigger() {
//         let mut field = Field::new(20, 20).unwrap();
//         let mut mine = Cell::new((0, 0));
//         mine.mine();
//         field.0[0][0] = mine;
//         // field.open_cell((19, 19)); // don't open the empty cell, so everything remains closed
//
//         assert!(!field.check_all_non_mines_open());
//     }
// }
