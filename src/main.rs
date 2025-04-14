use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

#[derive(Clone)]
enum Expression {
    Literal(i32),
    Cell(usize, usize),
    BinaryOp(Box<Expression>, char, Box<Expression>),
    Function(String, String),
}

struct Spreadsheet {
    grid: Vec<Vec<i32>>,
    rows: usize,
    cols: usize,
    view_top: usize,
    view_left: usize,
    output_enabled: bool,
    cell_expressions: HashMap<(usize, usize), Expression>,
    dependencies: HashMap<(usize, usize), HashSet<(usize, usize)>>,       // For each cell, which cells it depends on.
    reverse_dependencies: HashMap<(usize, usize), HashSet<(usize, usize)>>, // For each cell, which cells depend on it.
    errors: HashMap<(usize, usize), String>, // Cells with errors (display "ERR")
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
            cell_expressions: HashMap::new(),
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    // --- Utility functions unchanged ---

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
                if let Some(_) = self.errors.get(&(i, j)) {
                    print!("{:<4}", "ERR");
                } else {
                    print!("{:<4}", self.grid[i][j]);
                }
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
        self.grid.get(row)?.get(col).copied()
    }

    fn parse_cell_reference(cell: &str) -> Option<(usize, usize)> {
        let split_index = cell.find(|c: char| c.is_ascii_digit())?;
        let (col_part, row_part) = cell.split_at(split_index);
        if col_part.is_empty() {
            return None;
        }
        let col = Self::column_label_to_index(col_part)?;
        let row_raw = row_part.parse::<usize>().ok()?;
        if row_raw == 0 {
            return None;
        }
        Some((row_raw - 1, col))
    }

    fn parse_operand(operand: &str) -> Result<Expression, String> {
        let operand = operand.trim();
        operand.parse::<i32>()
            .map(Expression::Literal)
            .or_else(|_| {
                Self::parse_cell_reference(operand)
                    .map(|(r, c)| Expression::Cell(r, c))
                    .ok_or_else(|| format!("'{}' is not a valid integer or cell reference", operand))
            })
    }

    fn parse_expression(expr: &str) -> Result<Expression, String> {
        let trimmed = expr.trim();

        if let Some(open_paren) = trimmed.find('(') {
            if trimmed.ends_with(')') {
                let func_name = trimmed[..open_paren].trim().to_uppercase();
                let arg = trimmed[open_paren+1..trimmed.len()-1].trim().to_string();
                return Ok(Expression::Function(func_name, arg));
            }
        }

        let operators = ['+', '-', '*', '/'];
        let mut operator_pos = None;
        let mut operator_count = 0;

        for (i, c) in trimmed.chars().enumerate() {
            if operators.contains(&c) {
                operator_count += 1;
                operator_pos = Some((i, c));
            }
        }

        if operator_count == 1 {
            let (op_pos, op_char) = operator_pos.unwrap();
            let left = trimmed[..op_pos].trim();
            let right = trimmed[op_pos + 1..].trim();

            if left.is_empty() || right.is_empty() {
                return Err("missing operand(s)".to_string());
            }

            let left_expr = Self::parse_operand(left)?;
            let right_expr = Self::parse_operand(right)?;

            return Ok(Expression::BinaryOp(
                Box::new(left_expr),
                op_char,
                Box::new(right_expr),
            ));
        }

        if operator_count == 0 {
            if let Ok(num) = trimmed.parse::<i32>() {
                return Ok(Expression::Literal(num));
            }

            if let Some((r, c)) = Self::parse_cell_reference(trimmed) {
                return Ok(Expression::Cell(r, c));
            }

            if trimmed.is_empty() {
                return Err("expression is empty".to_string());
            }

            return Err(format!("'{}' is not a valid expression", trimmed));
        }

        Err("expressions can only contain one operator".to_string())
    }

    // --- Dependency Methods ---

    // Extract all cell references (dependencies) from an expression.
    fn extract_dependencies(expr: &Expression) -> HashSet<(usize, usize)> {
        let mut deps = HashSet::new();
        fn recurse(expr: &Expression, deps: &mut HashSet<(usize, usize)>) {
            match expr {
                Expression::Cell(r, c) => { deps.insert((*r, *c)); }
                Expression::BinaryOp(left, _, right) => {
                    recurse(left, deps);
                    recurse(right, deps);
                }
                // You can later extend this for functions with ranges.
                _ => {}
            }
        }
        recurse(expr, &mut deps);
        deps
    }

    // Update dependencies (forward and reverse) for a given cell.
    fn update_dependencies(&mut self, cell: (usize, usize), expr: &Expression) -> Result<(), String> {
        // Remove old dependencies for 'cell'
        if let Some(old_deps) = self.dependencies.get(&cell) {
            for dep in old_deps {
                if let Some(set) = self.reverse_dependencies.get_mut(dep) {
                    set.remove(&cell);
                }
            }
        }

        let new_deps = Self::extract_dependencies(expr);
        for dep in &new_deps {
            self.reverse_dependencies.entry(*dep).or_default().insert(cell);
        }
        self.dependencies.insert(cell, new_deps);
        Ok(())
    }

    // Cycle detection: simulate applying new dependencies, then run DFS.
    fn detect_cycle(&self, start: (usize, usize), new_deps: &HashSet<(usize, usize)>) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        fn dfs(
            node: (usize, usize),
            dependencies: &HashMap<(usize, usize), HashSet<(usize, usize)>>,
            visited: &mut HashSet<(usize, usize)>,
            stack: &mut HashSet<(usize, usize)>,
        ) -> bool {
            if stack.contains(&node) {
                return true;
            }
            if visited.contains(&node) {
                return false;
            }
            visited.insert(node);
            stack.insert(node);
            if let Some(neighbors) = dependencies.get(&node) {
                for &neighbor in neighbors {
                    if dfs(neighbor, dependencies, visited, stack) {
                        return true;
                    }
                }
            }
            stack.remove(&node);
            false
        }

        let mut simulated = self.dependencies.clone();
        simulated.insert(start, new_deps.clone());
        dfs(start, &simulated, &mut visited, &mut stack)
    }

    // Recalculate all dependent cells (including the changed one) 
    // Propagating errors: if a dependency fails, mark the cell as error.
    fn recalculate_dependents(&mut self, cell: (usize, usize)) -> Result<(), String> {
        let mut visited = HashSet::new();
        let mut stack = vec![cell];
    
        while let Some(current) = stack.pop() {
            // Propagate to dependents (reverse dependencies)
            if let Some(dependents) = self.reverse_dependencies.get(&current) {
                for &dep in dependents {
                    if visited.insert(dep) {
                        stack.push(dep);
                    }
                }
            }
    
            // Check for errors in dependencies and propagate the error to the current cell
            if let Some(deps) = self.dependencies.get(&current) {
                let mut dep_error = None;
                for &d in deps {
                    // Check if any dependency has an error
                    if let Some(err) = self.errors.get(&d) {
                        dep_error = Some(err.clone());
                        break;  // Stop as soon as an error is found
                    }
                }
    
                // If any dependency had an error, mark the current cell as having an error
                if let Some(_) = dep_error {
                    // Mark the cell as errored (this is the key change)
                    self.errors.insert(current, "ERR".to_string());
                    self.update_cell(current.0, current.1, 0); // Set the cell to 0 as default error value
                    continue;  // Skip further recalculation for this cell
                }
            }
    
            // If there's no error, proceed with normal evaluation
            if let Some(expr) = self.cell_expressions.get(&current) {
                match self.evaluate_expression(expr) {
                    Ok(val) => {
                        self.update_cell(current.0, current.1, val);
                        self.errors.remove(&current);  // Clear any errors from previous evaluations
                    }
                    Err(e) => {
                        // Store the error as 'ERR' and set the value to 0
                        self.errors.insert(current, e.clone());
                        self.update_cell(current.0, current.1, 0); // Set the cell to 0 if error occurs
                    }
                }
            }
        }
        Ok(())
    }
    

    // --- End Dependency Methods ---

    // --- Evaluation methods (unchanged) ---
    fn evaluate_expression(&self, expr: &Expression) -> Result<i32, String> {
        match expr {
            Expression::Literal(v) => Ok(*v),
            Expression::Cell(r, c) => self.get_cell_value(*r, *c)
                .ok_or_else(|| {
                    let col_label = Self::column_index_to_label(*c);
                    format!("invalid cell reference {}{}", col_label, r + 1)
                }),
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
            Expression::Function(name, arg) => match name.as_str() {
                "SLEEP" => self.handle_sleep(arg),
                "MAX" => self.handle_max(arg),
                "MIN" => self.handle_min(arg),
                "AVG" => self.handle_avg(arg),
                "SUM" => self.handle_sum(arg),
                "STDEV" => self.handle_stdev(arg),
                _ => Err(format!("Unknown function: {}", name)),
            },
        }
    }

    // --- Function stubs ---
    fn handle_sleep(&self, arg: &str) -> Result<i32, String> {
        Err(format!("SLEEP({}) not implemented", arg))
    }
    fn handle_max(&self, arg: &str) -> Result<i32, String> {
        Err(format!("MAX({}) not implemented", arg))
    }
    fn handle_min(&self, arg: &str) -> Result<i32, String> {
        Err(format!("MIN({}) not implemented", arg))
    }
    fn handle_avg(&self, arg: &str) -> Result<i32, String> {
        Err(format!("AVG({}) not implemented", arg))
    }
    fn handle_sum(&self, arg: &str) -> Result<i32, String> {
        Err(format!("SUM({}) not implemented", arg))
    }
    fn handle_stdev(&self, arg: &str) -> Result<i32, String> {
        Err(format!("STDEV({}) not implemented", arg))
    }

    // --- Assignment & Dependency Methods (with cycle detection and recalculation) ---
    fn handle_assignment(&mut self, input: &str) -> Result<(), String> {
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err("unrecognized cmd".to_string());
        }
    
        let cell_ref = parts[0].trim();
        let expr = parts[1].trim();
    
        let (target_row, target_col) = Self::parse_cell_reference(cell_ref)
            .ok_or_else(|| format!("invalid cell format: '{}'", cell_ref))?;
    
        if target_row >= self.rows || target_col >= self.cols {
            return Err("target cell out of bounds".to_string());
        }
    
        let parsed_expr = Self::parse_expression(expr)?;
    
        // Extract new dependencies
        let new_deps = Self::extract_dependencies(&parsed_expr);
    
        // Check for cycles
        if self.detect_cycle((target_row, target_col), &new_deps) {
            return Err("cycle detected".to_string());
        }
    
        // Store new expression and update dependencies
        self.cell_expressions.insert((target_row, target_col), parsed_expr.clone());
    
        // Update the dependencies, and handle cycle detection here
        if let Err(_) = self.update_dependencies((target_row, target_col), &parsed_expr) {
            return Err("cycle detected".to_string()); // Return early on cycle detection
        }
    
        // Evaluate expression and handle errors
        let value = match self.evaluate_expression(&parsed_expr) {
            Ok(val) => {
                self.errors.remove(&(target_row, target_col)); // Clear any previous errors
                val
            }
            Err(e) => {
                self.errors.insert((target_row, target_col), e.clone()); // Store error
                0 // Use a fallback value for grid when an error occurs
            }
        };
    
        // Update the grid cell with the evaluated value or error
        self.update_cell(target_row, target_col, value);
    
        // Force recalculation of dependents, even if value didn't change
        let _ = self.recalculate_dependents((target_row, target_col));
    
        Ok(()) // Always return Ok unless cycle detected
    }
    
    

    fn scroll(&mut self, dir: &str) {
        match dir {
            "w" => self.view_top = self.view_top.saturating_sub(10),
            "s" => self.view_top = (self.view_top + 10).min(self.rows.saturating_sub(10)),
            "a" => self.view_left = self.view_left.saturating_sub(10),
            "d" => self.view_left = (self.view_left + 10).min(self.cols.saturating_sub(10)),
            _ => {}
        }
    }

    fn handle_scroll_to(&mut self, input: &str) -> Result<(), String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
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

    fn process_input(&mut self, input: &str) -> Result<(), String> {
        let trimmed = input.trim().to_lowercase();
        match trimmed.as_str() {
            "disable_output" => {
                self.output_enabled = false;
                Ok(())
            }
            "enable_output" => {
                self.output_enabled = true;
                Ok(())
            }
            _ => {
                if trimmed.starts_with("scroll_to") {
                    self.handle_scroll_to(input)
                } else {
                    self.handle_assignment(input)
                }
            }
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
    if rows == 0 || cols == 0 || rows > 999 || cols > 18278 {
        println!("Invalid grid size. Max rows: 999, Max cols: 18278");
        return;
    }
    let mut sheet = Spreadsheet::new(rows, cols);
    let mut last_status = "ok".to_string();
    loop {
        sheet.print_grid();
        print!("[0.0] ({}) > ", last_status);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            last_status = "ok".to_string();
            continue;
        }
        if input == "q" {
            break;
        } else if ["w", "a", "s", "d"].contains(&input) {
            sheet.scroll(input);
            last_status = "ok".to_string();
        } else {
            match sheet.process_input(input) {
                Ok(_) => last_status = "ok".to_string(),
                Err(e) => last_status = e,
            }
        }
    }
}
