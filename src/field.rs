pub mod cell;

use cell::Cell;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum FieldError {
    NotEnoughCells,
    InvalidCellPosition((u8, u8)),
    InvalidMinesAmount,
    MinesAlreadyExist,
}

#[derive(Default, PartialEq, Eq)]
pub struct Field(Vec<Vec<Cell>>);

impl Field {
    pub fn new(rows_amount: u8, columns_amount: u8) -> Result<Self, FieldError> {
        if (rows_amount as u16 * columns_amount as u16) < 2_u16 {
            // return an error if there are less than 2 cells total
            Err(FieldError::NotEnoughCells)
        } else {
            // otherwise, return a new `Field` instance where all the cells are empty
            Ok(Field(
                (0..rows_amount)
                    .map(|row_index| {
                        (0..columns_amount)
                            // save each cell's position in themselves
                            .map(|column_index| Cell::new((row_index, column_index)))
                            .collect::<Vec<Cell>>()
                    })
                    .collect::<Vec<Vec<Cell>>>(),
            ))
        }
    }

    pub fn get_size(&self) -> (u8, u8, u16) {
        // get the field's dimensions
        let rows_amount = self.0.len() as u8;
        let columns_amount = self.0.first().map(|row| row.len()).unwrap_or(0) as u8;
        let cells_amount = rows_amount as u16 * columns_amount as u16;

        // and return them
        (rows_amount, columns_amount, cells_amount)
    }

    pub fn insert_mines(
        &mut self,
        mines_amount: u16,
        excepted_cell_position: Option<(u8, u8)>, // (row, column)
    ) -> Result<(), FieldError> {
        // get the number of rows, the width of a single row, and the total cells number of the field
        let (rows_amount, columns_amount, cells_amount) = self.get_size();

        // return an error if the provided excepted cell's position goes beyond the field's dimensions
        if let Some((row_index, column_index)) = excepted_cell_position {
            if (row_index > rows_amount - 1) || (column_index > columns_amount - 1) {
                return Err(FieldError::InvalidCellPosition((row_index, column_index)));
            }
        }

        // return an error if the required amount of mines is less than 1 or is more than the total amount of cells
        // minus 1 (there should always at least one mine and at least one cell without a mine)
        if mines_amount < 1 || mines_amount > (cells_amount - 1) {
            return Err(FieldError::InvalidMinesAmount);
        }

        // flatten the field for an easier interaction with it
        let mut flattened_field = self.0.iter_mut().flatten().collect::<Vec<&mut Cell>>();

        // return an error if there are mines already: can't populate with mines a field that's already been populated
        if flattened_field.iter().any(|cell| cell.is_mine()) {
            return Err(FieldError::MinesAlreadyExist);
        }

        // remove the reference to the excepted cell (if any) to avoid marking it as mined
        if let Some((row_index, column_index)) = excepted_cell_position {
            // `row_index * columns_amount + column_index` is a formula of getting a 1D flattened-vector's index based
            // on the original 2D vector's coordinates
            flattened_field.remove((row_index * columns_amount + column_index) as usize);
        }

        // shuffle the mutable borrowings to randomly distribute the mines
        let mut rng = thread_rng();
        flattened_field.shuffle(&mut rng);

        // make the first `number_of_mines` cells mines and store the cells with the mines in a vector
        let cells_with_mines = flattened_field
            .iter_mut()
            .take(mines_amount as usize)
            .map(|cell| cell.mine())
            .collect::<Vec<Cell>>();

        // distribute the numbers that show the number of mines in the cells adjacent to those with mines
        self.insert_numbers(cells_with_mines);

        Ok(())
    }

