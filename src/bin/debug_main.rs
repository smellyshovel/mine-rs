use mine_rs::{field::Field, Minesweeper, MinesweeperAction, MinesweeperStatus};

fn get_action() -> Option<MinesweeperAction> {
    println!("Enter the desired action and the target cell's coordinates (e.g. `f 3,5` to flag the 6th cell on the 4th line):");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok()?;

    let [action, cell_position]: [&str; 2] = input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .as_slice()
        .try_into()
        .ok()?;

    let cell_position = cell_position
        .trim()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect::<Vec<u8>>();

    let cell_position = (*cell_position.first()?, *cell_position.get(1)?);

    match action {
        "o" => Some(MinesweeperAction::OpenCell(cell_position)),
        "s" => Some(MinesweeperAction::OpenSurroundingCells(cell_position)),
        "f" => Some(MinesweeperAction::FlagCell(cell_position)),
        _ => None,
    }
}

fn print_field(field: &Field) {
    println!("DISPLAY:\n{}", field);
    // println!("DEBUG:\n{:?}", field);
}

fn main() {
    let mut game = Minesweeper::new(9, 24, 18).expect("Couldn't create a game instance!");

    print_field(&game.get_field());

    loop {
        let Some(action) = get_action() else {
            println!("Incorrect input! Please, try again.");
            continue;
        };

        if let MinesweeperStatus::End(result) = game
            .take_action(action)
            .expect("Couldn't take the specified action! Fatal error.")
        {
            println!("{:?}", result);
            break;
        }

        print_field(&game.get_field());
    }
}
