use std::fmt::{Debug, Display, Formatter};

/// The cell variant.
///
/// A cell can either be empty or contain a mine.
#[derive(Debug, PartialEq, Eq)]
enum CellVariant {
    /// Represents an empty cell. The empty cell is one that doesn't contain a mine.
    ///
    /// The parameter represents the number of mines around the cell.
    Empty(u8),
    /// Represents a mined cell.
    Mine,
}

/// The cell's state.
///
/// A cell can either be open or closed. When closed, it can also either be or not be flagged.
#[derive(Debug, PartialEq, Eq)]
enum CellState {
    /// Represents a closed cell.
    ///
    /// The boolean value indicates whether the cell is flagged (`true`) or not (`false`).
    Closed(bool),
    /// Represents an open cell.
    Open,
}

/// The representation of a cell.
///
/// A cell is described with its position in the field, a variant and a state.
#[derive(PartialEq, Eq)]
pub struct Cell {
    /// The cell's position in the field is represented with its row's and column's indices (respectively).
    position: (u8, u8),
    /// The cell's variant is either of the `CellVariant` enum.
    variant: CellVariant,
    /// The cell's state is either of the `CellState` enum.
    state: CellState,
}

impl Cell {
    /// Creates a new closed not flagged empty `Cell` instance with the position provided.
    pub fn new(position: (u8, u8)) -> Self {
        Cell {
            position,
            variant: CellVariant::Empty(0),
            state: CellState::Closed(false),
        }
    }

    /// Checks whether the cell is mined.
    pub fn is_mined(&self) -> bool {
        self.variant == CellVariant::Mine
    }

    /// Mines the cell in-place.
    pub fn mine(&mut self) {
        self.variant = CellVariant::Mine;
    }

    /// Returns the amount of mines around the cell or `None` if the cell itself is mined.
    pub fn get_mines_around_amount(&self) -> Option<u8> {
        if let CellVariant::Empty(adjacent_mines_amount) = self.variant {
            Some(adjacent_mines_amount)
        } else {
            None
        }
    }

    /// Increments the value representing the number of mines around the cell.
    ///
    /// Won't produce any effect if the cell itself is mined.
    pub fn increment_mines_around_amount(&mut self) {
        if let CellVariant::Empty(adjacent_mines_amount) = self.variant {
            self.variant = CellVariant::Empty(adjacent_mines_amount + 1);
        }
    }

    /// Checks whether the cell is open.
    pub fn is_open(&self) -> bool {
        self.state == CellState::Open
    }

    /// Opens the cell in-place.
    pub fn open(&mut self) {
        self.state = CellState::Open;
    }

    /// Checks whether the cell is flagged.
    pub fn is_flagged(&self) -> bool {
        if let CellState::Closed(is_flagged) = self.state {
            is_flagged
        } else {
            false
        }
    }

    /// Toggles the flag of the cell in-place.
    ///
    /// Won't produce any effect if the cell itself is open.
    pub fn toggle_flag(&mut self) {
        if let CellState::Closed(is_flagged) = self.state {
            self.state = CellState::Closed(!is_flagged)
        };
    }

    /// Returns the positions of the cell's adjacent cells.
    ///
    /// The method implies an infinite field, so the caller must double check the returned values with respect
    /// to the field's dimensions (so that there are no out-of-bounds cells' positions).
    pub fn get_adjacent_cells_positions(&self) -> Vec<(u8, u8)> {
        // Transform the cell's coordinates into `i16` to be able to subtract and add without overflow.
        let (row_index, column_index) = (self.position.0 as i16, self.position.1 as i16);

        // Create a 2D vector of all the cells' indices surrounding the current one.
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
        // Flatten the 2D vector for an easier filtration.
        .flatten()
        .filter(|(row_index, column_index)| {
            // Filter out all the cells' indices that go beyond the field's dimensions. Namely, where the row's and
            // column's indices are less than 0 (the case of the first row/column).
            *row_index >= 0 && *column_index >= 0
        })
        // Convert the coordinates back into `u8`.
        .map(|(row_index, column_index)| (row_index as u8, column_index as u8))
        .collect::<Vec<(u8, u8)>>()
    }
}

