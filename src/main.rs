use std::io::Write;

struct Spreadsheet {
    grid: Vec<Vec<i32>>,
    rows: usize,
    cols: usize,
    view_top: usize,
    view_left: usize,
}

impl Spreadsheet {
    fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![0; cols]; rows];
        Self {
            grid,
            rows,
            cols,
            view_top: 0,
            view_left: 0,
        }
    }

    fn column_index_to_label(mut index: usize) -> String {
        let mut label = String::new();
        index += 1;
        while index > 0 {
            let rem = (index - 1) % 26;
            label.insert(0, (b'A' + rem as u8) as char);
            index = (index - 1) / 26;
        }
        label
    }

    fn column_label_to_index(label: &str) -> Option<usize> {
        let mut index = 0;
        for c in label.chars() {
            if !c.is_ascii_uppercase() {
                return None;
            }
            index = index * 26 + (c as usize - 'A' as usize + 1);
        }
        Some(index - 1)
    }

    fn print_grid(&self) {
        print!("{:<4}", "");
        for j in self.view_left..(self.view_left + 10).min(self.cols) {
            print!("{:<4}", Self::column_index_to_label(j));
        }
        println!();

        for i in self.view_top..(self.view_top + 10).min(self.rows) {
            print!("{:<4}", i + 1);
            for j in self.view_left..(self.view_left + 10).min(self.cols) {
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

    fn get_cell_value(&self, row: usize, col: usize) -> Option<i32> {
        if row < self.rows && col < self.cols {
            Some(self.grid[row][col])
        } else {
            None
        }
    }

    fn parse_cell_reference(cell: &str) -> Option<(usize, usize)> {
        let split_index = cell.find(|c: char| c.is_ascii_digit())?;
        let (col_part, row_part) = cell.split_at(split_index);
        let col = Self::column_label_to_index(col_part)?;
        let row = row_part.parse::<usize>().ok()? - 1;
        Some((row, col))
    }

    fn handle_assignment(&mut self, input: &str) {
        if let Some((left, expr)) = input.split_once('=') {
            let left = left.trim();
            if let Some((target_row, target_col)) = Self::parse_cell_reference(left) {
                let expr = expr.trim().replace(" ", "");
                let operator = expr.chars().find(|c| "+-*/".contains(*c));
                if let Some(op) = operator {
                    let parts: Vec<&str> = expr.split(op).collect();
                    if parts.len() != 2 {
                        println!("Invalid expression format.");
                        return;
                    }

                    let get_value = |s: &str| {
                        if let Ok(val) = s.parse::<i32>() {
                            Some(val)
                        } else if let Some((r, c)) = Self::parse_cell_reference(s) {
                            self.get_cell_value(r, c)
                        } else {
                            None
                        }
                    };

                    if let (Some(val1), Some(val2)) = (get_value(parts[0]), get_value(parts[1])) {
                        let result = match op {
                            '+' => val1 + val2,
                            '-' => val1 - val2,
                            '*' => val1 * val2,
                            '/' => {
                                if val2 == 0 {
                                    println!("Division by zero.");
                                    return;
                                } else {
                                    val1 / val2
                                }
                            }
                            _ => return,
                        };
                        self.update_cell(target_row, target_col, result);
                    } else {
                        println!("Invalid values in expression.");
                    }
                } else if let Ok(val) = expr.parse::<i32>() {
                    self.update_cell(target_row, target_col, val);
                } else if let Some((r, c)) = Self::parse_cell_reference(&expr) {
                    if let Some(val) = self.get_cell_value(r, c) {
                        self.update_cell(target_row, target_col, val);
                    } else {
                        println!("Invalid cell reference.");
                    }
                } else {
                    println!("Invalid assignment.");
                }
            }
        }
    }

    fn scroll(&mut self, direction: &str) {
        match direction {
            "w" => {
                if self.view_top >= 10 {
                    self.view_top -= 10;
                } else {
                    self.view_top = 0;
                }
            }
            "s" => {
                if self.view_top + 10 < self.rows {
                    let remaining = self.rows - self.view_top - 10;
                    self.view_top += remaining.min(10);
                }
            }
            "a" => {
                if self.view_left >= 10 {
                    self.view_left -= 10;
                } else {
                    self.view_left = 0;
                }
            }
            "d" => {
                if self.view_left + 10 < self.cols {
                    let remaining = self.cols - self.view_left - 10;
                    self.view_left += remaining.min(10);
                }
            }
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

    let rows = args[1].parse::<usize>().unwrap_or(0);
    let cols = args[2].parse::<usize>().unwrap_or(0);

    if rows == 0 || cols == 0 {
        println!("Rows and columns must be > 0.");
        return;
    }

    if rows > 999 {
        println!("Maximum rows: 999");
        return;
    }

    if cols > 18278 {
        println!("Maximum columns: 18278 (up to ZZZ)");
        return;
    }

    let mut sheet = Spreadsheet::new(rows, cols);

    loop {
        sheet.print_grid();
        println!("\nUse WASD to scroll, 'q' to quit, or enter a formula like 'A1=7', 'B2=3+C3'");
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "q" {
            break;
        } else if ["w", "a", "s", "d"].contains(&input) {
            sheet.scroll(input);
        } else {
            sheet.handle_assignment(input);
        }
    }
}