//! The game renderer functions.

use crate::app::AppGame;
use mine_rs::{field::cell::Cell, MinesweeperStatus};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, Widget},
};
use std::cmp;

/// The symbols used as the arrows indicating that there are more cells to the top/left/bottom/right (respectively).
/// This is used when the field is too big to be fully displayed in the terminal.
///
/// The aforementioned order is preserved everywhere in the code.
const ARROW_SYMBOLS: [&str; 4] = ["↑", "←", "↓", "→"];
/// The size (height for up and down, width for left and right) of a single arrow.
/// This is used for the layouts' sizes calculations.
const ARROW_SYMBOL_SIZE: u8 = 1;
/// The number of terminal rows that a single cell occupies (including the margins/paddings/borders if any).
const CELL_HEIGHT: u8 = 3;
/// The number of terminal columns that a single cell occupies (including the margins/paddings/borders if any).
const CELL_WIDTH: u8 = 5;

const CLOSED_CELL_SYMBOL: &str = "███";
const FLAG_SYMBOL: &str = " 🚩 ";
const MINE_SYMBOL: &str = " 💣 ";
const WRONG_CHOICE_SYMBOL: &str = " ❌ ";
const CLOCK_SYMBOL: &str = " 🕓 ";
const CELL_COLOR: Color = Color::Yellow;
const CELL_PALE_COLOR: Color = Color::LightYellow;
const APP_BG_COLOR: Color = Color::White;
const FIELD_BORDER_COLOR: Color = Color::Yellow;
const FIELD_BORDER_PALE_COLOR: Color = Color::LightYellow;
const PAUSED_GAME_POPUP_BORDER_COLOR: Color = Color::LightYellow;
const OUTCOME_POPUP_VICTORY_BORDER_COLOR: Color = Color::Green;
const OUTCOME_POPUP_LOSS_BORDER_COLOR: Color = Color::Red;
const LEAVE_CONFIRMATION_POPUP_BORDER_COLOR: Color = Color::Red;
const INFO_WIDGET_BLOCK_COLOR: Color = Color::LightYellow;
const REGULAR_TEXT_COLOR: Color = Color::Black;
const LEGEND_TEXT_COLOR: Color = Color::DarkGray;

const LEGEND_TEXT: [&str; 5] = [
    "[↑][←][↓][→] / [w][a][s][d] / [i][j][k][l]: move the cursor",
    "[SPACE] / [ENTER]: open the selected cell (or surrounding cells)",
    "[f]: toggle flag for the selected cell",
    "[p]: pause the game",
    "[q] / [ESC]: leave",
];
const PAUSED_GAME_POPUP_TEXT: [&str; 3] = ["Paused", "", "(Press [p] to continue)"];
const VICTORY_LINE_TEXT: &str = "You won! Congratulations!";
const LOSS_LINE_TEXT: &str = "You lost... Wanna try again?";
const OUTCOME_POPUP_TEXT: [&str; 4] = [
    "",
    "Use:",
    "[SPACE] / [ENTER] to start a new game",
    "[q] / [ESC] to leave back to the menu",
];
const LEAVE_CONFIRMATION_POPUP_TEXT: [&str; 6] = [
    "Are you sure you want to quit?",
    "The progress shall not be saved!",
    "",
    "Use:",
    "[SPACE] / [ENTER] - CONFIRM",
    "[q] / [ESC] - CANCEL",
];

