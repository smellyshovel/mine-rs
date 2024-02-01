//! The functionality related to the menu renderer.

use crate::app::AppMenu;
use crate::app::MenuItem::{ColumnsAmount, MinesAmount, RowsAmount};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Widget},
};

const LEGEND_TEXT: [&str; 5] = [
    "[↑][↓] / [w][s] / [i][j][k][l]: select the options",
    "[←][→] / [a][d] / [j][k]: decrement / increment the selected option's value",
    "[SPACE] / [ENTER]: start the game",
    "[f]: restore the selected option's default value",
    "[q] / [ESC]: leave",
];
const LEGEND_TEXT_COLOR: Color = Color::DarkGray;

pub fn render_menu(menu: &mut AppMenu, frame: &mut Frame) {
    // The root container is the whole terminal rectangle.
    let root_container = frame.size();

    // The app.rs layout consists of the menu, error and legend containers. The menu container's size is first calculated
    // as the remainder of the height after all the other allocations.
    let (menu_container, error_container, legend_container) = create_app_layout(&root_container);

    // Here menu gets shrank to some concrete dimensions.
    let (menu_container, menu_items_containers) = create_menu_layout(&menu_container, 3);

    // Now, as all the containers are ready (except for the popups' ones - those are generated on-demand), we can
    // actually render the parts of the application into them.

    // 1. Render the terminal background.
    frame.render_widget(Block::default().bg(Color::White), root_container);

    // Prepare the conditions for checking whether a menu item by some index is currently selected or not.
    let menu_items_rendering_conditions = [
        menu.selected_item == ColumnsAmount,
        menu.selected_item == RowsAmount,
        menu.selected_item == MinesAmount,
    ];

    // A closure to build a given menu item's style on the fly.
    let build_menu_item_style = |i| {
        Style::default()
            .bg(if menu_items_rendering_conditions[i] {
                Color::Yellow
            } else {
                Color::White
            })
            .fg(if menu_items_rendering_conditions[i] {
                Color::White
            } else {
                Color::Yellow
            })
    };

    // 2. Run through the list of menu items and render them all as paragraphs.
    vec![
        format!("\nWidth: < {} >", menu.columns_amount),
        format!("\nHeight: < {} >", menu.rows_amount),
        format!("\nMines: < {} >", menu.mines_amount),
    ]
    .into_iter()
    .enumerate()
    .for_each(|(i, item)| {
        frame.render_widget(
            Paragraph::new(item)
                .alignment(Alignment::Center)
                .style(build_menu_item_style(i)),
            menu_items_containers[i],
        )
    });

    // 2. Render the border around the menu.
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow)),
        menu_container,
    );

    // 3. Render the error (if any).
    if let Some(error) = &menu.error {
        frame.render_widget(
            Paragraph::new(format!("{:?}", error))
                .alignment(Alignment::Center)
                .red(),
            error_container,
        )
    }

    // 4. Render the legend.
    frame.render_widget(build_legend_widget(), legend_container);
}

/// The function build a layout for the application (this time, the menu). The layout of the menu is represented with
/// 3 rectangles: one for the menu itself (to hold the menu items), one for displaying a potential error messages and
/// one for the legend (the in-menu controls description).
fn create_app_layout(container: &Rect) -> (Rect, Rect, Rect) {
    // The error is always a one-liner, but we save some space for the padding (1 top and 1 bottom). So the total value
    // is 3: 1 (top padding) + 1 (text) + 1 (bottom padding).
    let error_container_height = 3;
    // The height of the legend is calculated based on the amount of lines in the legend text we need to display.
    let legend_container_height = LEGEND_TEXT.len() as u16;
    // The menu container's height is all that's left in the parental container.
    let menu_container_height = container.height - error_container_height - legend_container_height;

    // Create a vector of vertically-stacked rectangles with the pre-defined widths.
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_lengths([
            menu_container_height,
            error_container_height,
            legend_container_height,
        ]))
        .split(*container)
        .to_vec();

    // There's no need to horizontally split the menu container (to horizontally align it) because it's going to be
    // processed further and the menu is going to have a hard-coded width.
    let menu_container = vertical_layout[0];

    // For the error container we create a subgrid only to allow for a margin.
    let error_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([0, 100, 0]))
        .margin(1)
        .split(vertical_layout[1])[1];

    // The legend container is 90% of the width of the container and is horizontally-centered.
    let legend_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([5, 90, 5]))
        .split(vertical_layout[2])[1];

    (menu_container, error_container, legend_container)
}

fn create_menu_layout(container: &Rect, menu_items_amount: u16) -> (Rect, Vec<Rect>) {
    // The height for the menu is the number of menu items multiplied by one item's height (3) and plus 2 (because of 1
    // char padding top and bottom).
    let settings_container_height = 3 * menu_items_amount + 2;
    // This is purely a constant.
    let settings_container_width = 40;

    // Create a vertical grid to vertically center the menu items container.
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_lengths([
            (container.height - settings_container_height) / 2,
            settings_container_height,
            (container.height - settings_container_height) / 2,
        ]))
        .split(*container);

    // Divide the middle part of the vertical layout in such a manner to visually center the menu items container
    // horizontally.
    let menu_items_container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_lengths([
            (container.width - settings_container_width) / 2,
            settings_container_width,
            (container.width - settings_container_width) / 2,
        ]))
        .split(vertical_layout[1])[1];

    (
        // Return the menu items container...
        menu_items_container,
        // ...and separate sub-containers for each of the individual menu items.
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(Constraint::from_lengths([3, 3, 3]))
            .margin(1)
            .split(menu_items_container)
            .to_vec(),
    )
}

/// The function builds the ready-to-use legend block (some text that provides information about the in-menu controls).
fn build_legend_widget() -> impl Widget {
    let rows = LEGEND_TEXT.map(|legend_line| {
        let cells = legend_line.split_at(legend_line.find(':').expect("Couldn't find the delimiter character (`:`). Double-check the `LEGEND_TEXT` const's contents."));

        Row::new([
            Line::from(cells.0).alignment(Alignment::Right),
            Line::from(cells.1).alignment(Alignment::Left),
        ])
    });

    Table::new(rows, Constraint::from_percentages([50, 50])).fg(LEGEND_TEXT_COLOR)
}
