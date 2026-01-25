# pluck-list

An interactive terminal UI (TUI) tool to incrementally "pluck" lines from a newline-delimited list and produce derived lists via keyboard-only interaction.

## Features

- **Keyboard-only workflow**: Designed for speed and efficiency without mouse interaction.
- **Multiple Pluck Modes**:
  - **Top-down**: Pluck the first $N$ lines.
  - **Bottom-up**: Pluck the last $N$ lines.
  - **String match**: Filter lines using a string or regex with a live preview.
- **Split View**: Once lines are plucked, the UI splits into a "Modified List" (what remains) and a "New List" (what was plucked).
- **Incremental Operations**: Subsequent plucks always operate on the "Modified List".
- **Save/Save As**: Save your working set or your results to files.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
cargo build --release
```

## Usage

Provide a file or pipe multiline output to `pluck-list`:

```bash
# Using a file
./pluck-list data.txt

# Using piped output
ls -R | ./pluck-list
```

### Keybindings

- **`TAB` (⇥)**: Cycle active buffer (Prompt ↔ Modified List ↔ New List).
- **`Arrows` / `PageUp` / `PageDown`**: Navigate the active list buffer.
- **`Enter` (↵)**: Select a menu option or confirm input.
- **`Esc`**: Cancel current input or return to the main menu.
- **`s`**: Save the Modified List in-place (if the input was a file).
- **`S`**: Save the currently active list buffer (Modified or New) to a new path.
- **`q`**: Quit the application.
