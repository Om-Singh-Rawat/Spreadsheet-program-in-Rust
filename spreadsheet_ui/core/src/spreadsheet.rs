use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq)]
enum Expression {
    Literal(i32),
    Cell(usize, usize),
    BinaryOp(Box<Expression>, char, Box<Expression>),
    Function(String, String),
}

enum FunctionArg {
    Range((usize, usize), (usize, usize)), // ((row1, col1), (row2, col2))
    #[allow(dead_code)]
    Cell(usize, usize),                    // (row, col)
    #[allow(dead_code)]
    Literal(i32),
}

#[derive(Clone, PartialEq)]
pub struct Spreadsheet {
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
    cell_raw: HashMap<(usize, usize), String>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
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
            cell_raw: HashMap::new(),
        }
    }

    // --- Utility functions unchanged ---

    pub fn column_index_to_label(mut index: usize) -> String {
        let mut label = String::new();
        index += 1;
        while index > 0 {
            let rem = (index - 1) % 26;
            label.insert(0, (b'A' + rem as u8) as char);
            index = (index - 1) / 26;
        }
        label
    }

    pub fn column_label_to_index(label: &str) -> Option<usize> {
        let mut index = 0;
        for c in label.chars() {
            if !c.is_ascii_uppercase() {
                return None;
            }
            index = index * 26 + (c as usize - 'A' as usize + 1);
        }
        Some(index - 1)
    }

    pub fn print_grid(&self) {
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

    pub fn update_cell(&mut self, row: usize, col: usize, value: i32) {
        if row < self.rows && col < self.cols {
            self.grid[row][col] = value;
        }
    }

    pub fn get_cell_value(&self, row: usize, col: usize) -> Option<i32> {
        self.grid.get(row)?.get(col).copied()
    }

    pub fn parse_cell_reference(cell: &str) -> Option<(usize, usize)> {
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

    fn parse_operand(&self, operand: &str) -> Result<Expression, String> {
        let operand = operand.trim();
        operand.parse::<i32>()
            .map(Expression::Literal)
            .or_else(|_| {
                if let Some((r, c)) = Self::parse_cell_reference(operand) {
                    if r >= self.rows || c >= self.cols {
                        let label = format!("{}{}", Self::column_index_to_label(c), r + 1);
                        Err(format!("Cell {} is out of bounds", label))
                    } else {
                        Ok(Expression::Cell(r, c))
                    }
                } else {
                    Err(format!("'{}' is not a valid integer or cell reference", operand))
                }
            })
    }

    fn parse_expression(&self, expr: &str) -> Result<Expression, String> {
        let trimmed = expr.trim();
    
        if let Some(open_paren) = trimmed.find('(') {
            if !trimmed.ends_with(')') {
                return Err("invalid function declaration".to_string());
            }
    
            let func_name = trimmed[..open_paren].trim();
            if func_name.is_empty() {
                return Err("empty function name".to_string());
            }
    
            if !func_name.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(format!("invalid function name: '{}'", func_name));
            }
    
            let func_name_upper = func_name.to_uppercase();
            let supported = ["SUM", "AVG", "MIN", "MAX", "STDEV", "SLEEP"];
            if !supported.contains(&func_name_upper.as_str()) {
                return Err(format!("unknown function: '{}'", func_name_upper));
            }
    
            let arg = trimmed[open_paren+1..trimmed.len()-1].trim();
            let func_arg = self.parse_function_argument(arg)?;
    
            match func_name_upper.as_str() {
                "SUM" | "AVG" | "MIN" | "MAX" | "STDEV" => {
                    if !matches!(func_arg, FunctionArg::Range(..)) {
                        return Err(format!("{} requires a range", func_name_upper));
                    }
                }
                "SLEEP" => {
                    if !matches!(func_arg, FunctionArg::Cell(..) | FunctionArg::Literal(..)) {
                        return Err("SLEEP requires a cell reference or integer".to_string());
                    }
                }
                _ => unreachable!()
            }
    
            Ok(Expression::Function(func_name_upper, arg.to_string()))
        
        } else {
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
    
                let left_expr = self.parse_operand(left)?;
                let right_expr = self.parse_operand(right)?;
    
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
                    if r >= self.rows || c >= self.cols {
                        let label = format!("{}{}", Self::column_index_to_label(c), r + 1);
                        return Err(format!("Cell {} is out of bounds", label));
                    }
                    return Ok(Expression::Cell(r, c));
                }
    
                if trimmed.is_empty() {
                    return Err("expression is empty".to_string());
                }
    
                return Err(format!("'{}' is not a valid expression", trimmed));
            }
    
            Err("expressions can only contain one operator".to_string())
        }
    }

    // --- Dependency Methods ---

    // Extract all cell references (dependencies) from an expression.
    /// Extract dependencies by walking the expression tree.
    /// For SUM functions, if the argument decodes to a range, all cells in that range are added.