    fn insert_numbers(&mut self, cells_with_mines: Vec<Cell>) {
        // get the width and the height of the field
        let (height, width, _) = self.get_size();

        // go through all the cells with mines and get their adjacent cells' indices
        let all_cells_indices_to_increment = cells_with_mines
            .iter()
            .flat_map(|cell| cell.get_adjacent_cells_indices((height, width)))
            .collect::<Vec<(u8, u8)>>();

        // for each cell-to-increment index (position), get the cell itself and increment its number
        all_cells_indices_to_increment
            .iter()
            .for_each(|(row_index, column_index)| {
                if let Some(cell) = self
                    .0
                    .get_mut(*row_index as usize)
                    .and_then(|row| row.get_mut(*column_index as usize))
                {
                    cell.increment_adjacent_mines_amount();
                }
            });
    }

    // TODO tests!
    pub fn get_cell(&self, (row_index, column_index): (u8, u8)) -> Option<&Cell> {
        self.0
            .get(row_index as usize)
            .and_then(|r| r.get(column_index as usize))
    }

    pub fn open_cell(&mut self, (row_index, column_index): (u8, u8)) {
        // get the width and the height of the field
        let (height, width, _) = self.get_size();

        if let Some(cell) = self
            .0
            .get_mut(row_index as usize)
            .and_then(|row| row.get_mut(column_index as usize))
        {
            let next_cells_to_open_positions = cell.open((height, width));

            next_cells_to_open_positions
                .iter()
                .for_each(|cell_to_open_position| {
                    self.open_cell(*cell_to_open_position);
                });
        };
    }

