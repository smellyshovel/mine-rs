use std::fmt::{Debug, Display, Formatter};

/// The cell variant.
///
/// A cell can either be empty or contain a mine.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CellVariant {
    /// Represents an empty cell. The empty cell is one that doesn't contain a mine.
    ///
    /// The parameter represents the amount of mines around the cell.
    Empty(u8),
    /// Represents a mined cell.
    Mine,
}

/// The cell's state.
///
/// A cell can either be open or closed. When closed, it can also either be or not be flagged.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CellState {
    /// Represents a closed cell.
    ///
    /// The boolean value indicates whether the cell's flagged (`true`) or not (`false`).
    Closed(bool),
    /// Represents an open cell.
    Open,
}

/// The representation of a cell.
///
/// A cell is described with its position in the field, a variant and a state.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The cell's position in the field is represented with its row's and column's indices (respectively).
    position: (u8, u8),
    /// The cell's variant is either of the `CellVariant` enum.
    variant: CellVariant,
    /// The cell's state is either of the `CellState` enum.
    state: CellState,
}

impl Cell {
    /// Creates a new closed un-flagged empty `Cell` instance with the position provided.
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

    /// Mines the cell and returns the updated itself.
    pub fn mine(&mut self) -> Self {
        *self = Cell {
            position: self.position,
            variant: CellVariant::Mine,
            state: self.state,
        };

        *self
    }

    /// Returns the amount of mines around the cell or `None` if the cell itself is mined.
    pub fn get_mines_around_amount(&self) -> Option<u8> {
        if let CellVariant::Empty(adjacent_mines_amount) = self.variant {
            Some(adjacent_mines_amount)
        } else {
            None
        }
    }

    /// Increments the number representing the amount of mines around the cell.
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

    /// Opens the cell and returns the updated itself.
    pub fn open(&mut self) -> Self {
        self.state = CellState::Open;
        *self
    }

    /// Check whether the cell is flagged.
    pub fn is_flagged(&self) -> bool {
        if let CellState::Closed(is_flagged) = self.state {
            is_flagged
        } else {
            false
        }
    }

    /// Toggles the flag of the cell and returns the updated itself.
    ///
    /// Won't produce any effect if the cell itself is open.
    pub fn toggle_flag(&mut self) -> Self {
        if let CellState::Closed(is_flagged) = self.state {
            self.state = CellState::Closed(!is_flagged)
        };

        *self
    }

