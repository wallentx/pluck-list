# pluck-list

**The interactive tool for stateful list partitioning.**

Standard CLI pipelines are stateless. This makes iterative partitioning (splitting a dataset into multiple specific subsets) unnecessarily complex. If you need to extract sequential batches from a list, tools like `head` and `tail` force you to manually calculate and manage offsets for every step. This process is brittle and tedious.

`pluck-list` solves this by treating your list as a mutable resource. When you "pluck" a selection, it is removed from the source. This automatically exposes the remainder for the next operation, turning a complex chain of math and temporary files into a linear workflow.

Use it to interactively filter and split text files, selecting lines via counts or regex and moving them into new buffers.

![pluck-list](https://github.com/user-attachments/assets/fdf04d81-c28a-4f11-8033-0f2d31f44db7)

## Features

- **TUI Interface**: A keyboard-driven workflow for navigating and selecting data.
- **Selection Modes**:
  - **Top-down**: Extract the first $N$ lines.
  - **Bottom-up**: Extract the last $N$ lines.
  - **String match**: Filter lines by string or regex with a live preview.
- **Dual Buffers**: Once lines are moved, the interface displays the "Modified List" (remaining lines) and the "New_List" (extracted lines) side-by-side.
- **Iterative Filtering**: Subsequent operations apply to the "Modified List," allowing for multi-step data extraction.
- **Exporting**: Save either buffer to a file with overwrite protection.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
cargo build --release
```

The binary will be available at `./target/release/pluck-list`.

## Usage

Provide a file path or pipe multiline output to `pluck-list`:

```bash
# Using a file
./target/release/pluck-list data.txt

# Using piped output
ls -R | ./target/release/pluck-list
```

### Keybindings

- **`TAB`**: Cycle the active buffer (Prompt ↔ Modified List ↔ New_List).
- **`Arrows` / `PageUp` / `PageDown`**: Navigate the active buffer.
- **`Enter`**: Confirm selection or input.
- **`Esc`**: Cancel input or return to the main menu.
- **`s`**: Save the Modified List in-place (if the input was a file).
- **`S`**: Save the active buffer to a new path.
- **`q`**: Quit the application.