pub fn render_game(app: &mut AppGame, frame: &mut Frame) {
    // the root container is the whole terminal rectangle
    let root_container = frame.size();

    // the app.rs layout consists of the field, stats and legend containers.
    // The stats are represented by the flags-, mines- and time-info containers.
    let (
        field_container,
        (flags_info_container, mines_info_container, time_info_container),
        legend_container,
    ) = create_app_layout(&root_container);

    // the amounts of rows and columns we need to show totally (the real field size)
    let (total_rows_amount, total_columns_amount, _) = app.game.get_field().get_size();

    // update the amounts of rows and columns that we can actually show (respecting the container's size)
    app.visible_rows_amount = calculate_visible_rows_amount(&field_container, total_rows_amount);
    app.visible_columns_amount =
        calculate_visible_columns_amount(&field_container, total_columns_amount);

    // the field layout consists of the grid and 4 arrows' (up, left, down and right) containers
    let (grid_container, arrow_containers) = create_field_layout(
        &field_container,
        app.visible_rows_amount as u16,
        app.visible_columns_amount as u16,
    );

    // adjust the arrow symbols for the proper alignment and declare the default alignment settings for the arrows
    let arrow_symbols = adjust_arrow_symbols(&field_container, arrow_containers);
    let arrow_alignments = [
        Alignment::Center,
        Alignment::Left,
        Alignment::Center,
        Alignment::Right,
    ];

    // the grid layout is essentially a 2D vector of cells
    let grid = build_grid_layout(
        &grid_container,
        app.visible_rows_amount,
        app.visible_columns_amount,
    );

    // Now, as all the containers are ready (except for the popups' ones - those are generated on-demand), we can
    // actually render the parts of the application into them.

    // 1. Render the terminal background
    frame.render_widget(Block::default().bg(APP_BG_COLOR), root_container);

    // 2. Render the border around the field
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(FIELD_BORDER_COLOR)),
        field_container,
    );

    // 3. Render the arrows above the field border
    [
        app.window_offset.0 != 0,
        app.window_offset.1 != 0,
        app.window_offset.0 != total_rows_amount - app.visible_rows_amount,
        app.window_offset.1 != total_columns_amount - app.visible_columns_amount,
    ]
    .iter()
    .enumerate()
    .for_each(|(i, rendering_condition)| {
        if *rendering_condition {
            frame.render_widget(
                Paragraph::new(arrow_symbols[i].clone())
                    .alignment(arrow_alignments[i])
                    .fg(FIELD_BORDER_PALE_COLOR),
                arrow_containers[i],
            );
        }
    });

    // 4. Render the paused game popup if the game is paused or otherwise the cells
    if let MinesweeperStatus::Pause = app.game.get_status() {
        // 4.a.1. Render an empty block in place of the grid
        frame.render_widget(Block::default().bg(APP_BG_COLOR), grid_container);

        // 4.a.2. Render the paused game popup
        render_popup(
            frame,
            PAUSED_GAME_POPUP_TEXT.map(|line| line.to_string()),
            PAUSED_GAME_POPUP_BORDER_COLOR,
        );
    } else {
        // 4.b.1. Render a grid of the cells
        grid.iter().enumerate().for_each(|(row_index, row)| {
            row.iter()
                .enumerate()
                .for_each(|(column_index, cell_container)| {
                    // the real indices are those including the window offset
                    let real_row_index = row_index as u8 + app.window_offset.0;
                    let real_column_index = column_index as u8 + app.window_offset.1;

                    let cell = app
                        .game
                        .get_field()
                        .get_cell((real_row_index, real_column_index))
                        .expect("Fatal error: couldn't find the cell by its coordinates.");

                    let is_selected = app.cursor_position == (real_row_index, real_column_index);

                    let grid_cell = build_cell_widget(
                        cell,
                        is_selected,
                        app.game.get_status() == &MinesweeperStatus::End(false),
                    );
                    frame.render_widget(grid_cell, *cell_container)
                });
        });
    }

    // 5. Render the stats
    frame.render_widget(
        build_flags_info_widget(app.game.get_field().get_flagged_cells_amount()),
        flags_info_container,
    );
    frame.render_widget(
        build_mines_info_widget(app.game.get_field().get_mines_amount()),
        mines_info_container,
    );

    frame.render_widget(
        build_time_info_widget(format_duration(app.game.get_time())),
        time_info_container,
    );

    // 6. Render the legend
    frame.render_widget(build_legend_widget(), legend_container);

    // 7. Render the outcome (victory/loss) popup in case the game has ended
    if let MinesweeperStatus::End(is_victory) = app.game.get_status() {
        let first_line = if *is_victory {
            VICTORY_LINE_TEXT
        } else {
            LOSS_LINE_TEXT
        };

        let rest_lines = OUTCOME_POPUP_TEXT;

        let lines: Vec<_> = [first_line]
            .iter()
            .chain(rest_lines.iter())
            .map(|s| s.to_string())
            .collect();

        let border_color = if *is_victory {
            OUTCOME_POPUP_VICTORY_BORDER_COLOR
        } else {
            OUTCOME_POPUP_LOSS_BORDER_COLOR
        };

        render_popup(frame, lines, border_color);
    }

    // 8. Render the leave confirmation popup in case the leave has been requested
    if app.awaiting_leave_confirmation {
        render_popup(
            frame,
            LEAVE_CONFIRMATION_POPUP_TEXT.map(|line| line.to_string()),
            LEAVE_CONFIRMATION_POPUP_BORDER_COLOR,
        );
    }
}

