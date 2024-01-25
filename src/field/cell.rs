use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CellVariant {
    Empty(u8),
    Mine,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CellState {
    Closed,
    Open,
    Flagged,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cell {
    position: (u8, u8), // `(row, column)`
    variant: CellVariant,
    state: CellState,
}

impl Cell {
    pub fn new(position: (u8, u8)) -> Self {
        // create and return a new closed empty cell with the provided position
        Cell {
            position,
            variant: CellVariant::Empty(0),
            state: CellState::Closed,
        }
    }

    pub fn is_mine(&self) -> bool {
        self.variant == CellVariant::Mine
    }

    pub fn mine(&mut self) -> Self {
        // modify the cell in-place by changing its variant
        *self = Cell {
            position: self.position,
            variant: CellVariant::Mine,
            state: self.state,
        };

        // and return the modified cell
        *self
    }

    pub fn get_adjacent_mines_amount(&self) -> Option<u8> {
        // return the amount of mines in the cell's adjacent cells or `None` if the cell is a mine itself
        if let CellVariant::Empty(adjacent_mines_amount) = self.variant {
            Some(adjacent_mines_amount)
        } else {
            None
        }
    }

    pub fn increment_adjacent_mines_amount(&mut self) {
        // only increment the adjacent mined cells amount if the cell itself isn't mined
        if let CellVariant::Empty(adjacent_mines_amount) = self.variant {
            self.variant = CellVariant::Empty(adjacent_mines_amount + 1);
        }
    }

    pub fn is_open(&self) -> bool {
        self.state == CellState::Open
    }

    pub fn open(&mut self, field_size: (u8, u8)) -> Vec<(u8, u8)> {
        if let CellState::Closed = self.state {
            // only open the cell if it's closed
            self.state = CellState::Open;
        } else {
            // if not, don't do anything and immediately return an empty vector
            return vec![];
        }

        // we only go here if the cell was opened above
        if let CellVariant::Empty(0) = self.variant {
            // return the indices of the cell's adjacent cells to recursively open them as well in the caller
            self.get_adjacent_cells_indices(field_size)
        } else {
            // return an empty vector (thereby indicating that no adjacent cells should be recursively opened by the
            // caller) if the cell borders a mine
            vec![]
        }
    }

    pub fn is_flagged(&self) -> bool {
        self.state == CellState::Flagged
    }

    pub fn toggle_flag(&mut self) {
        // a closed cell becomes flagged, a flagged cell becomes closed. No changes for the cells with numbers
        self.state = match self.state {
            CellState::Closed => CellState::Flagged,
            CellState::Flagged => CellState::Closed,
            _ => self.state,
        };
    }

    pub fn get_adjacent_cells_indices(
        &self,
        (field_height, field_width): (u8, u8),
    ) -> Vec<(u8, u8)> {
        // transform the cell's coordinates into `i16` to be able to subtract and add without overflow
        let (row_index, column_index) = (self.position.0 as i16, self.position.1 as i16);

        // create a 2D vector of all the cells' indices surrounding the current one
        vec![
            vec![
                (row_index - 1, column_index - 1),
                (row_index, column_index - 1),
                (row_index + 1, column_index - 1),
            ],
            vec![
                (row_index - 1, column_index),
                /*         current         */
                (row_index + 1, column_index),
            ],
            vec![
                (row_index - 1, column_index + 1),
                (row_index, column_index + 1),
                (row_index + 1, column_index + 1),
            ],
        ]
        .into_iter()
        .flatten() // flatten the 2D vector for an easier filtration
        .filter(|(row_index, column_index)| {
            // filter out all the cells' indices that go beyond the field's dimensions:
            //  1. row's and column's indices can not be less than 0 (the first row/column)
            //  2. row's and column's indices can not be more than the maximum (last) row/column indices (i.e. the
            //     field's height/width - 1)
            *row_index >= 0
                && *row_index <= (field_height as i16 - 1)
                && *column_index >= 0
                && *column_index <= (field_width as i16 - 1)
        })
        .map(|(row_index, column_index)| (row_index as u8, column_index as u8))
        .collect::<Vec<(u8, u8)>>()
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let CellState::Flagged = self.state {
            return write!(f, "🚩");
        }

        match self.variant {
            CellVariant::Empty(n) => match n {
                0 => write!(f, "⬜ "),
                1 => write!(f, "1️"),
                2 => write!(f, "2️"),
                3 => write!(f, "3️"),
                4 => write!(f, "4️"),
                5 => write!(f, "5️"),
                6 => write!(f, "6️"),
                7 => write!(f, "7️"),
                8 => write!(f, "8️"),
                9 => write!(f, "9️"),
                _ => write!(f, "?"),
            },
            CellVariant::Mine => write!(f, "💣"),
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.state {
            CellState::Closed => write!(f, "⬛ "),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Cell, CellState, CellVariant};

    #[test]
    fn new_creates_a_closed_empty_cell_with_the_provided_position() {
        let new_cell = Cell::new((5, 5));

        assert_eq!(
            new_cell,
            Cell {
                position: (5, 5),
                variant: CellVariant::Empty(0),
                state: CellState::Closed
            }
        );
    }

    #[test]
    fn is_mine_correctly_determines_the_presence_of_a_mine() {
        let mut mined_cell = Cell::new((5, 5));
        assert!(!mined_cell.is_mine());

        mined_cell.mine();
        assert!(mined_cell.is_mine());
    }

    #[test]
    fn mine_correctly_makes_the_cell_a_mine() {
        let mut mined_cell = Cell::new((5, 5));
        assert!(!mined_cell.is_mine());

        mined_cell.mine();
        assert!(mined_cell.is_mine());
    }

    #[test]
    fn mine_does_not_produce_no_side_effects() {
        let mut cell = Cell::new((5, 5));
        cell.mine();

        assert_eq!(
            cell,
            Cell {
                position: (5, 5),
                variant: CellVariant::Mine,
                state: CellState::Closed,
            }
        );
    }

    #[test]
    fn get_adjacent_mines_amount_returns_the_correct_amount_of_adjacent_mines() {
        let mut adjacent_cell = Cell::new((5, 5));

        adjacent_cell.increment_adjacent_mines_amount(); // 1
        adjacent_cell.increment_adjacent_mines_amount(); // 2
        adjacent_cell.increment_adjacent_mines_amount(); // 3

        assert_eq!(adjacent_cell.get_adjacent_mines_amount().unwrap(), 3_u8);
    }

    #[test]
    fn get_adjacent_mines_amount_returns_none_when_the_cell_is_a_mine_itself() {
        let mut mined_cell = Cell::new((5, 5));
        mined_cell.mine();

        assert!(mined_cell.get_adjacent_mines_amount().is_none());
    }

    #[test]
    fn increment_adjacent_mines_amount_properly_increments_the_value() {
        let mut cell = Cell::new((5, 5));
        cell.increment_adjacent_mines_amount();

        assert_eq!(cell.get_adjacent_mines_amount().unwrap(), 1_u8);
    }

    #[test]
    fn increment_adjacent_mines_amount_does_not_do_anything_if_the_cell_is_a_mine() {
        let mut cell = Cell::new((5, 5));
        cell.mine();

        cell.increment_adjacent_mines_amount();
        assert!(cell.is_mine());
    }

    #[test]
    fn is_open_correctly_determines_whether_the_cell_is_open() {
        let mut cell = Cell::new((5, 5));
        assert!(!cell.is_open());

        cell.open((10, 10));
        assert!(cell.is_open());
    }

    #[test]
    fn open_correctly_opens_a_closed_cell() {
        let mut cell = Cell::new((5, 5));

        cell.open((10, 10));
        assert!(cell.is_open());
    }

    #[test]
    fn open_returns_an_empty_vector_when_opening_an_already_opened_cell() {
        let mut cell = Cell::new((5, 5));
        cell.open((10, 10));

        let res = cell.open((10, 10));

        assert!(cell.is_open());
        assert!(res.is_empty());
    }

    #[test]
    fn open_returns_adjacent_cells_if_the_opened_cell_is_empty() {
        let mut cell = Cell::new((10, 10));

        let res = cell.open((20, 20));

        assert!(cell.is_open());
        assert_eq!(
            res,
            vec![
                (9, 9),
                (10, 9),
                (11, 9),
                (9, 10),
                (11, 10),
                (9, 11),
                (10, 11),
                (11, 11)
            ]
        );
    }

    #[test]
    fn open_returns_an_empty_vector_if_the_cell_borders_a_mine() {
        let mut cell = Cell::new((10, 10));
        cell.increment_adjacent_mines_amount();

        let res = cell.open((20, 20));

        assert!(cell.is_open());
        assert!(res.is_empty());
    }

    #[test]
    fn open_wont_open_the_cell_if_it_is_flagged() {
        let mut cell = Cell::new((10, 10));
        cell.toggle_flag();

        cell.open((20, 20));

        assert!(!cell.is_open() && cell.is_flagged());
    }

    #[test]
    fn is_flagged_correctly_determines_whether_the_cell_is_flagged() {
        let mut cell = Cell::new((5, 5));
        assert!(!cell.is_flagged());

        cell.toggle_flag();
        assert!(cell.is_flagged());
    }

    #[test]
    fn toggle_flag_correctly_toggles_the_flag() {
        let mut cell = Cell::new((5, 5));
        assert!(!cell.is_flagged());

        cell.toggle_flag();
        assert!(cell.is_flagged());

        cell.toggle_flag();
        assert!(!cell.is_flagged());
    }

    #[test]
    fn toggle_flag_does_not_do_anything_for_open_cells() {
        let mut cell = Cell::new((5, 5));
        cell.open((10, 10));
        assert!(cell.is_open());

        cell.toggle_flag();
        assert!(cell.is_open());
    }

    #[test]
    fn get_adjacent_cells_indices_works_correctly_for_inner_cells() {
        let cell = Cell::new((10, 10));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(
            adjacent_cells_indices,
            vec![
                (9, 9),
                (10, 9),
                (11, 9),
                (9, 10),
                (11, 10),
                (9, 11),
                (10, 11),
                (11, 11)
            ]
        )
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_first_row() {
        let cell = Cell::new((0, 10));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(
            adjacent_cells_indices,
            vec![(0, 9), (1, 9), (1, 10), (0, 11), (1, 11)]
        )
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_first_column() {
        let cell = Cell::new((10, 0));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(
            adjacent_cells_indices,
            vec![(9, 0), (11, 0), (9, 1), (10, 1), (11, 1)]
        )
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_last_row() {
        let cell = Cell::new((19, 10));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(
            adjacent_cells_indices,
            vec![(18, 9), (19, 9), (18, 10), (18, 11), (19, 11)]
        )
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_last_column() {
        let cell = Cell::new((10, 19));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(
            adjacent_cells_indices,
            vec![(9, 18), (10, 18), (11, 18), (9, 19), (11, 19)]
        )
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_zero_zero() {
        let cell = Cell::new((0, 0));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(adjacent_cells_indices, vec![(1, 0), (0, 1), (1, 1)])
    }

    #[test]
    fn get_adjacent_cells_indices_edge_case_max_max() {
        let cell = Cell::new((19, 19));
        let adjacent_cells_indices = cell.get_adjacent_cells_indices((20, 20));

        assert_eq!(adjacent_cells_indices, vec![(18, 18), (19, 18), (18, 19)])
    }
}
