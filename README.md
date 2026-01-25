# pluck-list

**The visual, interactive `mv` command for text lists.**

`pluck-list` turns the tedious task of splitting and filtering text files into a fast, keyboard-driven workflow.

Imagine you have a massive log file, a list of cloud resources, or a queue of messy data. You don't just want to *view* it; you want to **extract** parts of it. Instead of juggling `grep`, `head`, and temporary files, `pluck-list` lets you interactively "pluck" lines out of your main list and move them into a new one.

Whether you're grabbing the top 100 lines for a test set, or using a regex to surgically remove specific patterns, you get live visual feedback on exactly what you're changing. It is designed to help you **whittle down** noise to find the signal.

## Features

- **Keyboard-only workflow**: Designed for speed and efficiency without mouse interaction.
- **Multiple Pluck Modes**:
  - **Top-down**: Pluck the first $N$ lines.
  - **Bottom-up**: Pluck the last $N$ lines.
  - **String match**: Filter lines using a string or regex with a live preview.
- **Split View**: Once lines are plucked, the UI splits into a "Modified List" (what remains) and a "New List" (what was plucked).
- **Incremental Operations**: Subsequent plucks always operate on the "Modified List", allowing you to peel away layers of data.
- **Save/Save As**: Save your working set or your results to files with overwrite protection.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
cargo build --release
```

The binary will be available at `./target/release/pluck-list`.

## Usage

Provide a file or pipe multiline output to `pluck-list`:

```bash
# Using a file
./target/release/pluck-list data.txt

# Using piped output
ls -R | ./target/release/pluck-list
```

### Keybindings

- **`TAB` (⇥)**: Cycle active buffer (Prompt ↔ Modified List ↔ New List).
- **`Arrows` / `PageUp` / `PageDown`**: Navigate the active list buffer.
- **`Enter` (↵)**: Select a menu option or confirm input.
- **`Esc`**: Cancel current input or return to the main menu.
- **`s`**: Save the Modified List in-place (if the input was a file).
- **`S`**: Save the currently active list buffer (Modified or New) to a new path.
- **`q`**: Quit the application.