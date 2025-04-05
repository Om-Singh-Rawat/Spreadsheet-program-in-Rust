use std::io::{self, Write};

struct Spreadsheet {
    grid: Vec<Vec<i32>>,
    rows: usize,
    cols: usize,
    view_top: usize,
    view_left: usize,
}

impl Spreadsheet {
    fn new(rows: usize, cols: usize) -> Spreadsheet {
        let grid = vec![vec![0; cols]; rows];
        Spreadsheet {
            grid,
            rows,
            cols,
            view_top: 0,
            view_left: 0,
        }
    }

    fn print_grid(&self) {
        for i in self.view_top..self.view_top + 10 {
            if i >= self.rows {
                break;
            }
            for j in self.view_left..self.view_left + 10 {
                if j >= self.cols {
                    break;
                }
                if j == 0 {
                    print!("{:<4}", i + 1);
                }
                print!("{:<4}", self.grid[i][j]);
            }
            println!();
        }
    }

    fn update_cell(&mut self, row: usize, col: usize, value: i32) {
        if row < self.rows && col < self.cols {
            self.grid[row][col] = value;
        }
    }

    fn scroll(&mut self, direction: &str) {
        match direction {
            "w" if self.view_top > 0 => self.view_top -= 1,
            "s" if self.view_top + 10 < self.rows => self.view_top += 1,
            "a" if self.view_left > 0 => self.view_left -= 1,
            "d" if self.view_left + 10 < self.cols => self.view_left += 1,
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: ./spreadsheet <rows> <cols>");
        return;
    }

    let rows: usize = args[1].parse().unwrap_or(10);
    let cols: usize = args[2].parse().unwrap_or(10);

    let mut spreadsheet = Spreadsheet::new(rows, cols);

    loop {
        spreadsheet.print_grid();

        println!("\nUse WASD to scroll, input 'q' to quit, or 'r c v' to update cell (row, col, value):");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "q" {
            break;
        }

        if input == "w" || input == "a" || input == "s" || input == "d" {
            spreadsheet.scroll(input);
        } else if let Some((row_str, rest)) = input.split_once(" ") {
            if let Some((col_str, value_str)) = rest.split_once(" ") {
                if let (Ok(row), Ok(col), Ok(value)) = (row_str.parse::<usize>(), col_str.parse::<usize>(), value_str.parse::<i32>()) {
                    spreadsheet.update_cell(row - 1, col - 1, value);  // Convert to 0-based index
                }
            }
        }
    }
}