/// The `Debug` implementation displays the closed cells as open.
impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let CellState::Closed(true) = self.state {
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

/// The `Display` implementation represents the cell in a real-game fashion.
impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.state {
            // In the real game, the cells don't reveal their inner state.
            CellState::Closed(is_flagged) => {
                if is_flagged {
                    write!(f, "🚩")
                } else {
                    write!(f, "⬛ ")
                }
            }
            // The rest of the cases is successfully covered with the `Debug` trait's implementation.
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Cell, CellState, CellVariant};

    #[test]
    fn create_a_cell_instance() {
        let cell = Cell::new((10, 10));

        assert_eq!(
            cell,
            Cell {
                position: (10, 10),
                variant: CellVariant::Empty(0),
                state: CellState::Closed(false)
            }
        );
    }

    #[test]
    fn mine_cell_and_is_mine() {
        let mut cell = Cell::new((10, 10));
        assert!(!cell.is_mined());

        cell.mine();
        assert!(cell.is_mined());
    }

    #[test]
    fn increment_mines_around_amount_and_get_mines_around_amount_for_an_empty_cell() {
        let mut cell = Cell::new((10, 10));
        assert_eq!(cell.get_mines_around_amount().unwrap(), 0);

        cell.increment_mines_around_amount();
        cell.increment_mines_around_amount();
        cell.increment_mines_around_amount();

        assert_eq!(cell.get_mines_around_amount().unwrap(), 3);
    }

    #[test]
    fn increment_mines_around_amount_and_get_mines_around_amount_for_a_mined_cell() {
        let mut cell = Cell::new((10, 10));
        cell.mine();
        assert_eq!(cell.get_mines_around_amount(), None);

        cell.increment_mines_around_amount();
        cell.increment_mines_around_amount();
        cell.increment_mines_around_amount();

        assert_eq!(cell.get_mines_around_amount(), None);
    }

    #[test]
    fn open_and_is_open() {
        let mut cell = Cell::new((10, 10));
        assert!(!cell.is_open());

        cell.open();
        assert!(cell.is_open());
    }

    #[test]
    fn toggle_flag_and_is_flagged_for_an_empty_cell() {
        let mut cell = Cell::new((10, 10));
        assert!(!cell.is_flagged());

        cell.toggle_flag();
        assert!(cell.is_flagged());

        cell.toggle_flag();
        assert!(!cell.is_flagged());
    }

    #[test]
    fn toggle_flag_and_is_flagged_for_an_open_cell() {
        let mut cell = Cell::new((10, 10));
        cell.open();
        assert!(!cell.is_flagged());

        cell.toggle_flag();
        assert!(!cell.is_flagged());

        cell.toggle_flag();
        assert!(!cell.is_flagged());
    }

    #[test]
    fn get_adjacent_cells_positions_for_middle_cell() {
        let cell = Cell::new((10, 10));
        let adjacent_cells_positions = cell.get_adjacent_cells_positions();

        assert_eq!(
            adjacent_cells_positions,
            [
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
    fn get_adjacent_cells_positions_for_0th_row() {
        let cell = Cell::new((0, 10));
        let adjacent_cells_positions = cell.get_adjacent_cells_positions();

        assert_eq!(
            adjacent_cells_positions,
            [(0, 9), (1, 9), (1, 10), (0, 11), (1, 11)]
        );
    }

    #[test]
    fn get_adjacent_cells_positions_for_0th_column() {
        let cell = Cell::new((10, 0));
        let adjacent_cells_positions = cell.get_adjacent_cells_positions();

        assert_eq!(
            adjacent_cells_positions,
            [(9, 0), (11, 0), (9, 1), (10, 1), (11, 1)]
        );
    }

    #[test]
    fn get_adjacent_cells_positions_for_0_0_edge_case() {
        let cell = Cell::new((0, 0));
        let adjacent_cells_positions = cell.get_adjacent_cells_positions();

        assert_eq!(adjacent_cells_positions, [(1, 0), (0, 1), (1, 1)]);
    }
}
