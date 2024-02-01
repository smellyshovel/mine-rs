pub mod cell;

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
    /// position is incorrect (i.e., the row's and/or the column's indices are beyond the field's bounds).
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
#[derive(PartialEq, Eq)]
pub struct Field {
    /// The grid of cells of the field. A 2D vector, where the top level represents rows, and the nested vector of each
    /// row represents cells.
    grid: Vec<Vec<Cell>>,
    /// The total number of mined cells.
    mines_amount: u16,
}

impl Field {
    /// Creates a new [`Field`] with the provided dimensions and number of mines.
    ///
    /// Even though the method accepts the desired mines amount, it doesn't populate the field with them. The reason for
    /// that is that most of the time we don't want the player to click on a mined cell as their first move (i.e., we
    /// want a certain cell to be excepted from holding a mine), and the position of the cell to except would only be
    /// known after the field has been created and the user has requested to open their first cell.
    ///
    /// On the other hand, the configuration of the field, and, namely, the number of mines required, happens at the
    /// same time as when configuring the field's dimensions. Therefore, it's better to know if the number of mines
    /// required is too small or too large right when creating a new field instead of when it has already been created,
    /// so that an error (if any) could be shown to the player at the configuration stage, rather than after they
    /// actually start playing.
    ///
    /// The method might fail with [`FieldError::NotEnoughCells`] in case the total requested field's size is less than
    /// two cells or with [`FieldError::InvalidMinesAmount`] in case the requested mines amount is less than one or is
    /// more than the total number of cells minus 1.
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
    /// As a side effect, it also calls the `self::update_mines_around_values` method.
    ///
    /// The method might fail with [`FieldError::InvalidExceptedCellPosition`] in case the excepted row's and/or
    /// column's indices are beyond the field's bounds or with [`FieldError::MinesAlreadyExist`] in case the method is
    /// called when there are mines in the field already.
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

        // Fill the first `number_of_mines` cells with mines.
        flattened_field
            .into_iter()
            .take(self.mines_amount as usize)
            .for_each(|cell| {
                cell.mine();
            });

        self.update_mines_around_values();

        Ok(())
    }

    /// The method increments the numerical values of the mined cells' adjacent cells, which represent the number of
    /// mines around an adjacent cell.
    fn update_mines_around_values(&mut self) {
        // Flatten the field for an easier interaction with it.
        let flattened_field = self.grid.iter_mut().flatten();
        // Get mutable borrowings for all the mined cells.
        let cells_with_mines = flattened_field.filter(|cell| cell.is_mined());

        // Get a flat vector of all the mined cells' adjacent cells' positions.
        let adjacent_cells_positions = cells_with_mines
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
    }

    /// Returns the field's height (the number of rows), width (the number of columns) and the two values multiplied,
    /// which is effectively the total number of cells.
    pub fn get_size(&self) -> (u8, u8, u16) {
        let rows_amount = self.grid.len() as u8;
        let columns_amount = self.grid.first().map(|row| row.len()).unwrap_or(0) as u8;
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        (rows_amount, columns_amount, cells_amount)
    }

    // TODO needs tests
    pub fn get_mines_amount(&self) -> u16 {
        self.mines_amount
    }

    /// Returns a read-only cell reference by its position or [`None`] if there's no cell at the given position.
    pub fn get_cell(&self, (row_index, column_index): (u8, u8)) -> Option<&Cell> {
        self.grid
            .get(row_index as usize)
            .and_then(|r| r.get(column_index as usize))
    }

    /// Returns a mutable cell reference by its position or [`None`] if there's no cell at the given position.
    fn get_cell_mut(&mut self, (row_index, column_index): (u8, u8)) -> Option<&mut Cell> {
        self.grid
            .get_mut(row_index as usize)
            .and_then(|r| r.get_mut(column_index as usize))
    }

    /// Opens a cell by its position.
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
    /// The method won't produce any effect if the target cell is closed or flagged or if its numerical value is not the
    /// same as the number of flags placed around it.
    pub fn open_surrounding_cells(&mut self, (row_index, column_index): (u8, u8)) {
        if let Some(target_cell) = self.get_cell((row_index, column_index)) {
            let adjacent_cells_indices = target_cell.get_adjacent_cells_positions();

            let flagged_adjacent_cells_amount = adjacent_cells_indices
                .iter()
                .filter_map(|(row_index, column_index)| self.get_cell((*row_index, *column_index)))
                .filter(|adjacent_cell| adjacent_cell.is_flagged())
                .collect::<Vec<&Cell>>()
                .len() as u8;

            if let Some(a) = target_cell.get_mines_around_amount() {
                if target_cell.is_open()
                    && target_cell.get_mines_around_amount().is_some()
                    && flagged_adjacent_cells_amount == a
                {
                    adjacent_cells_indices
                        .into_iter()
                        .for_each(|adjacent_cell_position| {
                            self.open_cell(adjacent_cell_position);
                        });
                };
            }
        }
    }

    /// Toggles flag for the cell (if any) with the given position.
    pub fn toggle_cell_flag(&mut self, (row_index, columns_index): (u8, u8)) {
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
            .collect::<Vec<&Cell>>()
            .len() as u16
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
        for (i, _) in self.grid[0].iter().enumerate() {
            write!(
                f,
                "{:^3}",
                if i == 0 {
                    "    0 ".to_string()
                } else {
                    i.to_string()
                }
            )?;
        }

        writeln!(f)?;

        for (i, row) in self.grid.iter().enumerate() {
            write!(f, "{:^3}", i)?;

            for cell in row {
                write!(f, "{} ", cell)?;
            }

            writeln!(f)?;
        }

        write!(f, "")
    }
}