    pub fn open_surrounding_cells(&mut self, (row_index, column_index): (u8, u8)) {
        // get the width and the height of the field
        let (height, width, _) = self.get_size();

        if let Some(target_cell) = self
            .0
            .get(row_index as usize)
            .and_then(|row| row.get(column_index as usize))
        {
            let adjacent_cells_indices = target_cell.get_adjacent_cells_indices((height, width));

            let flagged_adjacent_cells_amount = adjacent_cells_indices
                .iter()
                .filter_map(|(row_index, column_index)| {
                    self.0
                        .get(*row_index as usize)
                        .and_then(|row| row.get(*column_index as usize))
                })
                .filter(|adjacent_cell| adjacent_cell.is_flagged())
                .collect::<Vec<&Cell>>()
                .len() as u8;

            if target_cell.is_open()
                && target_cell.get_adjacent_mines_amount().is_some()
                && flagged_adjacent_cells_amount == target_cell.get_adjacent_mines_amount().unwrap()
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
            .0
            .get_mut(row_index as usize)
            .and_then(|row| row.get_mut(columns_index as usize))
        {
            cell.toggle_flag()
        }
    }

    pub fn get_flagged_cells_amount(&self) -> u16 {
        self.0
            .iter()
            .flatten()
            .filter(|cell| cell.is_flagged())
            .map(|_| 1)
            .sum()
    }

    pub fn check_open_mines_exist(&self) -> bool {
        self.0
            .iter()
            .flatten()
            .any(|cell| cell.is_open() && cell.is_mine())
    }

    pub fn check_all_non_mines_open(&self) -> bool {
        self.0
            .iter()
            .flatten()
            .filter(|cell| !cell.is_mine())
            .all(|cell| cell.is_open())
    }

    // TODO tests!
    /// The method should be called when the game is lost to reveal the real positions of mines.
    pub fn open_missed_mines(&mut self) {
        let (rows_amount, columns_amount, _) = self.get_size();

        self.0
            .iter_mut()
            .flatten()
            .filter(|cell| cell.is_mine() && !cell.is_flagged())
            .for_each(|cell| {
                cell.open((rows_amount, columns_amount));
            });
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.0.iter() {
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
        self.0[0].iter().enumerate().for_each(|(i, _)| {
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

        for (i, row) in self.0.iter().enumerate() {
            write!(f, "{:^3}", i).unwrap();

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
    use super::cell::Cell;
    use super::{Field, FieldError};

    #[test]
    fn new_creates_a_field_with_specified_dimensions() {
        let asymmetrical_field = Field::new(2, 3);

        assert_eq!(
            asymmetrical_field,
            Ok(Field(vec![
                vec![Cell::new((0, 0)), Cell::new((0, 1)), Cell::new((0, 2))],
                vec![Cell::new((1, 0)), Cell::new((1, 1)), Cell::new((1, 2))],
            ]))
        );
    }

    #[test]
    fn new_wont_create_a_field_with_the_size_less_than_two() {
        let empty_field = Field::new(0, 0);
        assert_eq!(empty_field, Err(FieldError::NotEnoughCells));

        let no_rows_field = Field::new(1, 0);
        assert_eq!(no_rows_field, Err(FieldError::NotEnoughCells));

        let no_columns_field = Field::new(0, 1);
        assert_eq!(no_columns_field, Err(FieldError::NotEnoughCells));

        let one_cell_field = Field::new(1, 1);
        assert_eq!(one_cell_field, Err(FieldError::NotEnoughCells));

        let two_cells_field = Field::new(1, 2);
        assert!(two_cells_field.is_ok());
    }

    #[test]
    fn get_size_correctly_calculates_field_dimensions() {
        let regular_field = Field::new(2, 3).unwrap();
        assert_eq!(regular_field.get_size(), (2, 3, 2 * 3));

        let another_field = Field::new(81, 241).unwrap();
        assert_eq!(another_field.get_size(), (81, 241, 81 * 241));
    }

    #[test]
    fn insert_mines_inserts_a_correct_amount_of_mines() {
        let desired_amount_of_mines = 9;

        let mut field = Field::new(9, 9).unwrap();
        field.insert_mines(desired_amount_of_mines, None).unwrap();

        let real_amount_of_mines = field
            .0
            .iter()
            .flatten()
            .map(|cell| if let true = cell.is_mine() { 1 } else { 0 })
            .sum();

        assert_eq!(desired_amount_of_mines, real_amount_of_mines);
    }

    #[test]
    fn insert_mines_respects_the_excepted_cell() {
        for _ in 0..1000 {
            let mut field = Field::new(9, 9).unwrap();
            field.insert_mines(80, Some((0, 0))).unwrap();

            assert!(!field.0[0][0].is_mine());
        }
    }

    #[test]
    fn insert_mines_errors_when_excepted_index_is_out_of_bounds() {
        let mut field = Field::new(9, 9).unwrap();
        let result = field.insert_mines(9, Some((8, 9)));
        assert_eq!(result, Err(FieldError::InvalidCellPosition((8, 9))));

        let mut field = Field::new(9, 9).unwrap();
        let result = field.insert_mines(9, Some((9, 8)));
        assert_eq!(result, Err(FieldError::InvalidCellPosition((9, 8))));

        let mut field = Field::new(9, 9).unwrap();
        let result = field.insert_mines(9, Some((9, 9)));
        assert_eq!(result, Err(FieldError::InvalidCellPosition((9, 9))));
    }

    #[test]
    fn insert_mines_errors_when_less_than_one_mine_is_requested() {
        let mut field = Field::new(9, 9).unwrap();
        let result = field.insert_mines(0, None);
        assert_eq!(result, Err(FieldError::InvalidMinesAmount));
    }

    #[test]
    fn insert_mines_errors_when_too_many_mines_are_requested() {
        let mut field = Field::new(9, 9).unwrap();
        let result = field.insert_mines(81, None);
        assert_eq!(result, Err(FieldError::InvalidMinesAmount));
    }

    #[test]
    fn insert_mines_errors_on_double_invocation() {
        let mut field = Field::new(9, 9).unwrap();
        field.insert_mines(9, Some((0, 0))).unwrap();

        let result = field.insert_mines(9, None);
        assert_eq!(result, Err(FieldError::MinesAlreadyExist));
    }

    #[test]
    fn insert_numbers_correctly_inserts_numbers() {
        fn b(position: (u8, u8)) -> Cell {
            // build a mined cell (a bomb)
            let mut cell = Cell::new(position);
            cell.mine();
            cell
        }

        fn n(position: (u8, u8), number: u8) -> Cell {
            // builds a cell with a number
            let mut cell = Cell::new(position);
            (0..number).for_each(|_| cell.increment_adjacent_mines_amount());
            cell
        }

        let mut field_without_numbers = Field(vec![
            vec![b((0, 0)), n((0, 1), 0), n((0, 2), 0)],
            vec![n((1, 0), 0), n((1, 1), 0), b((1, 2))],
            vec![n((2, 0), 0), n((2, 1), 0), n((2, 2), 0)],
        ]);

        field_without_numbers.insert_numbers(vec![
            field_without_numbers.0[0][0],
            field_without_numbers.0[1][2],
        ]);

        let reference = Field(vec![
            vec![b((0, 0)), n((0, 1), 2), n((0, 2), 1)],
            vec![n((1, 0), 1), n((1, 1), 2), b((1, 2))],
            vec![n((2, 0), 0), n((2, 1), 1), n((2, 2), 1)],
        ]);

        assert_eq!(field_without_numbers, reference);
    }

    fn helper_assert_for_cells(
        field: Field,
        cells_positions: Vec<(u8, u8)>,
        should_be_open: bool,
    ) -> Field {
        cells_positions
            .iter()
            .for_each(|(row_index, column_index)| {
                assert_eq!(
                    field.0[*row_index as usize][*column_index as usize].is_open(),
                    should_be_open
                );
            });

        field
    }

    #[test]
    fn open_cell_correctly_opens_cell_when_the_target_cell_is_empty() {
        // Such a field, when opening the (1, 1) cell
        /*
        ⬜  ⬜  ⬜  1️ 💣
        ⬜  ⬜  ⬜  2️ 2️
        ⬜  ⬜  ⬜  1️ 💣
        ⬜  1️  1️  2️ 1️
        ⬜  1️  💣  1️ ⬜
         */

        // must produce the following result
        /*
        ⬜  ⬜  ⬜ 1️  ⬛
        ⬜  ⬜  ⬜ 2️  ⬛
        ⬜  ⬜  ⬜ 1️  ⬛
        ⬜  1️ 1️  2️  ⬛
        ⬜  1️ ⬛  ⬛  ⬛
         */

        let mut field = Field::new(5, 5).unwrap();
        field.0[0][4].mine();
        field.0[2][4].mine();
        field.0[4][2].mine();
        field.0[0][3].increment_adjacent_mines_amount(); // 1
        field.0[1][3].increment_adjacent_mines_amount(); // 1...
        field.0[1][3].increment_adjacent_mines_amount(); // 2
        field.0[2][3].increment_adjacent_mines_amount(); // 1
        field.0[3][1].increment_adjacent_mines_amount(); // 1
        field.0[3][2].increment_adjacent_mines_amount(); // 1
        field.0[3][3].increment_adjacent_mines_amount(); // 1...
        field.0[3][3].increment_adjacent_mines_amount(); // 2
        field.0[4][1].increment_adjacent_mines_amount(); // 1

        field.open_cell((1, 1));

        let cells_that_should_be_opened_positions = [
            (0u8, 0u8),
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 0),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
            (3, 3),
            (4, 0),
            (4, 1),
        ];

        field =
            helper_assert_for_cells(field, cells_that_should_be_opened_positions.to_vec(), true);

        let cells_that_should_be_closed_positions =
            [(0, 4), (1, 4), (2, 4), (3, 4), (4, 2), (4, 3), (4, 4)];

        assert_eq!(
            cells_that_should_be_opened_positions.len()
                + cells_that_should_be_closed_positions.len(),
            field.get_size().2 as usize
        );

        helper_assert_for_cells(field, cells_that_should_be_closed_positions.to_vec(), false);
    }

    #[test]
    fn open_cell_correctly_opens_cell_when_the_target_cell_contains_a_number() {
        // Such a field, when opening the (1, 1) cell
        /*
        ⬜  1️  💣  2️  💣
        ⬜  1️  1️  2️  1️
        ⬜  ⬜  ⬜  ⬜  ⬜
        1️  1️  1️  ⬜  ⬜
        1️  💣  1️  ⬜  ⬜
         */

        // must produce the following result
        /*
        ⬛  ⬛  ⬛  ⬛  ⬛
        ⬛  1️  ⬛  ⬛  ⬛
        ⬛  ⬛  ⬛  ⬛  ⬛
        ⬛  ⬛  ⬛  ⬛  ⬛
        ⬛  ⬛  ⬛  ⬛  ⬛
         */

        let mut field = Field::new(5, 5).unwrap();
        field.0[0][2].mine();
        field.0[0][4].mine();
        field.0[4][1].mine();
        // we only care about the scenario when the target cell is a number
        field.0[1][1].increment_adjacent_mines_amount(); // 1

        field.open_cell((1, 1));

        let cells_that_should_be_opened_positions = [(1, 1)];

        field =
            helper_assert_for_cells(field, cells_that_should_be_opened_positions.to_vec(), true);

        let cells_that_should_be_closed_positions = [
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (1, 0),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 0),
            (2, 1),
            (2, 2),
            (2, 3),
            (2, 4),
            (3, 0),
            (3, 1),
            (3, 2),
            (3, 3),
            (3, 4),
            (4, 0),
            (4, 1),
            (4, 2),
            (4, 3),
            (4, 4),
        ];

        assert_eq!(
            cells_that_should_be_opened_positions.len()
                + cells_that_should_be_closed_positions.len(),
            field.get_size().2 as usize
        );

        helper_assert_for_cells(field, cells_that_should_be_closed_positions.to_vec(), false);
    }

    #[test]
    fn open_surrounding_cells_correctly_opens_surrounding_cells() {
        // let assume we have a 5*5 field where at the (0, 0) position is the only mine. Currently we're as the
        // state after the the first move, where the user has just opened the cell (1, 0). Next, they flag the (0, 0)
        // cell and open the surrounding cells for (1, 0). This should effectively open all the cells.

        // create the filed with one mine a properly distributed numbers
        let mut field = Field::new(5, 5).unwrap();
        field.0[0][0].mine();
        field.insert_numbers(vec![field.0[0][0]]);

        // open the (1, 0) cell
        field.open_cell((1, 0));

        // flag the mine
        field.flag_cell((0, 0));

        // open the surrounding cells for the opened (1, 0)
        field.open_surrounding_cells((1, 0));

        // all the cells except for the one with the mine should be opened now
        let all_cells_opened = field.0.iter().flatten().skip(1).all(|cell| cell.is_open());

        assert!(all_cells_opened);
    }

    #[test]
    fn open_surrounding_cells_does_not_produce_no_effect_when_the_target_cell_is_closed() {
        // the same field from the above, but this time we won't open the (1, 0) cells and will then try to open its
        // surrounding cells. This should fail (by not producing any effect)

        // create the filed with one mine a properly distributed numbers
        let mut field = Field::new(5, 5).unwrap();
        field.0[0][0].mine();
        field.insert_numbers(vec![field.0[0][0]]);

        // don't open the (1, 0) cell
        // field.open_cell((1, 0));

        // flag the mine
        field.flag_cell((0, 0));

        // open the surrounding cells for the close (1, 0)
        field.open_surrounding_cells((1, 0));

        // all the cells should remain closed (the first one - the one with the mine - should remain `Flagged`)
        let all_cells_closed = field.0.iter().flatten().skip(1).all(|cell| !cell.is_open());

        assert!(all_cells_closed);
    }

    #[test]
    fn open_surrounding_cells_does_not_produce_no_effect_when_the_target_cell_is_flagged() {
        // the same field from the above, but this time we flag the mined (0, 0) cell and trying to open the surrounding
        // cells for it. This attempt should fail by not producing any effect

        // create the filed with one mine a properly distributed numbers
        let mut field = Field::new(5, 5).unwrap();
        field.0[0][0].mine();
        field.insert_numbers(vec![field.0[0][0]]);

        // don't open the (1, 0) cell
        // field.open_cell((1, 0));

        // flag the mine
        field.flag_cell((0, 0));

        // open the surrounding cells for it
        field.open_surrounding_cells((0, 0));

        // all the cells should remain closed (the first one - the one with the mine - should remain `Flagged`)
        let all_cells_closed = field.0.iter().flatten().skip(1).all(|cell| !cell.is_open());

        assert!(all_cells_closed);
    }

    #[test]
    fn open_surrounding_cells_does_not_produce_no_effect_when_the_flags_number_mismatch_the_mines_around_amount(
    ) {
        fn calc_num_of_closed_cells(field: &Field) -> usize {
            field
                .0
                .iter()
                .flatten()
                .filter(|cell| !cell.is_open() && !cell.is_flagged())
                .collect::<Vec<&Cell>>()
                .len()
        }

        // the same field, but this time opening the surrounding cells should fail (not produce eny effect) because
        // 1. For the first part of the test there won't be any flags at all
        // 2. For the second part of the test there will be two flags instead of one

        // create the filed with one mine a properly distributed numbers
        let mut field = Field::new(5, 5).unwrap();
        field.0[0][0].mine();
        field.insert_numbers(vec![field.0[0][0]]);

        // open the (1, 0) cell
        field.open_cell((1, 0));

        // don't flag the mine
        // field.flag_cell((0, 0));

        // open the surrounding cells for the opened (1, 0)
        field.open_surrounding_cells((1, 0));

        // all the cells except for the one with that was opened should be closed
        let number_of_closed_cells = calc_num_of_closed_cells(&field);

        assert_eq!(number_of_closed_cells, 24);

        // SECOND PART //

        // flag two cells
        field.flag_cell((0, 0));
        field.flag_cell((0, 1));

        // open the surrounding cells for the opened (1, 0)
        field.open_surrounding_cells((1, 0));

        // all the cells except for the one with that was opened should be closed
        let number_of_closed_cells = calc_num_of_closed_cells(&field);

        // should be 22 because one cell has been opened and 2 were flagged: 25 - 1 - 2 is 22
        assert_eq!(number_of_closed_cells, 22);
    }

    #[test]
    fn flag_cell_correctly_toggles_the_flag() {
        let mut field = Field::new(20, 20).unwrap();

        field.flag_cell((0, 0));
        assert!(field.0[0][0].is_flagged());

        field.flag_cell((0, 0));
        assert!(!field.0[0][0].is_open() && !field.0[0][0].is_flagged());
    }

    #[test]
    fn check_open_mines_exist_correctly_indicates_the_presence_of_mines() {
        let mut field = Field::new(20, 20).unwrap();
        let mut mine = Cell::new((0, 0));
        mine.mine();
        field.0[0][0] = mine;
        field.open_cell((0, 0));

        assert!(field.check_open_mines_exist());
    }

    #[test]
    fn check_open_mines_exist_does_not_false_trigger() {
        let mut field = Field::new(20, 20).unwrap();
        let mut mine = Cell::new((0, 0));
        mine.mine();
        field.0[0][0] = mine;
        // field.open_cell((0, 0)); // don't open the mined cell

        assert!(!field.check_open_mines_exist());
    }

    #[test]
    fn check_all_non_mines_open_is_true_when_all_non_mines_are_open() {
        let mut field = Field::new(20, 20).unwrap();
        let mut mine = Cell::new((0, 0));
        mine.mine();
        field.0[0][0] = mine;
        field.open_cell((19, 19)); // this opens all the non-mined cells consecutively

        assert!(field.check_all_non_mines_open());
    }

    #[test]
    fn check_all_non_mines_open_does_not_false_trigger() {
        let mut field = Field::new(20, 20).unwrap();
        let mut mine = Cell::new((0, 0));
        mine.mine();
        field.0[0][0] = mine;
        // field.open_cell((19, 19)); // don't open the empty cell, so everything remains closed

        assert!(!field.check_all_non_mines_open());
    }
}
