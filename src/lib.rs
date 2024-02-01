mod field;
mod stopwatch;

use field::{Field, FieldError};
use stopwatch::Stopwatch;

/// The enum represents the variants of everything that can possibly go wrong during the game.
#[derive(Debug)]
enum MinesweeperError {
    /// This is used when something's wrong with the field. The `FieldError` variant is just a wrapper for the original
    /// [`FieldError`] type. The [`From`] trait is implemented for the `MinesweeperError` to en-wrap with it
    /// `FieldError`s.
    FieldError(FieldError),
    /// The error indicates that the game has already ended, and therefore the requested action could not be performed.
    GameAlreadyEnded,
}

impl From<FieldError> for MinesweeperError {
    fn from(field_error: FieldError) -> Self {
        MinesweeperError::FieldError(field_error)
    }
}

/// The status of a game.
#[derive(Debug, Eq, PartialEq)]
enum MinesweeperStatus {
    /// After the field has been created, but before it has been initialized with mines and numbers.
    Pre,
    /// An ongoing game.
    On,
    /// A paused game.
    Pause,
    /// A finished game. `true` for victory, `false` for loss.
    End(bool),
}

/// Describes all the possible action a user can take.
#[derive(Debug)]
enum MinesweeperAction {
    /// A request to open a cell by its position.
    OpenCell((u8, u8)),
    /// A request to open the cells adjacent to the one with the provided position.
    OpenSurroundingCells((u8, u8)),
    /// This action is a combination of the two ones above with it's automatically deciding which one to use exactly.
    ///
    /// This is intended to be used with frontends which have a limited number of inputs, so that both actions could use
    /// the same trigger.
    OpenCellOrSurroundingCells((u8, u8)),
    /// A request to flag a cell by its position.
    FlagCell((u8, u8)),
}

/// The struct representing a Minesweeper game itself.
#[derive(Debug)]
struct Minesweeper {
    /// The field used in the game.
    field: Field,
    /// The game status.
    status: MinesweeperStatus,
    /// The in-game stopwatch. It's started as soon as the first cell gets opened and is paused when the game is paused.
    stopwatch: Stopwatch,
}

impl Minesweeper {
    fn new(
        rows_amount: u8,
        columns_amount: u8,
        mines_amount: u16,
    ) -> Result<Self, MinesweeperError> {
        let field = Field::new(rows_amount, columns_amount, mines_amount)?;

        Ok(Minesweeper {
            field,
            status: MinesweeperStatus::Pre,
            stopwatch: Stopwatch::default(),
        })
    }

    /// The method performs the requested action, updates the status of the game and returns it.
    ///
    /// Might fail with a [`MinesweeperError`] in case something goes wrong.
    pub fn take_action(
        &mut self,
        action_type: MinesweeperAction,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        // Early-return an error if trying to take an action when the game has already ended.
        if let MinesweeperStatus::End(_) = self.status {
            return Err(MinesweeperError::GameAlreadyEnded);
        }

        // Early-return the current status (in other words, don't do anything) if trying to take some action when the
        // game is paused.
        if let MinesweeperStatus::Pause = self.status {
            return Ok(&MinesweeperStatus::Pause);
        }

        // Match and perform the requested action.
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
                let target_cell = self.field.get_cell(cell_position);

                if let Some(cell) = target_cell {
                    // We're not calling the underlying method here directly because this action is just an alias.
                    if cell.is_open() {
                        // For the already-open cells, perform the `OpenSurroundingCells` action.
                        self.take_action(MinesweeperAction::OpenSurroundingCells(cell_position))?;
                    } else {
                        // For the closed ones, perform the `OpenCell` action.
                        self.take_action(MinesweeperAction::OpenCell(cell_position))?;
                    }
                }
            }
            MinesweeperAction::FlagCell(cell_position) => {
                self.field.toggle_cell_flag(cell_position);
            }
        };

        self.update_status();
        Ok(&self.status)
    }

    /// A private helper that updates the game status. Should be called after each action that can potentially change
    /// it.
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

    /// The method is a private helper that determines whether the game has been lost or won. If neither (ongoing),
    /// returns the `None` value.
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

    /// Toggles the pause on the game's stopwatch.
    ///
    /// The frontends should take care of hiding the field during pauses themselves.
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

    /// Returns the total amount of time the game has been in the `On` status.
    pub fn get_time(&self) -> u64 {
        self.stopwatch.get_elapsed_time().as_secs()
    }
}
