use clap::Parser;
use mine_rs::{field::Field, Minesweeper, MinesweeperAction, MinesweeperStatus};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long)]
    height: Option<u8>,
    #[arg(short, long)]
    width: Option<u8>,
    #[arg(short, long)]
    mines: Option<u16>,
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let mut game = Minesweeper::new(
        args.height.unwrap_or(5),
        args.width.unwrap_or(5),
        args.mines.unwrap_or(5),
    )
    .expect("Couldn't create a game instance!");

    print_field(game.get_field(), args.debug);

    loop {
        let Some(action) = get_action() else {
            println!("Incorrect input! Please, try again.");
            continue;
        };

        if let MinesweeperStatus::End(result) = game
            .take_action(action)
            .expect("Couldn't take the specified action! Fatal error.")
        {
            let human_readable_result = if *result { "VICTORY" } else { "LOSS" };

            print_field(game.get_field(), args.debug);
            println!("{human_readable_result}");

            break;
        }

        print_field(game.get_field(), args.debug);
    }
}

fn print_field(field: &Field, debug: bool) {
    if debug {
        println!("DEBUG:\n{:?}", field);
    } else {
        println!("DISPLAY:\n{}", field);
    }
}

fn get_action() -> Option<MinesweeperAction> {
    println!("Enter the desired action and the target cell's coordinates (e.g. `f 3,5` to flag the 6th cell on the 4th\
    line. Other actions include `o` to open a cell and `s` to open the cell's surrounding cells):");

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