/// The method creates the base grid needed for the application. Namely, we need to show the field, some statistics for
/// the ongoing game and the controls-legend.
fn create_app_layout(container: &Rect) -> (Rect, (Rect, Rect, Rect), Rect) {
    // the stats container's height is 3 rows: 2 for borders and one for the contents
    let stats_container_height = 3;
    // the legend container's height is 4 rows (for the controls-related information)
    let legend_container_height = LEGEND_TEXT.len() as u16;
    // the field container's height is all that's left
    let field_container_height =
        container.height - stats_container_height - legend_container_height;

    // create a set of vertically-stacked rectangles
    let app_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_lengths([
            field_container_height,
            stats_container_height,
            legend_container_height,
        ]))
        .split(*container)
        .to_vec();

    // split the top rectangle into 3: 2 margins and a central one for the grid
    let field_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([10, 80, 10]))
        .split(app_layout[0])[1];

    // the middle rectangle is also split into 3 ones
    let stats_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([25, 50, 25]))
        .split(app_layout[1])[1];

    // the central one from the above is split into 3 equal sections once again (for the 3 stats-items)
    let flags_mines_and_time_containers = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([33, 34, 33]))
        .split(stats_container)
        .to_vec();

    let (flags_info_container, mines_info_container, time_info_container) = (
        flags_mines_and_time_containers[0],
        flags_mines_and_time_containers[1],
        flags_mines_and_time_containers[2],
    );

    // the bottom rectangle is split the same fashion as the top one
    let legend_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([5, 90, 5]))
        .split(app_layout[2])[1];

    (
        field_container,
        (
            flags_info_container,
            mines_info_container,
            time_info_container,
        ),
        legend_container,
    )
}

/// Ideally, we'd like to show all the rows of the field. But this is often impossible to do, since the total number
/// of the rows is more than the space available vertically to render all these rows. Therefore, in such cases,
/// we divide the total available space by the height of a single row to find out how many full rows would fit into
/// the container.
fn calculate_visible_rows_amount(field_container: &Rect, total_rows_amount: u8) -> u8 {
    // the height needed to render the field including the space allocated for the arrows
    let height_needed = (CELL_HEIGHT * total_rows_amount + (ARROW_SYMBOL_SIZE * 2)) as u32;

    // if the total height needed to render the field is less than or equal to the height of the container
    if height_needed <= field_container.height as u32 {
        // then allow to render all the rows of the field
        total_rows_amount
    } else {
        // otherwise, the amount of rows to render is calculated based on how many rows could potentially fit into the
        // available container's height subtracting the space allocated for the arrows
        ((field_container.height - (ARROW_SYMBOL_SIZE as u16) * 2) / (CELL_HEIGHT as u16)) as u8
    }
}

