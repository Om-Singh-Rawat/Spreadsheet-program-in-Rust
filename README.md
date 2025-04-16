# Rust Spreadsheet

This is a terminal-based spreadsheet application written in Rust. It supports interactive cell editing, dependency-based recalculation, a set of built-in functions, and command-based navigation and control. The spreadsheet recalculates after every input, ensuring consistency across all dependent cells.

---

## Build and Usage

### Build Options

- **Build the binary executable**
  ```bash
  make
  ```
  This compiles the project and places the binary in `target/release/spreadsheet`.

- **Run tests**
  ```bash
  make test
  ```
  Executes test cases written in Rust under the `tests/` directory.

- **Generate PDF report**
  ```bash
  make report
  ```
  Compiles the LaTeX report located in `report/report.tex` into `report/report.pdf`.

### Running the Program

```bash
./target/release/spreadsheet <rows> <columns>
```

Launches the spreadsheet with the specified number of rows and columns.

---

## Features

### Cell Assignments

Cells can be assigned values directly or computed from other cells using basic arithmetic operations.

**Examples:**
```text
A1 = 5
B1 = A1 + 10
C1 = A1 * B1
```

- Only a **single operation** is allowed per assignment (e.g., `A1 = A2 + A3 + A4` is invalid).
- Right-hand-side values may be either integers or references to other cells.

---

### Built-in Functions

The spreadsheet supports a variety of built-in functions. All except `SLEEP` operate over **ranges of cells**:

**Supported functions:**
- `SUM(range)`
- `AVG(range)`
- `MIN(range)`
- `MAX(range)`
- `STDEV(range)`

**Examples:**
```text
A1 = SUM(B1:B5)
B2 = AVG(A1:A3)
C3 = MAX(D1:D4)
```

---

### Special Function: `SLEEP`

The `SLEEP` function is treated differently:

```text
A1 = SLEEP(5)
```

- `SLEEP(n)` halts execution for `n` seconds before updating the spreadsheet.
- It accepts a single argument, which may be a literal integer or a cell reference containing an integer.
- The result of `SLEEP` (the sleep duration) is assigned to the cell.
- During recalculation, `SLEEP` still enforces the delay if it is part of the dependency graph.

---

### Recalculation Engine

- After every user input, the spreadsheet performs a **full recalculation** of all affected cells.
- Cells are recomputed regardless of whether their final value changes.
- If an expression is invalid (e.g., due to syntax errors or circular dependencies), an error is returned.

---

### Interactive Commands

The spreadsheet interface supports the following interactive commands:

- `scroll_to <cell>` – Moves the cursor/view to the specified cell (e.g., `scroll_to B3`)
- `w` / `a` / `s` / `d` – Scrolls the view by 10 rows/columns up, left, down, or right, respectively
- `enable_output` – Enables printing of the spreadsheet after updates
- `disable_output` – Disables output printing
- `q` – Quits the interactive loop

The interface runs in a continuous loop, accepting commands and assignments until `q` is issued.

---

## Project Structure

```
.
├── src/              # Source code for the spreadsheet engine
├── tests/            # Rust test cases
├── report/           # LaTeX report and related files
├── Makefile          # Build automation
├── Cargo.toml        # Cargo configuration and dependencies
└── README.md         # Project documentation
```

---

## Notes

- All recalculations are done using tracked dependencies between cells.
- The system ensures deterministic updates and detects invalid expressions early.
- The interface is designed to be both scriptable and interactive, making it suitable for experimentation and testing.

---
