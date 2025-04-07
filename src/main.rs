use std::io::{self, Write};

/// The AST for expressions.
enum Expression {
    Literal(i32),
    Cell(usize, usize),
    BinaryOp(Box<Expression>, char, Box<Expression>),
    Function(String, String), // e.g., Function("MAX", "B1:B9")
}

/// The Spreadsheet struct encapsulating the grid and view parameters.
struct Spreadsheet {
    grid: Vec<Vec<i32>>,
    rows: usize,
    cols: usize,
    view_top: usize,
    view_left: usize,
    output_enabled: bool, // for disable/enable output commands
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
            output_enabled: true,
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
        // Only print if output is enabled.
        if !self.output_enabled {
            return;
        }
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

    /// Parses an expression string into an Expression AST.
    fn parse_expression(expr: &str) -> Result<Expression, String> {
        let trimmed = expr.trim();
        // Check if it is a literal.
        if let Ok(num) = trimmed.parse::<i32>() {
            return Ok(Expression::Literal(num));
        }
        // Check if it is a cell reference.
        if let Some((r, c)) = Self::parse_cell_reference(trimmed) {
            return Ok(Expression::Cell(r, c));
        }
        // Check for a binary operation.
        for op in "+-*/".chars() {
            if trimmed.contains(op) {
                let parts: Vec<&str> = trimmed.split(op).collect();
                if parts.len() != 2 {
                    return Err("invalid binary operation format".to_string());
                }
                let left_expr = Self::parse_expression(parts[0])?;
                let right_expr = Self::parse_expression(parts[1])?;
                return Ok(Expression::BinaryOp(Box::new(left_expr), op, Box::new(right_expr)));
            }
        }
        // Check for a function call. (E.g., MAX(A1:B9), SLEEP(3))
        if let Some(open_paren) = trimmed.find('(') {
            if trimmed.ends_with(')') {
                let func_name = &trimmed[..open_paren];
                let arg = &trimmed[open_paren + 1..trimmed.len() - 1];
                return Ok(Expression::Function(func_name.to_uppercase(), arg.to_string()));
            }
        }
        Err("unrecognized expression".to_string())
    }

    /// Evaluates an Expression AST, returning its integer value.
    fn evaluate_expression(&self, expr: &Expression) -> Result<i32, String> {
        match expr {
            Expression::Literal(v) => Ok(*v),
            Expression::Cell(r, c) => self.get_cell_value(*r, *c).ok_or("invalid cell reference".to_string()),
            Expression::BinaryOp(lhs, op, rhs) => {
                let left_val = self.evaluate_expression(lhs)?;
                let right_val = self.evaluate_expression(rhs)?;
                match op {
                    '+' => Ok(left_val + right_val),
                    '-' => Ok(left_val - right_val),
                    '*' => Ok(left_val * right_val),
                    '/' => {
                        if right_val == 0 {
                            Err("division by zero".to_string())
                        } else {
                            Ok(left_val / right_val)
                        }
                    }
                    _ => Err("unknown operator".to_string()),
                }
            }
            Expression::Function(name, arg) => {
                // Stub implementations for future functions.
                match name.as_str() {
                    "SLEEP" => {
                        // Placeholder: In the future, sleep for the given seconds.
                        // Here we simply return 0.
                        Ok(0)
                    }
                    "MAX" => Self::handle_max(arg),
                    "MIN" => Self::handle_min(arg),
                    "AVG" => Self::handle_avg(arg),
                    "SUM" => Self::handle_sum(arg),
                    "STDEV" => Self::handle_stdev(arg),
                    _ => Err(format!("unsupported function: {}", name)),
                }
            }
        }
    }

    // Stub functions for range-based operations.
    fn handle_max(_range: &str) -> Result<i32, String> {
        // Future implementation: compute max over the given range.
        Ok(0)
    }
    fn handle_min(_range: &str) -> Result<i32, String> {
        // Future implementation: compute min over the given range.
        Ok(0)
    }
    fn handle_avg(_range: &str) -> Result<i32, String> {
        // Future implementation: compute average over the given range.
        Ok(0)
    }
    fn handle_sum(_range: &str) -> Result<i32, String> {
        // Future implementation: compute sum over the given range.
        Ok(0)
    }
    fn handle_stdev(_range: &str) -> Result<i32, String> {
        // Future implementation: compute standard deviation over the given range.
        Ok(0)
    }

    /// Handles an assignment of the form "A1 = <expression>".
    fn handle_assignment(&mut self, input: &str) -> Result<(), String> {
        if let Some((left, right)) = input.split_once('=') {
            let left = left.trim();
            let expr_str = right.trim();
            let (r, c) = Self::parse_cell_reference(left).ok_or("invalid target cell".to_string())?;
            let parsed_expr = Self::parse_expression(expr_str)?;
            let value = self.evaluate_expression(&parsed_expr)?;
            self.update_cell(r, c, value);
            Ok(())
        } else {
            Err("unrecognized cmd".to_string())
        }
    }

    /// Processes input that may be a command or an assignment.
    fn process_input(&mut self, input: &str) -> Result<(), String> {
        let trimmed = input.trim();
        // Handle built-in commands.
        match trimmed.to_lowercase().as_str() {
            "disable_output" => {
                self.output_enabled = false;
                return Ok(());
            }
            "enable_output" => {
                self.output_enabled = true;
                return Ok(());
            }
            _ => {}
        }
        // Fallback: treat as an assignment.
        self.handle_assignment(trimmed)
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

    fn handle_scroll_to(&mut self, input: &str) -> Result<(), String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Usage: scroll_to <cell>".to_string());
        }
    
        if let Some((row, col)) = Self::parse_cell_reference(parts[1]) {
            if row >= self.rows || col >= self.cols {
                return Err("scroll_to: cell out of bounds".to_string());
            }
    
            self.view_top = row;
            self.view_left = col;
            Ok(())
        } else {
            Err("invalid cell reference in scroll_to".to_string())
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
        println!("Rows and columns must be > 0");
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
    let mut last_status = String::from("ok");

    loop {
        sheet.print_grid();
        print!("[0.0] ({}) > ", last_status);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "q" {
            break;
        } else if ["w", "a", "s", "d"].contains(&input) {
            sheet.scroll(input);
            last_status = "ok".to_string();
        } else if input.trim_start().starts_with("scroll_to") {
            match sheet.handle_scroll_to(input) {
                Ok(_) => last_status = "ok".to_string(),
                Err(e) => last_status = e,
            }
        } else {
            match sheet.process_input(input) {        
                Ok(_) => last_status = "ok".to_string(),
                Err(e) => last_status = e,
            }
        }
    }
}




//scroll_to <some out of bounds cell> earlier just scrolled to the last cell(row wise and/or column wise) in the sheet. now it gives out of bounds error