fn extract_dependencies(expr: &Expression) -> HashSet<(usize, usize)> {
    let mut deps = HashSet::new();

    fn recurse(expr: &Expression, deps: &mut HashSet<(usize, usize)>) {
        match expr {
            Expression::Cell(r, c) => {
                deps.insert((*r, *c));
            }
            Expression::BinaryOp(left, _, right) => {
                recurse(left, deps);
                recurse(right, deps);
            }
            Expression::Function(name, arg) => {
                // Handle all range-based functions the same way
                match name.as_str() {
                    "SUM" | "MIN" | "MAX" | "AVG" | "STDEV" => {
                        if let Ok(FunctionArg::Range((r1, c1), (r2, c2))) = 
                            Spreadsheet::parse_function_argument_static(arg) 
                        {
                            for row in r1.min(r2)..=r1.max(r2) {
                                for col in c1.min(c2)..=c1.max(c2) {
                                    deps.insert((row, col));
                                }
                            }
                        }
                    }
                    // Add other function types here if needed
                    _ => {}
                }
            }
            _ => {}
        }
    }

    recurse(expr, &mut deps);
    deps
}

    /// Update dependencies for a given cell:
    /// Remove the old dependencies, then extract from the new expression and update both
    /// forward and reverse dependency maps.
    fn update_dependencies(&mut self, cell: (usize, usize), expr: &Expression) -> Result<(), String> {
        // Remove old reverse dependencies for this cell.
        if let Some(old_deps) = self.dependencies.get(&cell) {
            for dep in old_deps {
                if let Some(set) = self.reverse_dependencies.get_mut(dep) {
                    set.remove(&cell);
                }
            }
        }

        // Extract new dependencies from the expression.
        let new_deps = Self::extract_dependencies(expr);
        for dep in &new_deps {
            self.reverse_dependencies.entry(*dep).or_default().insert(cell);
        }
        self.dependencies.insert(cell, new_deps);
        Ok(())
    }

    /// Cycle detection remains unchanged.
    /// It simulates applying the new dependencies for 'start' and then runs a DFS to detect cycles.
    pub fn detect_cycle(&self, start: (usize, usize), new_deps: &HashSet<(usize, usize)>) -> bool {
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

    /// Compute a topological order of all cells that are affected by a change starting from `start`.
    /// This orders cells so that any given cell is recalculated only after its dependencies.
    pub fn topological_order(&self, start: (usize, usize)) -> Vec<(usize, usize)> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        affected.insert(start);

        // Collect all affected cells (BFS via reverse_dependencies).
        while let Some(cell) = queue.pop_front() {
            if let Some(dependents) = self.reverse_dependencies.get(&cell) {
                for &dep in dependents {
                    if affected.insert(dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }

        // Build in-degree counts for the affected cells.
        let mut in_degree: HashMap<(usize, usize), usize> = HashMap::new();
        for &cell in &affected {
            in_degree.insert(cell, 0);
        }
        // For each cell, count dependencies that are also affected.
        for &cell in &affected {
            if let Some(deps) = self.dependencies.get(&cell) {
                for &dep in deps {
                    if affected.contains(&dep) {
                        *in_degree.entry(cell).or_insert(0) += 1;
                    }
                }
            }
        }

        // Use a queue to gather all cells with no incoming affected dependency.
        let mut topo_queue: VecDeque<(usize, usize)> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&cell, _)| cell)
            .collect();
        let mut sorted = Vec::new();
        while let Some(cell) = topo_queue.pop_front() {
            sorted.push(cell);
            // Decrease the in-degree for dependents.
            if let Some(dependents) = self.reverse_dependencies.get(&cell) {
                for &dep in dependents {
                    if affected.contains(&dep) {
                        if let Some(degree) = in_degree.get_mut(&dep) {
                            *degree = degree.saturating_sub(1);
                            if *degree == 0 {
                                topo_queue.push_back(dep);
                            }
                        }
                    }
                }
            }
        }
        sorted
    }

    /// Recalculate all dependent cells (including the changed one) in the proper dependency order.
    /// This version uses topological sorting to ensure that a cell’s dependencies are updated first.
    pub fn recalculate_dependents(&mut self, cell: (usize, usize)) -> Result<(), String> {
        // Obtain the topologically sorted order of all affected cells.
        let sorted_cells = self.topological_order(cell);

        // Process each cell in the determined order.
        for current in sorted_cells {
            // Propagate error: if any direct dependency has an error, mark this cell as error.
            if let Some(deps) = self.dependencies.get(&current) {
                let mut dep_error = None;
                for &d in deps {
                    if let Some(err) = self.errors.get(&d) {
                        dep_error = Some(err.clone());
                        break;  // Stop at the first encountered error.
                    }
                }
                if dep_error.is_some() {
                    self.errors.insert(current, "ERR".to_string());
                    self.update_cell(current.0, current.1, 0); // Set to default error value.
                    continue;  // Skip further evaluation for this cell.
                }
            }
            // If no dependency error, evaluate the cell’s expression.
            if let Some(expr) = self.cell_expressions.get(&current) {
                match self.evaluate_expression(expr) {
                    Ok(val) => {
                        self.update_cell(current.0, current.1, val);
                        self.errors.remove(&current); // Clear previous errors.
                    }
                    Err(e) => {
                        self.errors.insert(current, e.clone());
                        self.update_cell(current.0, current.1, 0);
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

    fn parse_function_argument_static(arg: &str) -> Result<FunctionArg, String> {
        let trimmed = arg.trim();
    
        if let Some(colon_pos) = trimmed.find(':') {
            let (start, end) = trimmed.split_at(colon_pos);
            let end = &end[1..];
    
            let start_pos = Self::parse_cell_reference(start)
                .ok_or_else(|| format!("Invalid start cell: '{}'", start))?;
            let end_pos = Self::parse_cell_reference(end)
                .ok_or_else(|| format!("Invalid end cell: '{}'", end))?;
    
            Ok(FunctionArg::Range(start_pos, end_pos))
        } else if let Some((row, col)) = Self::parse_cell_reference(trimmed) {
            Ok(FunctionArg::Cell(row, col))
        } else if let Ok(val) = trimmed.parse::<i32>() {
            Ok(FunctionArg::Literal(val))
        } else {
            Err(format!("Invalid function argument: {}", trimmed))
        }
    }
    
    // Keep your original method for use in evaluation contexts
    fn parse_function_argument(&self, arg: &str) -> Result<FunctionArg, String> {
        // First use the static method to parse the argument
        let parsed = Self::parse_function_argument_static(arg)?;
        
        // Then do bounds checking based on the parsed result
        match parsed {
            FunctionArg::Range((start_row, start_col), (end_row, end_col)) => {
                // Check start cell bounds
                if start_row >= self.rows || start_col >= self.cols {
                    let label = format!("{}{}", 
                        Self::column_index_to_label(start_col),
                        start_row + 1
                    );
                    return Err(format!("Start cell {} is out of bounds", label));
                }
                
                // Check end cell bounds
                if end_row >= self.rows || end_col >= self.cols {
                    let label = format!("{}{}", 
                        Self::column_index_to_label(end_col),
                        end_row + 1
                    );
                    return Err(format!("End cell {} is out of bounds", label));
                }
                
                // Check that range is in ascending order
                if start_row > end_row || start_col > end_col {
                    return Err("Range must be in ascending order".to_string());
                }
                
                Ok(FunctionArg::Range((start_row, start_col), (end_row, end_col)))
            },
            FunctionArg::Cell(row, col) => {
                if row >= self.rows || col >= self.cols {
                    let label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("Cell {} is out of bounds", label));
                }
                Ok(FunctionArg::Cell(row, col))
            },
            // Literals don't need bounds checking
            FunctionArg::Literal(val) => Ok(FunctionArg::Literal(val)),
        }
    }
    


    // --- Function stubs ---
    pub fn handle_sleep(&self, arg: &str) -> Result<i32, String> {
        match self.parse_function_argument(arg)? {
            FunctionArg::Cell(row, col) => {
                self.get_cell_value(row, col)
                    .ok_or_else(|| format!("Invalid cell: {}{}", Self::column_index_to_label(col), row + 1))
            }
            FunctionArg::Literal(val) => Ok(val),
            _ => unreachable!() // Already validated during parsing
        }
    }

    pub fn handle_sum(&self, arg: &str) -> Result<i32, String> {
        // SAFETY: `parse_expression` already guarantees `arg` is a valid range.
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) = 
            self.parse_function_argument(arg)? 
        else {
            // This case is unreachable due to validation in `parse_expression`.
            unreachable!("SUM argument should have been validated as a range during parsing")
        };
    
        let mut sum = 0;
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    sum += value;
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
        Ok(sum)
    }

    pub fn handle_max(&self, arg: &str) -> Result<i32, String> {
        // Get range from argument
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) = 
            self.parse_function_argument(arg)? 
        else {
            unreachable!("MAX argument should have been validated as a range during parsing")
        };
    
        let mut values = Vec::new();
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    values.push(value);
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
    
        if values.is_empty() {
            return Err("MAX of empty range".to_string());
        }
    
        Ok(*values.iter().max().unwrap())
    }
    
    pub fn handle_min(&self, arg: &str) -> Result<i32, String> {
        // Get range from argument
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) = 
            self.parse_function_argument(arg)? 
        else {
            unreachable!("MIN argument should have been validated as a range during parsing")
        };
    
        let mut values = Vec::new();
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    values.push(value);
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
    
        if values.is_empty() {
            return Err("MIN of empty range".to_string());
        }
    
        Ok(*values.iter().min().unwrap())
    }
    
    pub fn handle_avg(&self, arg: &str) -> Result<i32, String> {
        // Get range from argument
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) = 
            self.parse_function_argument(arg)? 
        else {
            unreachable!("AVG argument should have been validated as a range during parsing")
        };
    
        let mut sum = 0;
        let mut count = 0;
        
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    sum += value;
                    count += 1;
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
    
        if count == 0 {
            return Err("AVG of empty range".to_string());
        }
    
        // Integer division (truncating)
        Ok(sum / count)
    }
    
    pub fn handle_stdev(&self, arg: &str) -> Result<i32, String> {
        // Get range from argument
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) = 
            self.parse_function_argument(arg)? 
        else {
            unreachable!("STDEV argument should have been validated as a range during parsing")
        };
    
        // Collect all values
        let mut values = Vec::new();
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    values.push(value);
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
    
        if values.len() <= 1 {
            return Err("STDEV needs at least two values".to_string());
        }
    
        // Calculate mean
        let sum: i32 = values.iter().sum();
        let mean = sum as f64 / values.len() as f64;
    
        // Calculate variance
        let variance = values.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / (values.len() - 1) as f64;
    
        // Return standard deviation as integer (rounded)
        Ok(variance.sqrt().round() as i32)
    }
    

    // --- Assignment & Dependency Methods (with cycle detection and recalculation) ---
    pub fn handle_assignment(&mut self, input: &str) -> Result<(), String> {
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
    
        let parsed_expr = self.parse_expression(expr)?;
    
        // Extract new dependencies
        let new_deps = Self::extract_dependencies(&parsed_expr);
    
        // Check for cycles
        if self.detect_cycle((target_row, target_col), &new_deps) {
            return Err("cycle detected".to_string());
        }
    
        // Store new expression _only_ if it’s not a pure literal,
        // otherwise drop any previous formula.
        if !matches!(parsed_expr, Expression::Literal(_)) {
            self.cell_expressions.insert((target_row, target_col), parsed_expr.clone());
        } else {
            self.cell_expressions.remove(&(target_row, target_col));
        }
    
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
    
    

    pub fn scroll(&mut self, dir: &str) {
        match dir {
            "w" => self.view_top = self.view_top.saturating_sub(10),
            "s" => self.view_top = (self.view_top + 10).min(self.rows.saturating_sub(10)),
            "a" => self.view_left = self.view_left.saturating_sub(10),
            "d" => self.view_left = (self.view_left + 10).min(self.cols.saturating_sub(10)),
            _ => {}
        }
    }

    pub fn handle_scroll_to(&mut self, input: &str) -> Result<(), String> {
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

    pub fn process_input(&mut self, input: &str) -> Result<(), String> {
        let trimmed = input.trim().to_string();
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


        /// Return the *raw* contents of a cell.  
    /// If it was entered as a formula/expression, we return that;  
    /// otherwise we return its literal value as a string.
    pub fn get_cell_content(&self, row: usize, col: usize) -> String {
        if let Some(raw) = self.cell_raw.get(&(row, col)) {
            // always echo exactly what the user typed: "A1+1" or "10"
            raw.clone()
        } else {
            // first‐time cells are blank → show zero
            self.get_cell_value(row, col).unwrap_or(0).to_string()
        }
    }

    /// True if this cell was ever assigned a parsed‐expression (i.e. is a “formula”).
    pub fn is_formula(&self, row: usize, col: usize) -> bool {
        self.cell_expressions.contains_key(&(row, col))
    }

    /// If the last computation of this cell errored, return the error string.
    pub fn get_error(&self, row: usize, col: usize) -> Option<String> {
        self.errors.get(&(row, col)).cloned()
    }
}
