//! The terminal application

use crate::app::MenuItem::{ColumnsAmount, MinesAmount, RowsAmount};
use crate::app::MoveCursorDirection::{Down, Left, Right, Up};
use crate::game_ui::render_game;
use crate::menu_ui::render_menu;
use crate::tui::Render;
use crate::update::{ControlsSupport, MoveCursorDirection};
pub use mine_rs::Minesweeper;
use mine_rs::{MinesweeperAction, MinesweeperError, MinesweeperStatus};
use ratatui::Frame;
use std::cmp;

const DEFAULT_ROWS_AMOUNT: u8 = 16;
const DEFAULT_COLUMNS_AMOUNT: u8 = 16;
const DEFAULT_MINES_AMOUNT: u16 = 40;

/// The terminal application
#[derive(Debug)]
pub struct App {
    /// The app.rs can be represented by one variant at a time.
    pub variant: AppVariant,
    /// Indicates that the main application loop should be broken on the next tick and thus the app.rs should quit.
    pub should_quit: bool,
}

impl App {
    pub fn new(
        rows_amount: Option<u8>,
        columns_amount: Option<u8>,
        mines_amount: Option<u16>,
    ) -> Result<App, MinesweeperError> {
        Ok(App {
            variant: if let (Some(rows_amount), Some(columns_amount), Some(mines_amount)) =
                (rows_amount, columns_amount, mines_amount)
            {
                AppVariant::InGame(AppGame::new(rows_amount, columns_amount, mines_amount)?)
            } else {
                AppVariant::InMenu(AppMenu::new(rows_amount, columns_amount, mines_amount))
            },
            should_quit: false,
        })
    }

    pub fn tick(&mut self) {
        match &self.variant {
            AppVariant::InMenu(menu) if menu.should_quit => self.quit(),
            AppVariant::InGame(game) => {
                if game.should_leave {
                    self.back_to_menu()
                } else if game.should_emergency_leave {
                    self.quit()
                }
            }
            _ => (),
        };
    }

    pub fn back_to_menu(&mut self) {
        if let AppVariant::InGame(game) = &self.variant {
            let (rows_amount, columns_amount, _) = game.game.get_field().get_size();
            self.variant = AppVariant::InMenu(AppMenu::new(
                Some(rows_amount),
                Some(columns_amount),
                Some(game.game.get_field().get_mines_amount()),
            ))
        };
    }

    /// Quit the application altogether.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl ControlsSupport for App {
    fn move_cursor(&mut self, direction: MoveCursorDirection) {
        self.variant.move_cursor(direction);
    }

    fn perform_main_action(&mut self) -> Result<(), MinesweeperError> {
        self.variant.perform_main_action()
    }

    fn perform_secondary_action(&mut self) -> Result<(), MinesweeperError> {
        self.variant.perform_secondary_action()
    }

    fn pause(&mut self) {
        self.variant.pause();
    }

    fn leave(&mut self, force: bool) {
        self.variant.leave(force);
    }
}

impl Render for App {
    fn render(&mut self, frame: &mut Frame) {
        self.variant.render(frame);
    }
}

/// The current application variant.
#[derive(Debug)]
pub enum AppVariant {
    /// When the menu's being displayed
    InMenu(AppMenu),
    /// When the game's being displayed
    InGame(AppGame),
}

impl ControlsSupport for AppVariant {
    fn move_cursor(&mut self, direction: MoveCursorDirection) {
        match self {
            AppVariant::InMenu(menu) => menu.move_cursor(direction),
            AppVariant::InGame(game) => game.move_cursor(direction),
        }
    }

    fn perform_main_action(&mut self) -> Result<(), MinesweeperError> {
        match self {
            AppVariant::InMenu(menu) => {
                let game = menu.create_new_game();

                if game.is_ok() {
                    *self = AppVariant::InGame(game.unwrap());
                } else {
                    menu.error = game.err();
                }
            }
            AppVariant::InGame(game) => {
                let result = game.open_cell_or_surrounding_cells_or_confirm_leave()?;

                if let Some((rows_amount, columns_amount, mines_amount)) = result {
                    *self = AppVariant::InGame(AppGame::new(
                        rows_amount,
                        columns_amount,
                        mines_amount,
                    )?);
                }
            }
        }

        Ok(())
    }

    fn perform_secondary_action(&mut self) -> Result<(), MinesweeperError> {
        match self {
            AppVariant::InMenu(menu) => menu.restore_default(),
            AppVariant::InGame(game) => game.toggle_flag()?,
        }

        Ok(())
    }