#[cfg(test)]
mod test {
    use super::{Cell, Field, FieldError};

    #[test]
    fn create_field_instance_correct_params() {
        let field = Field::new(3, 3, 3);
        assert!(field.is_ok());

        assert_eq!(
            field.unwrap(),
            Field {
                grid: vec![
                    vec![Cell::new((0, 0)), Cell::new((0, 1)), Cell::new((0, 2)),],
                    vec![Cell::new((1, 0)), Cell::new((1, 1)), Cell::new((1, 2)),],
                    vec![Cell::new((2, 0)), Cell::new((2, 1)), Cell::new((2, 2)),],
                ],
                mines_amount: 3
            }
        )
    }

    #[test]
    fn create_field_fails_when_not_enough_cells() {
        let field = Field::new(1, 1, 1);
        assert!(field.is_err_and(|err| err == FieldError::NotEnoughCells));
    }

    #[test]
    fn create_field_fails_when_not_enough_mines() {
        let field = Field::new(3, 3, 0);
        assert!(field.is_err_and(|err| err == FieldError::InvalidMinesAmount(8)));
    }

    #[test]
    fn create_field_fails_when_too_many_mines() {
        let field = Field::new(3, 3, 9);
        assert!(field.is_err_and(|err| err == FieldError::InvalidMinesAmount(8)));
    }

    #[test]
    fn the_field_gets_correctly_populated_with_mines() {
        let mut field = Field::new(3, 3, 3).unwrap();
        let result = field.populate_with_mines(None);

        assert!(result.is_ok());
        assert_eq!(
            field.mines_amount,
            field
                .grid
                .iter()
                .flatten()
                .filter(|cell| cell.is_mined())
                .collect::<Vec<&Cell>>()
                .len() as u16
        );
    }

    #[test]
    fn populate_with_mines_correctly_excepts_a_cell() {
        for _ in 0..100 {
            let mut field = Field::new(3, 3, 3).unwrap();
            let result = field.populate_with_mines(Some((0, 0)));

            assert!(result.is_ok());
            assert!(!field.grid[0][0].is_mined())
        }
    }

    #[test]
    fn populate_with_mines_fails_on_invalid_excepted_cell_position() {
        let mut field = Field::new(3, 3, 3).unwrap();
        let result = field.populate_with_mines(Some((5, 5)));

        assert!(result.is_err_and(|err| err == FieldError::InvalidExceptedCellPosition((5, 5))));
    }

    #[test]
    fn populate_with_mines_fails_when_there_are_mines_already() {
        let mut field = Field::new(3, 3, 3).unwrap();
        field.populate_with_mines(None).unwrap();
        let result = field.populate_with_mines(None);

        assert!(result.is_err_and(|err| err == FieldError::MinesAlreadyExist));
    }

    fn create_stub_mined_field(enlarged: bool) -> Field {
        // "mine", "mine", "none"
        // "none", "none", "mine"
        // "none", "none", "none"
        // "none", "none", "none" <- only when enlarged
        let mut grid = vec![
            vec![
                {
                    let mut cell = Cell::new((0, 0));
                    cell.mine();
                    cell
                },
                {
                    let mut cell = Cell::new((0, 1));
                    cell.mine();
                    cell
                },
                Cell::new((0, 2)),
            ],
            vec![Cell::new((1, 0)), Cell::new((1, 1)), {
                let mut cell = Cell::new((1, 2));
                cell.mine();
                cell
            }],
            vec![Cell::new((2, 0)), Cell::new((2, 1)), Cell::new((2, 2))],
        ];

        if enlarged {
            // Add a row of empty cells.
            let empty_row = vec![Cell::new((3, 0)), Cell::new((3, 1)), Cell::new((3, 2))];
            grid.push(empty_row);
        }

        Field {
            grid,
            mines_amount: 3,
        }
    }

