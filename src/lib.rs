pub mod field;
mod stopwatch;

pub use field::cell::Cell;
pub use field::{Field, FieldError};

#[derive(Debug)]
pub enum MinesweeperError {
    FieldError(FieldError),
    GameAlreadyEnded,
}

impl From<FieldError> for MinesweeperError {
    fn from(field_error: FieldError) -> Self {
        MinesweeperError::FieldError(field_error)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum MinesweeperStatus {
    /// After the field has been created, but before it has been initialized with mines and numbers.
    Pre,
    /// An ongoing game.
    On,
    /// A paused game.
    Pause,
    /// A finished game. `true` for victory, `false` for loss.
    End(bool),
}

#[derive(Debug)]
pub enum MinesweeperAction {
    /// A request to open a cell by its position.
    OpenCell((u8, u8)),
    /// A request to open the cells adjacent to the one with the provided position.
    OpenSurroundingCells((u8, u8)),
    /// This action is a combination of the 2 ones above with it's automatically deciding which one to use exactly.
    ///
    /// This is intended to be used with frontends which have a limited number of inputs, so that both actions could use
    /// the same trigger.
    OpenCellOrSurroundingCells((u8, u8)),
    /// A request to flag a cell by its position.
    FlagCell((u8, u8)),
}

#[derive(Debug)]
pub struct Minesweeper {
    pub field: Field,
    pub mines_amount: u16,
    pub status: MinesweeperStatus,
    stopwatch: stopwatch::Stopwatch,
}

impl Minesweeper {
    pub fn new(
        rows_amount: u8,
        columns_amount: u8,
        mines_amount: u16,
    ) -> Result<Self, MinesweeperError> {
        let field = Field::new(rows_amount, columns_amount, mines_amount)?;

        Ok(Minesweeper {
            field,
            mines_amount,
            status: MinesweeperStatus::Pre,
            stopwatch: stopwatch::Stopwatch::default(),
        })
    }

    pub fn get_cell(&self, position: (u8, u8)) -> Option<&Cell> {
        self.field.get_cell(position)
    }

    pub fn get_flagged_cells_amount(&self) -> u16 {
        self.field.get_flagged_cells_amount()
    }

    pub fn take_action(
        &mut self,
        action_type: MinesweeperAction,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        // early-return an error if trying to take an action when the game has already ended
        if let MinesweeperStatus::End(_) = self.status {
            return Err(MinesweeperError::GameAlreadyEnded);
        }

        // early-return the current status (in other words, don't do anything) if trying to take some action when the
        // game is paused
        if let MinesweeperStatus::Pause = self.status {
            return Ok(&MinesweeperStatus::Pause);
        }

        // match and perform the requested action
        match action_type {
            MinesweeperAction::OpenCell(cell_position) => {
                if let MinesweeperStatus::On = self.status {
                } else {
                    self.field.populate_with_mines(Some(cell_position))?;

                    self.status = MinesweeperStatus::On;

                    self.stopwatch.start()
                }

                self.field.open_cell(cell_position);
            }
            MinesweeperAction::OpenSurroundingCells(cell_position) => {
                self.field.open_surrounding_cells(cell_position);
            }
            MinesweeperAction::OpenCellOrSurroundingCells(cell_position) => {
                let target_cell = self.get_cell(cell_position);

                if let Some(cell) = target_cell {
                    // we're not calling the underlying method here directly because this action is just an alias
                    if cell.is_open() {
                        // for the already-open cells take the `OpenSurroundingCells` action
                        self.take_action(MinesweeperAction::OpenSurroundingCells(cell_position))?;
                    } else {
                        // for the closed cells take the `OpenCell` action
                        self.take_action(MinesweeperAction::OpenCell(cell_position))?;
                    }
                }
            }
            MinesweeperAction::FlagCell(cell_position) => {
                self.field.flag_cell(cell_position);
            }
        };

        self.update_status();
        Ok(&self.status)
    }

    fn update_status(&mut self) {
        if let Some(victory) = self.check_victory_or_loss() {
            if !victory {
                // open all the missed mines when the game is lost
                self.field.open_missed_mines()
            };

            self.status = MinesweeperStatus::End(victory);
            self.stopwatch.stop();
        }
    }

    fn check_victory_or_loss(&self) -> Option<bool> {
        let loss = self.field.check_open_mines_exist();
        let victory = self.field.check_all_non_mines_open();

        if loss {
            Some(false)
        } else if victory {
            Some(true)
        } else {
            None
        }
    }

    pub fn toggle_pause(&mut self) {
        // it's only possible to pause an ongoing game
        if let MinesweeperStatus::On = self.status {
            self.status = MinesweeperStatus::Pause;
            self.stopwatch.stop();
        } else if let MinesweeperStatus::Pause = self.status {
            self.status = MinesweeperStatus::On;
            self.stopwatch.start();
        };
    }

    pub fn get_time(&self) -> u64 {
        self.stopwatch.get_elapsed_time().as_secs()
    }
}
