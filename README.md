[![Check, Test and Build](https://github.com/smellyshovel/mine_rs/actions/workflows/check-test-and-build.yml/badge.svg?event=push)](https://github.com/smellyshovel/mine_rs/actions/workflows/check-test-and-build.yml)

# Mine RS

An in-terminal Minesweeper implementation in Rust.

![Gameplay Preview](https://raw.githubusercontent.com/smellyshovel/mine-rs/dev/.github/gameplay-preview.gif)

## Usage

1. Download the latest version for your OS from the [releases](https://github.com/smellyshovel/mine_rs/releases) page
2. Make the file executable, e.g., for Ubuntu
    ```
   chmod +x mine_rs-Ubuntu
   ```
3. Run the file as a regular executable program
    ```
   ./mine_rs-Ubuntu
   ```

## Features

All the game logic is written as a library, so that it could support multiple frontends. At the moment, there are two: `debug_main` (text-based) and `main` (TUI).