# Rust Spreadsheet

This is a terminal-based spreadsheet application written in Rust. It supports interactive cell editing, dependency-based recalculation, a set of built-in functions, and command-based navigation and control. The spreadsheet recalculates after every input, ensuring consistency across all dependent cells.

---

## Dependencies

To run this project and its web-based extension, you'll need the following:

### 🧱 Core (Terminal-Based Spreadsheet)
- `cargo` and `rustc` (Rust toolchain)
- No external crates required

Entirely built with Rust's standard library (std).
### 🌐 Web Extension (`spreadsheet_ui/`)
- `wasm-pack` (Install via `cargo install wasm-pack`)
- `wasm-bindgen`
- `yew`
- `gloo`
- `web-sys`
- `reqwest`
- `tokio`
- `serde`, `serde_json`, `serde_derive`
- `rocket` (for backend)
- `rocket_cors` (for CORS support)

---

## Build and Usage

### Build Options

- **Build the binary executable**
  ```bash
  make
  ```
  This compiles the project and places the binary in `target/release/spreadsheet`.

- **Run the application**
  ```bash
  make run
  ```
  Builds the project (if necessary) and runs it with sample arguments (`999 18278`).

- **Run tests**
  ```bash
  make test
  ```
  Executes test cases written in Rust under the `tests/` directory.

- **Check formatting and lint code**
  ```bash
  make lint
  ```
  Runs `cargo fmt` and `cargo clippy` to check formatting and enforce lint rules. Fails on any warnings.

- **Generate code coverage report**
  ```bash
  make coverage
  ```
  Runs unit tests using `cargo tarpaulin` to produce an HTML coverage report. Automatically opens the report.

- **Generate PDF report and rustdoc documentation**
  ```bash
  make docs
  ```
  Compiles the LaTeX report from `report/report.tex` into `report/report.pdf` and copies it to the root as `report.pdf`. Generates the 'index.html' file and opens in browser using the in-built rust command 'cargo docs --open'.

- **Clean build artifacts and reports**
  ```bash
  make clean
  ```
  Removes build artifacts, LaTeX auxiliary files, generated reports, and the coverage HTML report.
<<<<<<< HEAD

---

### Web-Based Extension (Browser Spreadsheet UI)

This project also features a web-based spreadsheet extension located in the `spreadsheet_ui/` directory. It consists of:

- A **frontend** written in Rust using [Yew](https://yew.rs), compiled to WebAssembly using `wasm-pack`.
- A **backend server** written in Rust using [Rocket](https://rocket.rs) to handle login/signup and spreadsheet synchronization.

#### 💻 Run the Extension

```bash
make ext1
```

This builds the frontend and starts both servers **simultaneously**:

- The backend Rocket server runs on `http://localhost:8000`
- The frontend is served from `http://localhost:8080`

> Note: The frontend build output is placed in `spreadsheet_ui/server/static` and served by a static file server (Python) during development.

#### 🧩 Features of the Extension

- **User Login/Signup** with credential storage (currently local, pluggable to a DB)
- **Each user has their own spreadsheet** (persisted on the backend)
- **Cell dependencies and values are preserved** just like in the terminal app
- **Frontend UI** displays the spreadsheet, cell focus, and supports navigation
- **Color-coded dependencies**
- **Cyclic dependency detection**
- **Only integers currently supported in GUI view**
- **Grid limited to 20x10 due to scaling constraints**

> Future work may include collaborative editing, image-based CSV import via ChatGPT API, and better type support.

---

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
├── spreadsheet_ui/   # Front and back end files for the extension 
|   ├── client/       # Yew-based frontend (compiled to WebAssembly)
|   ├── core/         # Shared logic for parsing and evaluation
|   ├── server/       # Rocket-based backend with auth and persistence
|   └── Cargo.toml
├── src/              # Source code for the spreadsheet along with unit and integration tests.
├── report/           # LaTeX report and related files
├── Makefile
├── Cargo.toml
└── README.md
```

---

## Notes

- All recalculations are done using tracked dependencies between cells.
- The system ensures deterministic updates and detects invalid expressions early.
- The interface is designed to be both scriptable and interactive, making it suitable for experimentation and testing.

---

![Lint](https://github.com/Skyblock127/Rust-Assignment_COP290/actions/workflows/lint.yml/badge.svg)
![Tests](https://github.com/Skyblock127/Rust-Assignment_COP290/actions/workflows/test.yml/badge.svg)
![Coverage](https://github.com/Skyblock127/Rust-Assignment_COP290/actions/workflows/coverage.yml/badge.svg)
```

---