/// Ideally, we'd like to show all the columns of the field. But this is often impossible to do, since the total number
/// of the columns is more than the space available horizontally to render all these columns. Therefore, in such cases,
/// we divide the total available space by the width of a single column to find out how many full columns would fit into
/// the container.
fn calculate_visible_columns_amount(field_container: &Rect, total_columns_amount: u8) -> u8 {
    // the width needed to render the field including the space allocated for the arrows
    let width_needed = (CELL_WIDTH * total_columns_amount + (ARROW_SYMBOL_SIZE * 2)) as u32;

    // if the total width needed to render the field is less than or equal to the width of the container
    if width_needed <= field_container.width as u32 {
        // then allow to render all the columns of the field
        total_columns_amount
    } else {
        // otherwise, the amount of columns to render is calculated based on how many columns could potentially fit
        // into the available container's width subtracting the space allocated for the arrows
        ((field_container.width - (ARROW_SYMBOL_SIZE as u16) * 2) / (CELL_WIDTH as u16)) as u8
    }
}

/// This method produces a 3*3 grid, there the central rectangle will contain the cells grid, and the ones on the sides
/// will hold the arrows which are shown in cases when the field is too large to fully fit into the central rectangle.
///
/// The dimensions of the central grid-for rectangle are strictly fixed and are divisible without remainders by the
/// visible rows/cells amounts. This is necessary in order to avoid rendering incomplete or stretched cells.
///
/// The remainder of division of the total field container's size by the amount of visible rows/columns of the grid is
/// spread equally by the side-containers allocated for the arrows (these also serve as margins/paddings between the
/// field's border and the cells grid).
fn create_field_layout(
    game_container: &Rect,
    visible_rows_amount: u16,
    visible_columns_amount: u16,
) -> (Rect, [Rect; 4]) {
    // find the height and width needed to render the required amount of rows and columns (not including the arrows)
    let (height_for_rows, width_for_columns) = (
        visible_rows_amount * CELL_HEIGHT as u16,
        visible_columns_amount * CELL_WIDTH as u16,
    );

    // for the central rectangle allocate exactly as much space as needed to fit all the visible rows. Split the
    // remainder of the space equally between the upper and bottom arrows' containers
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(cmp::max(
                ARROW_SYMBOL_SIZE as u16,
                (game_container.height - height_for_rows) / 2,
            )),
            Constraint::Length(height_for_rows),
            Constraint::Length(cmp::max(
                ARROW_SYMBOL_SIZE as u16,
                (game_container.height - height_for_rows) / 2,
            )),
        ])
        .split(*game_container);

    // same fashion horizontally. Make the central rectangle exactly the number of columns needed to render the visible
    // columns and split the remainder of the space equally between the left/right arrows' containers
    let field_layout = vertical_layout
        .iter()
        .map(|row| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(cmp::max(
                        ARROW_SYMBOL_SIZE as u16,
                        (game_container.width - width_for_columns) / 2,
                    )),
                    Constraint::Length(width_for_columns),
                    Constraint::Length(cmp::max(
                        ARROW_SYMBOL_SIZE as u16,
                        (game_container.width - width_for_columns) / 2,
                    )),
                ])
                .split(*row)
                .to_vec()
        })
        .collect::<Vec<_>>();

    // as we don't care about the corner-containers and won't actually use them, return the main (central) rectangle
    // that's gonna be used as a grid and the side-containers for the arrows in the predefined order
    (
        field_layout[1][1], // for the grid
        [
            // for the arrows
            field_layout[0][1],
            field_layout[1][0],
            field_layout[2][1],
            field_layout[1][2],
        ],
    )
}