    fn pause(&mut self) {
        // it's only possible to toggle the pause for the game, not for the menu
        if let AppVariant::InGame(game) = self {
            // don't toggle the pause when the game's wating for leave confirnation
            if !game.awaiting_leave_confirmation {
                game.game.toggle_pause();
            }
        }
    }

    fn leave(&mut self, force: bool) {
        match self {
            AppVariant::InMenu(menu) => {
                menu.quit();
            }
            AppVariant::InGame(game) => {
                if force {
                    game.emergency_leave();
                } else {
                    game.confirm_or_cancel_leave_or_leave();
                }
            }
        }
    }
}

impl Render for AppVariant {
    fn render(&mut self, frame: &mut Frame) {
        match self {
            AppVariant::InMenu(ref mut menu) => render_menu(menu, frame),
            AppVariant::InGame(ref mut game) => render_game(game, frame),
        }
    }
}

/// The Menu app.rs variant
#[derive(Debug)]
pub struct AppMenu {
    pub rows_amount: u8,
    pub columns_amount: u8,
    pub mines_amount: u16,
    pub selected_item: MenuItem,
    pub error: Option<MinesweeperError>,
    should_quit: bool,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum MenuItem {
    ColumnsAmount,
    RowsAmount,
    MinesAmount,
}

impl AppMenu {
    fn new(rows_amount: Option<u8>, columns_amount: Option<u8>, mines_amount: Option<u16>) -> Self {
        AppMenu {
            rows_amount: rows_amount.unwrap_or(DEFAULT_ROWS_AMOUNT),
            columns_amount: columns_amount.unwrap_or(DEFAULT_COLUMNS_AMOUNT),
            mines_amount: mines_amount.unwrap_or(DEFAULT_MINES_AMOUNT),
            selected_item: ColumnsAmount,
            error: None,
            should_quit: false,
        }
    }

    fn move_cursor(&mut self, direction: MoveCursorDirection) {
        let layout = [ColumnsAmount, RowsAmount, MinesAmount];

        let mut current_index = layout
            .iter()
            .position(|x| x == &self.selected_item)
            .unwrap();

        match direction {
            Up => {
                current_index = current_index.saturating_sub(1);
            }
            Down => {
                let new_val = current_index.saturating_add(1);

                current_index = if new_val >= layout.len() - 1 {
                    layout.len() - 1
                } else {
                    new_val
                }
            }
            Left => {
                match self.selected_item {
                    ColumnsAmount => self.columns_amount = self.columns_amount.saturating_sub(1),
                    RowsAmount => self.rows_amount = self.rows_amount.saturating_sub(1),
                    MinesAmount => self.mines_amount = self.mines_amount.saturating_sub(1),
                };
            }
            Right => {
                match self.selected_item {
                    ColumnsAmount => self.columns_amount = self.columns_amount.saturating_add(1),
                    RowsAmount => self.rows_amount = self.rows_amount.saturating_add(1),
                    MinesAmount => self.mines_amount = self.mines_amount.saturating_add(1),
                };
            }
        };

        self.selected_item = layout.get(current_index).unwrap().clone();
    }

    fn create_new_game(&self) -> Result<AppGame, MinesweeperError> {
        AppGame::new(self.rows_amount, self.columns_amount, self.mines_amount)
    }

    fn restore_default(&mut self) {
        match self.selected_item {
            ColumnsAmount => self.columns_amount = DEFAULT_COLUMNS_AMOUNT,
            RowsAmount => self.rows_amount = DEFAULT_ROWS_AMOUNT,
            MinesAmount => self.mines_amount = DEFAULT_MINES_AMOUNT,
        };
    }