    #[test]
    fn mines_around_values_get_updated_correctly() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();

        let result = field
            .grid
            .iter()
            .flatten()
            .map(|cell| cell.get_mines_around_amount())
            .collect::<Vec<Option<u8>>>();

        assert_eq!(
            result,
            [
                None,
                None,
                Some(2),
                Some(2),
                Some(3),
                None,
                Some(0),
                Some(1),
                Some(1)
            ]
        );
    }

    #[test]
    fn get_size_correctly_calculates_dimensions() {
        let field = Field::new(3, 3, 3).unwrap();
        let size = field.get_size();

        assert_eq!(size, (3, 3, 9));
    }

    #[test]
    fn get_cell_correctly_finds_the_cell_by_its_position() {
        let field = Field::new(3, 3, 3).unwrap();
        let cell = field.get_cell((0, 0));

        assert!(cell.is_some());
        assert_eq!(cell.unwrap(), &field.grid[0][0])
    }

    #[test]
    fn get_cell_returns_none_for_non_existing_cells() {
        let field = Field::new(3, 3, 3).unwrap();
        let cell = field.get_cell((10, 10));

        assert!(cell.is_none());
    }

    #[test]
    fn get_cell_mut_correctly_finds_the_cell_by_its_position() {
        // let field = RefCell::new(Field::new(3, 3, 3).unwrap());
        // let mut b = field.borrow_mut();
        // let cell = b.get_cell_mut((0, 0));
        //
        // assert!(cell.is_some());
        // assert_eq!(cell.unwrap(), &mut (field.borrow_mut().grid[0][0]));
    }

    #[test]
    fn get_cell_mut_returns_none_for_non_existing_cells() {
        let mut field = Field::new(3, 3, 3).unwrap();
        let cell = field.get_cell_mut((10, 10));

        assert!(cell.is_none());
    }

    #[test]
    fn open_cell_opens_the_requested_cell() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();
        field.open_cell((0, 2));

        // First, make sure the target cell is opened.
        assert!(field.get_cell((0, 2)).unwrap().is_open());

        // Then get all the cells...
        let mut all_cells: Vec<_> = field.grid.iter_mut().flatten().collect();

        // ...And remove the target one. Make sure all the remaining cells are closed (no chain-opening in this case,
        // because the target cell has two mines around it).
        all_cells.remove(2);
        assert!(all_cells.iter().all(|cell| !cell.is_open()))
    }

    #[test]
    fn open_cell_chain_opens_empty_cells() {
        let mut field = create_stub_mined_field(true);
        field.update_mines_around_values();
        field.open_cell((2, 0));

        let open_cells_positions = [(2u8, 0u8), (3, 0), (3, 1), (3, 2)];
        let closed_cells_positions = [
            (0u8, 0u8),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 2),
            (2, 3),
        ];

        // A meta-assertion. Make sure we're not forgetting any cells.
        assert_eq!(
            field.get_size().2,
            (open_cells_positions.len() + closed_cells_positions.len()) as u16
        );

        // Make sure all the cells which need to be open are open.
        open_cells_positions
            .into_iter()
            .map(|pos| field.get_cell(pos))
            .all(|cell| cell.unwrap().is_open());

        // Make sure all the cells which need to be closed are closed.

        closed_cells_positions
            .into_iter()
            .map(|pos| field.get_cell(pos))
            .all(|cell| !cell.unwrap().is_open());
    }

    #[test]
    fn open_surrounding_cells_opens_correct_cells() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();
        field.get_cell_mut((0, 0)).unwrap().toggle_flag();
        field.get_cell_mut((0, 1)).unwrap().toggle_flag();
        field.get_cell_mut((1, 2)).unwrap().toggle_flag();
        field.open_cell((1, 1));
        field.open_surrounding_cells((1, 1));

        // The above is the winning strategy. All the non-flagged cells should be opened by now.
        assert!(field
            .grid
            .iter()
            .flatten()
            .filter(|cell| !cell.is_flagged())
            .all(|cell| cell.is_open()));
    }

    #[test]
    fn open_surrounding_cells_for_a_closed_cell_has_no_effect() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();
        field.get_cell_mut((0, 0)).unwrap().toggle_flag();
        field.get_cell_mut((0, 1)).unwrap().toggle_flag();
        field.get_cell_mut((1, 2)).unwrap().toggle_flag();
        // field.open_cell((1, 1)); <- don't open the target cell
        field.open_surrounding_cells((1, 1));

        // All the cells must remain closed.
        assert!(field.grid.iter().flatten().all(|cell| !cell.is_open()));
    }

    #[test]
    fn open_surrounding_cells_for_a_flagged_cell_has_no_effect() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();
        field.get_cell_mut((0, 0)).unwrap().toggle_flag();
        field.get_cell_mut((0, 1)).unwrap().toggle_flag();
        field.get_cell_mut((1, 2)).unwrap().toggle_flag();
        field.get_cell_mut((1, 1)).unwrap().toggle_flag(); // flag the target cell
        field.open_surrounding_cells((1, 1));

        // All the cells must remain closed.
        assert!(field.grid.iter().flatten().all(|cell| !cell.is_open()));
    }

    #[test]
    fn open_surrounding_cells_has_no_effect_on_incorrect_mines_around_amount() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();
        field.get_cell_mut((0, 0)).unwrap().toggle_flag();
        field.get_cell_mut((0, 1)).unwrap().toggle_flag();
        field.get_cell_mut((1, 2)).unwrap().toggle_flag();
        field.open_cell((1, 1));

        // So far so good, but add an excessive flag somewhere around
        field.get_cell_mut((2, 0)).unwrap().toggle_flag();

        field.open_surrounding_cells((1, 1));

        // All the cells (except for the target one) must remain closed.
        let mut all_cells: Vec<_> = field.grid.iter().flatten().collect();
        all_cells.remove(4);
        assert!(all_cells.into_iter().all(|cell| !cell.is_open()));
    }

    #[test]
    fn toggle_cell_flag_correctly_toggles_the_flag() {
        let mut field = Field::new(3, 3, 3).unwrap();
        assert!(!field.get_cell((1, 1)).unwrap().is_flagged());

        field.toggle_cell_flag((1, 1));
        assert!(field.get_cell((1, 1)).unwrap().is_flagged());

        field.toggle_cell_flag((1, 1));
        assert!(!field.get_cell((1, 1)).unwrap().is_flagged());
    }

    #[test]
    fn toggle_cell_flag_has_no_effect_if_the_cell_is_not_found() {
        let mut field = Field::new(3, 3, 3).unwrap();

        field.toggle_cell_flag((5, 5));
        assert!(field.grid.iter().flatten().all(|cell| !cell.is_flagged()));
    }

    #[test]
    fn get_flagged_cells_amount_returns_the_correct_amount_of_flagged_cells() {
        let mut field = Field::new(3, 3, 3).unwrap();

        field.toggle_cell_flag((0, 0));
        field.toggle_cell_flag((0, 1));
        field.toggle_cell_flag((0, 2));
        field.toggle_cell_flag((1, 0));

        assert_eq!(field.get_flagged_cells_amount(), 4);
    }

    #[test]
    fn open_mines_amount_is_determined_correctly() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();

        assert!(!field.check_open_mines_exist());

        // Open a mined cell.
        field.open_cell((0, 1));

        assert!(field.check_open_mines_exist());
    }

    #[test]
    fn the_win_condition_is_checked_correctly() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();

        assert!(!field.check_all_non_mines_open());

        field.get_cell_mut((0, 0)).unwrap().toggle_flag();
        field.get_cell_mut((0, 1)).unwrap().toggle_flag();
        field.get_cell_mut((1, 2)).unwrap().toggle_flag();
        field.open_cell((1, 1));
        field.open_surrounding_cells((1, 1));

        // The above actions lead to winning the game. So by now the game is considered to be won, which is exactly what
        // the method checks.
        assert!(field.check_all_non_mines_open());
    }

    #[test]
    fn missed_mines_get_opened_correctly() {
        let mut field = create_stub_mined_field(false);
        field.update_mines_around_values();

        // There are 3 mines on the field. Flag only one of them, then call the `open_missed_mines` method and check
        // that only the other two are open.
        field.toggle_cell_flag((0, 1));
        field.open_missed_mines();

        assert!(field.get_cell((0, 0)).unwrap().is_open());
        assert!(field.get_cell((1, 2)).unwrap().is_open());

        // The total number of open cells by now should be two.
        assert_eq!(
            field
                .grid
                .iter()
                .flatten()
                .filter(|cell| cell.is_open())
                .collect::<Vec<_>>()
                .len(),
            2
        );
    }
}