/// The arrow-symbols need adjustment before they could be rendered on the field. Namely, we need to align them with the
/// field's border (to render them on top of the border) and align the left and right arrows vertically, so that they
/// appear in the middle of the field.
///
/// The horizontal alignment for the top/bottom arrows is done at the render-time by the ratatui's built-in methods.
/// However, ratatui doesn't know how to align things vertically. So, in order to do that, we pad the left and right
/// arrows with the "new line" character as many times, as is the height of the field divided by half (and minus one for
/// the height of the arrow itself).
///
/// Now, as all the arrows are centered regarding their corresponding segments of the border, there's still a small
/// issue: the standard alignment is top-left, so the top and left arrows are located right above the field's border,
/// but the down and right arrows are probably not.
///
/// In order to fix that, we get the height/width of the bottom/right arrow containers and pad the arrows with
/// new lines / spaces, so that the arrows are located on their most-remote-from-the-center positions, and thus pushed
/// to be rendered above the field's border.
fn adjust_arrow_symbols(field_container: &Rect, arrow_containers: [Rect; 4]) -> [String; 4] {
    let mut arrow_symbols = ARROW_SYMBOLS.map(|s| s.to_string());

    // arrow left: pad with new lines for it to be vertically in the middle
    arrow_symbols[1] = format!(
        "{}{}",
        "\n".repeat((field_container.height / 2 - 1) as usize),
        arrow_symbols[1]
    );

    // arrow down: pad with new lines until it's on the last line (and thus equal with the field's border)
    arrow_symbols[2] = format!(
        "{}{}",
        "\n".repeat((arrow_containers[2].height - ARROW_SYMBOL_SIZE as u16) as usize),
        arrow_symbols[2]
    );

    // arrow right: pad with new spaces until it's on the last column (and thus equal with the field's border)
    arrow_symbols[3] = format!(
        "{}{}",
        " ".repeat((arrow_containers[3].width - ARROW_SYMBOL_SIZE as u16) as usize),
        arrow_symbols[3]
    );

    // arrow right: pad with new lines for it to be vertically in the middle
    arrow_symbols[3] = format!(
        "{}{}",
        "\n".repeat((field_container.height / 2 - 1) as usize),
        arrow_symbols[3]
    );

    arrow_symbols
}

/// The grid layout is what's used to display the cells of the field.
///
/// The container is first divided into equal rows, and then each row is divided into equal cells.
fn build_grid_layout(container: &Rect, rows_amount: u8, columns_amount: u8) -> Vec<Vec<Rect>> {
    // divide the space vertically into rows
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..rows_amount).map(|_| Constraint::Length(CELL_HEIGHT.into())))
        .split(*container);

    // divide each row horizontally into cells
    vertical_layout
        .iter()
        .map(|row| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints((0..columns_amount).map(|_| Constraint::Length(CELL_WIDTH.into())))
                .split(*row)
                .to_vec()
        })
        .collect::<Vec<_>>()
}

/// Build a popup with the provided contents (lines of a text), set to it the provided border color and render it in the
/// center of a given region.
///
/// The size calculation for the popup is performed based on the content's size: the width of the popup would always be
/// the same as the width of the text's longest line and the popup's height would always be the number of the lines of
/// the text.
fn render_popup(frame: &mut Frame, lines: impl IntoIterator<Item = String>, border_color: Color) {
    // collect the lines of the text into a vector of `String`s and remember the lines' amount
    let lines: Vec<String> = lines.into_iter().collect();
    let lines_amount = lines.len() as u16;

    // create a block that would be used as the popup's backdrop
    let block = Block::default()
        .bg(APP_BG_COLOR)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    // prepare the text: join the lines with the new line symbol, put the final text into a paragraph and center it
    let text = Paragraph::new(lines.join("\n"))
        .fg(REGULAR_TEXT_COLOR)
        .alignment(Alignment::Center)
        .block(block);

    // determine the height of the popup and the remaining height of the container
    let root = frame.size();
    let popup_height = lines_amount + 2;
    let remainder_height = root.height - popup_height;

    // create a vertical layout to vertically center the popup
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_lengths([
            remainder_height / 2,
            popup_height,
            remainder_height / 2,
        ]))
        .split(root);

    // determine the width of the popup and the remaining horizontal space of the container
    let popup_width = lines.iter().map(|m| m.len()).max().unwrap() as u16 + 2;
    let remainder_width = root.width - popup_width;

    // create a horizontal layout to horizontally center the popup. Take the central part of it to the widget there
    let container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_lengths([
            remainder_width / 2,
            popup_width,
            remainder_width / 2,
        ]))
        .split(popup_layout[1])[1];

    // clear the region so that it doesn't contain any old graphics and render the widget in the prepared region
    frame.render_widget(Clear, container);
    frame.render_widget(text, container);
}