    fn quit(&mut self) {
        self.should_quit = true
    }
}

/// The Game app.rs variant
#[derive(Debug)]
pub struct AppGame {
    /// The game instance.
    pub game: Minesweeper,
    /// The amount of rows that should be rendered in the field. Must always be less or equal to the total amount of
    /// rows.
    pub visible_rows_amount: u8,
    /// The amount of columns that should be rendered in the field. Must always be less or equal to the total amount of
    /// columns.
    pub visible_columns_amount: u8,
    /// The window is a sliding frame-view into the field. This is used when the field is too big to be displayed in the
    /// given container.
    ///
    /// The values represent the starting row/column from which the visible amount of rows/columns is displayed.
    ///
    /// So, for example, for the 5x5 field where there would only be 3 visible rows and 3 visible columns, in order to
    /// only display the portion of the field shown below, the `window_offset` must be set to `(2, 2)`.
    ///
    /// ```
    /// /*
    ///
    /// * * * * *
    /// * * * * *
    ///    _______
    /// * *|* * *|
    /// * *|* * *|
    /// * *|* * *|
    ///    _______
    ///
    /// */
    pub window_offset: (u8, u8),
    /// The position of the currently selected cell relative to the whole field. Must be added to the `window_offset`
    /// in order to get the position of the currently selected cell relative to the window (the visible part of the
    /// field).
    pub cursor_position: (u8, u8),
    /// Whether the cancel key was pressed and now the game's in the state of waiting for a confirmation from the user
    /// to leave back to the menu.
    pub awaiting_leave_confirmation: bool,
    /// Whether the leave was confirmed and now it's allowed to go back to the menu.
    pub should_leave: bool,
    /// Whether the app.rs should urgently leave without asking for a confirmation
    pub should_emergency_leave: bool,
}

impl AppGame {
    fn new(
        rows_amount: u8,
        columns_amount: u8,
        mines_amount: u16,
    ) -> Result<Self, MinesweeperError> {
        let game = Minesweeper::new(rows_amount, columns_amount, mines_amount)?;

        Ok(AppGame {
            game,
            visible_rows_amount: 0,
            visible_columns_amount: 0,
            window_offset: (0, 0),
            cursor_position: (0, 0),
            awaiting_leave_confirmation: false,
            should_leave: false,
            should_emergency_leave: false,
        })
    }

    fn move_cursor(&mut self, direction: MoveCursorDirection) {
        // don't move the cursor when the game's paused or when it's already finished
        if let MinesweeperStatus::Pause | MinesweeperStatus::End(_) = self.game.get_status() {
            return;
        }

        let (field_height, field_width, _) = self.game.get_field().get_size();
        let (cy, cx) = self.cursor_position;

        self.cursor_position = match direction {
            Up => (cy.saturating_sub(1), cx),
            Left => (cy, cx.saturating_sub(1)),
            Down => (cmp::min(cy + 1, field_height - 1), cx),
            Right => (cy, cmp::min(cx + 1, field_width - 1)),
        };

        let (cy, cx) = self.cursor_position;
        let (oy, ox) = self.window_offset;

        self.window_offset = {
            let new_oy = if cy > oy + self.visible_rows_amount - 2 {
                cmp::min(oy + 1, field_height - self.visible_rows_amount)
            } else if cy < oy + 1 {
                oy.saturating_sub(1)
            } else {
                self.window_offset.0
            };

            let new_ox = if cx > ox + self.visible_columns_amount - 2 {
                cmp::min(ox + 1, field_width - self.visible_columns_amount)
            } else if cx < ox + 1 {
                ox.saturating_sub(1)
            } else {
                self.window_offset.1
            };

            (new_oy, new_ox)
        }
    }

    fn open_cell_or_surrounding_cells_or_confirm_leave(
        &mut self,
    ) -> Result<Option<(u8, u8, u16)>, MinesweeperError> {
        if self.awaiting_leave_confirmation {
            self.leave();
            return Ok(None);
        }

        if let MinesweeperStatus::End(_) = self.game.get_status() {
            // if the game has ended, start a new one
            let (h, w, _) = self.game.get_field().get_size();
            return Ok(Some((h, w, self.game.get_field().get_mines_amount())));
        } else {
            // otherwise, open a cell or surrounding cells
            self.game
                .take_action(MinesweeperAction::OpenCellOrSurroundingCells(
                    self.cursor_position,
                ))?;
        }

        Ok(None)
    }

    fn toggle_flag(&mut self) -> Result<(), MinesweeperError> {
        if let MinesweeperStatus::On = self.game.get_status() {
            self.game
                .take_action(MinesweeperAction::FlagCell(self.cursor_position))?;
        }

        Ok(())
    }

    fn confirm_or_cancel_leave_or_leave(&mut self) {
        if let MinesweeperStatus::End(_) = self.game.get_status() {
            // if the game has ended, just leave without asking for confirmation
            self.leave();
        } else {
            // otherwise, ask for confirmation
            self.awaiting_leave_confirmation = !self.awaiting_leave_confirmation;
        }
    }

    fn leave(&mut self) {
        self.should_leave = true;
    }

    fn emergency_leave(&mut self) {
        self.should_emergency_leave = true;
    }
}