    /// Returns the positions of the cell's adjacent cells.
    ///
    /// The method implies an infinite field, so the returned values must be double-checked by the caller with respect
    /// for the field's dimensions (so that there are no out-of-bounds cells' positions).
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
            // column's indices are less than 0 (the case of the first  row/column).
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
//
// #[cfg(test)]
// mod test {
//     use super::{Cell, CellState, CellVariant};
//
//     #[test]
//     fn new_creates_a_closed_empty_cell_with_the_provided_position() {
//         let new_cell = Cell::new((5, 5));
//
//         assert_eq!(
//             new_cell,
//             Cell {
//                 position: (5, 5),
//                 variant: CellVariant::Empty(0),
//                 state: CellState::Closed
//             }
//         );
//     }
//
//     #[test]
//     fn is_mine_correctly_determines_the_presence_of_a_mine() {
//         let mut mined_cell = Cell::new((5, 5));
//         assert!(!mined_cell.is_mined());
//
//         mined_cell.mine();
//         assert!(mined_cell.is_mined());
//     }
//
//     #[test]
//     fn mine_correctly_makes_the_cell_a_mine() {
//         let mut mined_cell = Cell::new((5, 5));
//         assert!(!mined_cell.is_mined());
//
//         mined_cell.mine();
//         assert!(mined_cell.is_mined());
//     }
//
//     #[test]
//     fn mine_does_not_produce_no_side_effects() {
//         let mut cell = Cell::new((5, 5));
//         cell.mine();
//
//         assert_eq!(
//             cell,
//             Cell {
//                 position: (5, 5),
//                 variant: CellVariant::Mine,
//                 state: CellState::Closed,
//             }
//         );
//     }
//
//     #[test]
//     fn get_adjacent_mines_amount_returns_the_correct_amount_of_adjacent_mines() {
//         let mut adjacent_cell = Cell::new((5, 5));
//
//         adjacent_cell.increment_adjacent_mines_amount(); // 1
//         adjacent_cell.increment_adjacent_mines_amount(); // 2
//         adjacent_cell.increment_adjacent_mines_amount(); // 3
//
//         assert_eq!(adjacent_cell.get_adjacent_mines_amount().unwrap(), 3_u8);
//     }
//
//     #[test]
//     fn get_adjacent_mines_amount_returns_none_when_the_cell_is_a_mine_itself() {
//         let mut mined_cell = Cell::new((5, 5));
//         mined_cell.mine();
//
//         assert!(mined_cell.get_adjacent_mines_amount().is_none());
//     }
//
//     #[test]
//     fn increment_adjacent_mines_amount_properly_increments_the_value() {
//         let mut cell = Cell::new((5, 5));
//         cell.increment_adjacent_mines_amount();
//
//         assert_eq!(cell.get_adjacent_mines_amount().unwrap(), 1_u8);
//     }
//
//     #[test]
//     fn increment_adjacent_mines_amount_does_not_do_anything_if_the_cell_is_a_mine() {
//         let mut cell = Cell::new((5, 5));
//         cell.mine();
//
//         cell.increment_adjacent_mines_amount();
//         assert!(cell.is_mined());
//     }
//
//     #[test]
//     fn is_open_correctly_determines_whether_the_cell_is_open() {
//         let mut cell = Cell::new((5, 5));
//         assert!(!cell.is_open());
//
//         cell.open((10, 10));
//         assert!(cell.is_open());
//     }
//
//     #[test]
//     fn open_correctly_opens_a_closed_cell() {
//         let mut cell = Cell::new((5, 5));
//
//         cell.open((10, 10));
//         assert!(cell.is_open());
//     }
//
//     #[test]
//     fn open_returns_an_empty_vector_when_opening_an_already_opened_cell() {
//         let mut cell = Cell::new((5, 5));
//         cell.open((10, 10));
//
//         let res = cell.open((10, 10));
//
//         assert!(cell.is_open());
//         assert!(res.is_empty());
//     }
//
//     #[test]
//     fn open_returns_adjacent_cells_if_the_opened_cell_is_empty() {
//         let mut cell = Cell::new((10, 10));
//
//         let res = cell.open((20, 20));
//
//         assert!(cell.is_open());
//         assert_eq!(
//             res,
//             vec![
//                 (9, 9),
//                 (10, 9),
//                 (11, 9),
//                 (9, 10),
//                 (11, 10),
//                 (9, 11),
//                 (10, 11),
//                 (11, 11)
//             ]
//         );
//     }
//
//     #[test]
//     fn open_returns_an_empty_vector_if_the_cell_borders_a_mine() {
//         let mut cell = Cell::new((10, 10));
//         cell.increment_adjacent_mines_amount();
//
//         let res = cell.open((20, 20));
//
//         assert!(cell.is_open());
//         assert!(res.is_empty());
//     }
//
//     #[test]
//     fn open_wont_open_the_cell_if_it_is_flagged() {
//         let mut cell = Cell::new((10, 10));
//         cell.toggle_flag();
//
//         cell.open((20, 20));
//
//         assert!(!cell.is_open() && cell.is_flagged());
//     }
//
//     #[test]
//     fn is_flagged_correctly_determines_whether_the_cell_is_flagged() {
//         let mut cell = Cell::new((5, 5));
//         assert!(!cell.is_flagged());
//
//         cell.toggle_flag();
//         assert!(cell.is_flagged());
//     }
//
//     #[test]
//     fn toggle_flag_correctly_toggles_the_flag() {
//         let mut cell = Cell::new((5, 5));
//         assert!(!cell.is_flagged());
//
//         cell.toggle_flag();
//         assert!(cell.is_flagged());
//
//         cell.toggle_flag();
//         assert!(!cell.is_flagged());
//     }
//
//     #[test]
//     fn toggle_flag_does_not_do_anything_for_open_cells() {
//         let mut cell = Cell::new((5, 5));
//         cell.open((10, 10));
//         assert!(cell.is_open());
//
//         cell.toggle_flag();
//         assert!(cell.is_open());
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_works_correctly_for_inner_cells() {
//         let cell = Cell::new((10, 10));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(
//             adjacent_cells_indices,
//             vec![
//                 (9, 9),
//                 (10, 9),
//                 (11, 9),
//                 (9, 10),
//                 (11, 10),
//                 (9, 11),
//                 (10, 11),
//                 (11, 11)
//             ]
//         )
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_first_row() {
//         let cell = Cell::new((0, 10));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(
//             adjacent_cells_indices,
//             vec![(0, 9), (1, 9), (1, 10), (0, 11), (1, 11)]
//         )
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_first_column() {
//         let cell = Cell::new((10, 0));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(
//             adjacent_cells_indices,
//             vec![(9, 0), (11, 0), (9, 1), (10, 1), (11, 1)]
//         )
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_last_row() {
//         let cell = Cell::new((19, 10));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(
//             adjacent_cells_indices,
//             vec![(18, 9), (19, 9), (18, 10), (18, 11), (19, 11)]
//         )
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_last_column() {
//         let cell = Cell::new((10, 19));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(
//             adjacent_cells_indices,
//             vec![(9, 18), (10, 18), (11, 18), (9, 19), (11, 19)]
//         )
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_zero_zero() {
//         let cell = Cell::new((0, 0));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(adjacent_cells_indices, vec![(1, 0), (0, 1), (1, 1)])
//     }
//
//     #[test]
//     fn get_adjacent_cells_indices_edge_case_max_max() {
//         let cell = Cell::new((19, 19));
//         let adjacent_cells_indices = cell.get_adjacent_cells_positions((20, 20));
//
//         assert_eq!(adjacent_cells_indices, vec![(18, 18), (19, 18), (18, 19)])
//     }
// }