/// The function builds a widget (basically, a paragraph) that represents a single cell.
///
/// The function takes as input the library-representation of the cell and a flag which suggests whether the cell is
/// currently selected or not. Based on that information, it decides what text to render and which colors to use.
fn build_cell_widget(cell: &Cell, selected: bool, game_lost: bool) -> impl Widget {
    let symbol = if game_lost && cell.is_flagged() && !cell.is_mined() {
        WRONG_CHOICE_SYMBOL.to_string()
    } else if !cell.is_open() && !cell.is_flagged() {
        CLOSED_CELL_SYMBOL.to_string()
    } else if cell.is_flagged() {
        FLAG_SYMBOL.to_string()
    } else if let Some(adjacent_mines_amount) = cell.get_mines_around_amount() {
        if adjacent_mines_amount == 0 {
            "   ".to_string()
        } else {
            format!(" {adjacent_mines_amount} ")
        }
    } else {
        MINE_SYMBOL.to_string()
    };

    let color = if selected {
        CELL_COLOR
    } else {
        CELL_PALE_COLOR
    };

    // the cell stying
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(color));

    Paragraph::new(symbol).fg(color).block(block)
}

/// Prepares a paragraph to render as an info-block showing the number of flags placed on the field.
fn build_flags_info_widget(flags_amount: u16) -> impl Widget {
    Paragraph::new(flags_amount.to_string())
        .fg(REGULAR_TEXT_COLOR)
        .alignment(Alignment::Center)
        .block(build_info_widget_block(FLAG_SYMBOL.trim()))
}

/// Prepares a paragraph to render as an info-block showing the total number of mines hidden in the field.
fn build_mines_info_widget(mines_amount: u16) -> impl Widget {
    Paragraph::new(mines_amount.to_string())
        .fg(REGULAR_TEXT_COLOR)
        .alignment(Alignment::Center)
        .block(build_info_widget_block(MINE_SYMBOL.trim()))
}

/// Prepares a paragraph to render as an info-block showing the time it took from the beginning of the game.
fn build_time_info_widget(formatted_time: String) -> impl Widget {
    Paragraph::new(formatted_time)
        .fg(REGULAR_TEXT_COLOR)
        .alignment(Alignment::Center)
        .block(build_info_widget_block(CLOCK_SYMBOL.trim()))
}

/// A dependency of the 3 methods above (`build_flags_info_widget`, `build_mines_info_widget` and
/// `build_time_info_widget`) which creates a block used to display all info-blocks.
fn build_info_widget_block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(INFO_WIDGET_BLOCK_COLOR))
}

/// Formats the duration of the game in seconds as `MM:SS`.
fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

/// The function builds the ready-to-use legend block (some text that provides information about the in-game controls).
fn build_legend_widget() -> impl Widget {
    let rows = LEGEND_TEXT.map(|legend_row| {
        let cells = legend_row.split_at(legend_row.find(':').expect("Couldn't find the delimiter character (`:`). Double-check the `LEGEND_TEXT` const's contents."));

        Row::new([
            Line::from(cells.0).alignment(Alignment::Right),
            Line::from(cells.1).alignment(Alignment::Left),
        ])
    });

    Table::new(rows, Constraint::from_percentages([50, 50])).fg(LEGEND_TEXT_COLOR)
}
