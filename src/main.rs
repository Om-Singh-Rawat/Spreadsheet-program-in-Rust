use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, Write};

#[derive(Clone, Debug)]
enum Expression {
    Literal(i32),
    Cell(usize, usize),
    BinaryOp(Box<Expression>, char, Box<Expression>),
    Function(String, String),
}

enum FunctionArg {
    Range((usize, usize), (usize, usize)), // ((row1, col1), (row2, col2))
    #[allow(dead_code)]
    Cell(usize, usize), // (row, col)
    #[allow(dead_code)]
    Literal(i32),
}

struct Spreadsheet {
    grid: Vec<Vec<i32>>,
    rows: usize,
    cols: usize,
    view_top: usize,
    view_left: usize,
    output_enabled: bool,
    cell_expressions: HashMap<(usize, usize), Expression>,
    dependencies: HashMap<(usize, usize), HashSet<(usize, usize)>>, // For each cell, which cells it depends on.
    reverse_dependencies: HashMap<(usize, usize), HashSet<(usize, usize)>>, // For each cell, which cells depend on it.
    errors: HashMap<(usize, usize), String>, // Cells with errors (display "ERR")
    total_sleep_secs: u64,
    last_sleep_time: u64,
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
            total_sleep_secs: 0,
            last_sleep_time: 0,
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
        print!("{:<6} ", "");
        for j in self.view_left..(self.view_left + 10).min(self.cols) {
            print!("{:<6} ", Self::column_index_to_label(j));
        }
        println!();

        for i in self.view_top..(self.view_top + 10).min(self.rows) {
            print!("{:<6} ", i + 1);
            for j in self.view_left..(self.view_left + 10).min(self.cols) {
                if self.errors.contains_key(&(i, j)) {
                    print!("{:<6} ", "ERR");
                } else {
                    print!("{:<6} ", self.grid[i][j]);
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

    fn parse_operand(&self, operand: &str) -> Result<Expression, String> {
        let operand = operand.trim();
        operand
            .parse::<i32>()
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
                    Err(format!(
                        "'{}' is not a valid integer or cell reference",
                        operand
                    ))
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

            let arg = trimmed[open_paren + 1..trimmed.len() - 1].trim();
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
                _ => unreachable!(),
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

                // Handle unary minus case
                if op_char == '-' && left.is_empty() {
                    let right_expr = self.parse_operand(right)?;
                    return Ok(Expression::BinaryOp(
                        Box::new(Expression::Literal(0)),
                        '-',
                        Box::new(right_expr),
                    ));
                }

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
                Expression::Function(name, arg) => match name.as_str() {
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
                    "SLEEP" => {
                        if let Ok(FunctionArg::Cell(r, c)) =
                            Spreadsheet::parse_function_argument_static(arg)
                        {
                            deps.insert((r, c));
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        recurse(expr, &mut deps);
        deps
    }

    /// Update dependencies for a given cell:
    /// Remove the old dependencies, then extract from the new expression and update both
    /// forward and reverse dependency maps.
    fn update_dependencies(
        &mut self,
        cell: (usize, usize),
        expr: &Expression,
    ) -> Result<(), String> {
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
            self.reverse_dependencies
                .entry(*dep)
                .or_default()
                .insert(cell);
        }
        self.dependencies.insert(cell, new_deps);
        Ok(())
    }

    /// Cycle detection remains unchanged.
    /// It simulates applying the new dependencies for 'start' and then runs a DFS to detect cycles.
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

    /// Compute a topological order of all cells that are affected by a change starting from `start`.
    /// This orders cells so that any given cell is recalculated only after its dependencies.
    fn topological_order(&self, start: (usize, usize)) -> Vec<(usize, usize)> {
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
    fn recalculate_dependents(&mut self, cell: (usize, usize)) -> Result<(), String> {
        // Reset total sleep time for this recalculation run
        self.total_sleep_secs = 0;

        let sorted_cells = self.topological_order(cell);

        for current in sorted_cells {
            // Propagate error: if any direct dependency has an error, mark this cell as error.
            if let Some(deps) = self.dependencies.get(&current) {
                let mut dep_error = None;
                for &d in deps {
                    if let Some(err) = self.errors.get(&d) {
                        dep_error = Some(err.clone());
                        break;
                    }
                }
                if dep_error.is_some() {
                    self.errors.insert(current, "ERR".to_string());
                    self.update_cell(current.0, current.1, 0);
                    continue;
                }
            }

            // Fetch the expression early to avoid borrow conflict
            let expr_opt = self.cell_expressions.get(&current).cloned();
            if let Some(expr) = expr_opt {
                match self.evaluate_expression(&expr) {
                    Ok(val) => {
                        self.update_cell(current.0, current.1, val);
                        self.errors.remove(&current);
                    }
                    Err(e) => {
                        self.errors.insert(current, e.clone());
                        self.update_cell(current.0, current.1, 0);
                    }
                }
            }
        }

        // Sleep once after all evaluations
        if self.total_sleep_secs > 0 {
            self.last_sleep_time = self.total_sleep_secs;
            std::thread::sleep(std::time::Duration::from_secs(self.total_sleep_secs));
            self.total_sleep_secs = 0;
        }

        Ok(())
    }

    // --- End Dependency Methods ---

    // --- Evaluation methods (unchanged) ---
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<i32, String> {
        match expr {
            Expression::Literal(v) => Ok(*v),
            Expression::Cell(r, c) => self.get_cell_value(*r, *c).ok_or_else(|| {
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
                    _ => unreachable!("Operator should have been validated during parsing"),
                }
            }
            Expression::Function(name, arg) => match name.as_str() {
                "SLEEP" => self.handle_sleep(arg),
                "MAX" => self.handle_max(arg),
                "MIN" => self.handle_min(arg),
                "AVG" => self.handle_avg(arg),
                "SUM" => self.handle_sum(arg),
                "STDEV" => self.handle_stdev(arg),
                name => unreachable!(
                    "Function '{}' should have been validated during parsing",
                    name
                ),
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
                    let label = format!(
                        "{}{}",
                        Self::column_index_to_label(start_col),
                        start_row + 1
                    );
                    return Err(format!("Start cell {} is out of bounds", label));
                }

                // Check end cell bounds
                if end_row >= self.rows || end_col >= self.cols {
                    let label = format!("{}{}", Self::column_index_to_label(end_col), end_row + 1);
                    return Err(format!("End cell {} is out of bounds", label));
                }

                // Check that range is in ascending order
                if start_row > end_row || start_col > end_col {
                    return Err("Range must be in ascending order".to_string());
                }

                Ok(FunctionArg::Range(
                    (start_row, start_col),
                    (end_row, end_col),
                ))
            }
            FunctionArg::Cell(row, col) => {
                if row >= self.rows || col >= self.cols {
                    let label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("Cell {} is out of bounds", label));
                }
                Ok(FunctionArg::Cell(row, col))
            }
            // Literals don't need bounds checking
            FunctionArg::Literal(val) => Ok(FunctionArg::Literal(val)),
        }
    }

    // --- Function stubs ---
    fn handle_sleep(&mut self, arg: &str) -> Result<i32, String> {
        match self.parse_function_argument(arg)? {
            FunctionArg::Literal(secs) => {
                if secs > 0 {
                    self.total_sleep_secs += secs as u64;
                }
                Ok(secs) // even negative values are returned
            }
            FunctionArg::Cell(r, c) => {
                let val = self
                    .get_cell_value(r, c)
                    .ok_or_else(|| "Invalid cell in SLEEP".to_string())?;

                if val > 0 {
                    self.total_sleep_secs += val as u64;
                }
                Ok(val) // return value as-is
            }
            FunctionArg::Range(_, _) => {
                unreachable!("SLEEP should never receive a range after validation")
            }
        }
    }

    fn handle_sum(&self, arg: &str) -> Result<i32, String> {
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

    fn handle_max(&self, arg: &str) -> Result<i32, String> {
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) =
            self.parse_function_argument(arg)?
        else {
            unreachable!("MAX argument should have been validated as a range during parsing");
        };

        let mut max_val: Option<i32> = None;
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    max_val = Some(match max_val {
                        Some(current_max) => current_max.max(value),
                        None => value,
                    });
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }
        max_val.ok_or_else(|| "MAX of empty range".to_string())
    }

    fn handle_min(&self, arg: &str) -> Result<i32, String> {
        let FunctionArg::Range((start_row, start_col), (end_row, end_col)) =
            self.parse_function_argument(arg)?
        else {
            unreachable!("MIN argument should have been validated as a range during parsing");
        };

        let mut min_val: Option<i32> = None;
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let Some(value) = self.get_cell_value(row, col) {
                    min_val = Some(match min_val {
                        Some(current_min) => current_min.min(value),
                        None => value,
                    });
                } else {
                    let cell_label = format!("{}{}", Self::column_index_to_label(col), row + 1);
                    return Err(format!("invalid cell in range: {}", cell_label));
                }
            }
        }

        min_val.ok_or_else(|| "MIN of empty range".to_string())
    }

    fn handle_avg(&self, arg: &str) -> Result<i32, String> {
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
        // Integer division (truncating)
        Ok(sum / count)
    }

    fn handle_stdev(&self, arg: &str) -> Result<i32, String> {
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
        let variance = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (values.len() - 1) as f64;

        // Return standard deviation as integer (rounded)
        Ok(variance.sqrt().round() as i32)
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

        let parsed_expr = self.parse_expression(expr)?;

        // Extract new dependencies
        let new_deps = Self::extract_dependencies(&parsed_expr);

        // Check for cycles
        if self.detect_cycle((target_row, target_col), &new_deps) {
            return Err("cycle detected".to_string());
        }

        // Store new expression and update dependencies
        self.cell_expressions
            .insert((target_row, target_col), parsed_expr.clone());

        // Update the dependencies, and handle cycle detection here
        if self
            .update_dependencies((target_row, target_col), &parsed_expr)
            .is_err()
        {
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
            // Scroll up if possible
            "w" => {
                self.view_top = self.view_top.saturating_sub(10);
            }

            // Scroll down only if we are showing fewer than 10 rows at the bottom
            "s" => {
                if self.view_top + 10 < self.rows {
                    self.view_top = (self.view_top + 10).min(self.rows - 10);
                }
            }

            // Scroll left if possible
            "a" => {
                self.view_left = self.view_left.saturating_sub(10);
            }

            // Scroll right only if we are showing fewer than 10 columns at the right
            "d" => {
                if self.view_left + 10 < self.cols {
                    self.view_left = (self.view_left + 10).min(self.cols - 10);
                }
            }

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
        print!("[{}.0] ({}) > ", sheet.last_sleep_time, last_status);
        sheet.last_sleep_time = 0;
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

#[cfg(test)]
mod tests {
    use super::*;
    fn create_sheet(rows: usize, cols: usize) -> Spreadsheet {
        Spreadsheet::new(rows, cols)
    }

    #[test]
    fn test_literal_assignment() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 42").unwrap();
        assert_eq!(sheet.get_cell_value(0, 0), Some(42));
    }

    #[test]
    fn test_printing() {
        let mut sheet = create_sheet(0, 8);
        sheet.output_enabled = false;
        sheet.print_grid();
    }

    #[test]
    fn test_binary_addition() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 2").unwrap();
        sheet.process_input("A2 = 3").unwrap();
        sheet.process_input("A3 = A1 + A2").unwrap();
        assert_eq!(sheet.get_cell_value(2, 0), Some(5));
    }

    #[test]
    fn test_cell_reference() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("B1 = 10").unwrap();
        sheet.process_input("C1 = B1").unwrap();
        assert_eq!(sheet.get_cell_value(0, 2), Some(10));
    }

    #[test]
    fn test_invalid_cell() {
        let mut sheet = create_sheet(5, 5);
        let result = sheet.process_input("Z1000 = 5");
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_division_by_zero() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("A2 = 0").unwrap();
        sheet.process_input("A3 = A1 / A2").unwrap();
        assert_eq!(sheet.get_cell_value(2, 0), Some(0));
        assert_eq!(
            sheet.errors.get(&(2, 0)),
            Some(&"division by zero".to_string())
        );
    }

    #[test]
    fn test_sleep_literal() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = SLEEP(2)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 0), Some(2));
    }

    #[test]
    fn test_sleep_cell_reference() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("A2 = SLEEP(A1)").unwrap();
        assert_eq!(sheet.get_cell_value(1, 0), Some(5));
    }

    #[test]
    fn test_dependency_tracking() {
        let mut sheet = create_sheet(5, 5);

        // Set up dependencies
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("B1 = A1").unwrap();

        // Verify B1 depends on A1
        assert!(sheet.dependencies.get(&(0, 1)).unwrap().contains(&(0, 0)));

        // Update B1 to remove dependency
        sheet.process_input("B1 = 10").unwrap();

        // The dependency should be removed
        assert!(!sheet.dependencies.get(&(0, 1)).unwrap().contains(&(0, 0)));
    }

    #[test]
    fn test_function_argument_validation() {
        let mut sheet = create_sheet(5, 5);

        // SUM requires a range, not a single cell
        let result = sheet.process_input("A1 = SUM(B1)");
        assert!(result.is_err());

        // SLEEP requires a literal or cell, not a range
        let result = sheet.process_input("A1 = SLEEP(B1:C2)");
        assert!(result.is_err());

        // AVG with empty range
        let result = sheet.process_input("B1 = AVG()");
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_disable_output() {
        let mut sheet = create_sheet(5, 5);
        assert!(sheet.process_input("disable_output").is_ok());
        assert!(!sheet.output_enabled);

        assert!(sheet.process_input("enable_output").is_ok());
        assert!(sheet.output_enabled);
    }

    #[test]
    fn test_scroll_commands() {
        let mut sheet = create_sheet(20, 20);
        sheet.view_top = 0;
        sheet.view_left = 0;

        // Basic scrolling
        sheet.scroll("s"); // Down
        assert_eq!(sheet.view_top, 10);

        sheet.scroll("d"); // Right
        assert_eq!(sheet.view_left, 10);

        sheet.scroll("w"); // Up
        assert_eq!(sheet.view_top, 0);

        sheet.scroll("a"); // Left
        assert_eq!(sheet.view_left, 0);

        // Boundary conditions
        sheet.view_top = 0;
        sheet.view_left = 0;
        sheet.scroll("w"); // Try to go above top boundary
        assert_eq!(sheet.view_top, 0);
        sheet.scroll("a"); // Try to go past left boundary
        assert_eq!(sheet.view_left, 0);
    }

    #[test]
    fn test_scroll_to_command() {
        let mut sheet = create_sheet(20, 20);

        // Valid scroll_to
        sheet.process_input("scroll_to B5").unwrap();
        assert_eq!(sheet.view_top, 4);
        assert_eq!(sheet.view_left, 1);

        let result = sheet.process_input("scroll_to");
        assert_eq!(result.unwrap_err(), "Usage: scroll_to <cell>");

        // Invalid scroll_to
        let result = sheet.process_input("scroll_to Z100");
        assert!(result.is_err());

        let result = sheet.process_input("scroll_to invalid_cell");
        assert_eq!(result.unwrap_err(), "invalid cell reference in scroll_to");
    }

    #[test]
    fn test_output_toggle() {
        let mut sheet = create_sheet(5, 5);
        sheet.output_enabled = false;

        sheet.process_input("enable_output").unwrap();
        assert!(sheet.output_enabled);

        sheet.process_input("disable_output").unwrap();
        assert!(!sheet.output_enabled);
    }

    #[test]
    fn test_parse_cell_reference() {
        assert_eq!(Spreadsheet::parse_cell_reference("A1"), Some((0, 0)));
        assert_eq!(Spreadsheet::parse_cell_reference("A0"), None);
        assert_eq!(Spreadsheet::parse_cell_reference("B5"), Some((4, 1)));
        assert_eq!(Spreadsheet::parse_cell_reference("AA10"), Some((9, 26)));
        assert_eq!(Spreadsheet::parse_cell_reference("invalid"), None);
    }

    #[test]
    fn test_parse_operand_cell_out_of_bounds() {
        let mut sheet = create_sheet(5, 5); // 5x5 grid: rows 0–4, cols 0–4

        // This cell is in correct format but out of bounds (row 1000)
        let result = sheet.process_input("A1000 = 5");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "target cell out of bounds");

        let result = sheet.parse_operand("A1000 = 5");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "'A1000 = 5' is not a valid integer or cell reference"
        );

        let sheet = create_sheet(5, 5); // Columns A-C, Rows 1-5
        let result = sheet.parse_operand("E6");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cell E6 is out of bounds");
    }

    #[test]
    fn test_parse_expression_edge_cases() {
        let mut sheet = create_sheet(5, 5);

        // Integer literal
        assert!(sheet.process_input("A1 = 42").is_ok());

        // Valid cell reference
        assert!(sheet.process_input("A1 = B1").is_ok());

        // Binary operation
        assert!(sheet.process_input("A1 = B1 + C1").is_ok());

        // Unary minus
        assert!(sheet.process_input("A1 = -5").is_ok());

        // Invalid: multiple operators
        assert!(sheet.process_input("A1 = 5 + 3 * 2").is_err());

        // Invalid: empty
        assert!(sheet.process_input("A1 = ").is_err());

        // Invalid: malformed cell
        assert!(sheet.process_input("A1 = B").is_err());

        let result = sheet.parse_expression("AF11");
        assert_eq!(result.unwrap_err(), "Cell AF11 is out of bounds");
    }

    #[test]
    fn test_extract_and_update_dependencies() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("B1 = A1 + A1").unwrap();

        let deps = sheet.dependencies.get(&(0, 1)).unwrap();
        assert_eq!(deps.len(), 1); // Set should not duplicate (A1 only once)
    }

    #[test]
    fn test_error_handling() {
        let mut sheet = create_sheet(5, 5);

        // Division by zero
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("A2 = 0").unwrap();
        sheet.process_input("A3 = A1 / A2").unwrap();
        assert_eq!(sheet.get_cell_value(2, 0), Some(0)); // Default value on error
        assert!(sheet.errors.contains_key(&(2, 0))); // Error flag is set

        // Invalid cell reference
        let result = sheet.process_input("Z100 = 5");
        assert!(result.is_err());

        let result = sheet.process_input("B1 = AVG(A6:A10)"); // Row 5 (0-based) is beyond 4
        assert_eq!(result.unwrap_err(), "Start cell A6 is out of bounds");

        let result = sheet.process_input("B1 = SUM(A1:F1)"); // F1 is column 5
        assert_eq!(result.unwrap_err(), "End cell F1 is out of bounds");

        // Invalid expression
        let result = sheet.process_input("A1 = 5 + ");
        assert!(result.is_err());

        // Invalid function
        let result = sheet.process_input("A1 = NONEXISTENT(A2:A3)");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_propagation() {
        let mut sheet = create_sheet(5, 5);

        // Create an error
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("A2 = 0").unwrap();
        sheet.process_input("A3 = A1 / A2").unwrap(); // Error

        // Error should propagate through references
        sheet.process_input("B1 = A3").unwrap();
        assert!(sheet.errors.contains_key(&(0, 1)));

        // Error should propagate through functions
        sheet.process_input("B2 = SUM(A3:A3)").unwrap();
        assert!(sheet.errors.contains_key(&(1, 1)));
    }

    #[test]
    fn test_fallback_value_on_error() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("A2 = 0").unwrap();
        sheet.process_input("A3 = A1 / A2").unwrap(); // division by zero
        assert_eq!(sheet.get_cell_value(2, 0), Some(0));
    }

    #[test]
    fn test_cycle_detection() {
        let mut sheet = create_sheet(5, 5);

        // Direct cycle
        sheet.process_input("A1 = B1").unwrap();
        let result = sheet.process_input("B1 = A1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "cycle detected");

        // Self-reference
        let result = sheet.process_input("C1 = C1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "cycle detected");

        // Indirect cycle
        sheet = create_sheet(5, 5);
        sheet.process_input("A1 = B1").unwrap();
        sheet.process_input("B1 = C1").unwrap();
        let result = sheet.process_input("C1 = A1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "cycle detected");
    }

    #[test]
    fn test_dependency_updates() {
        let mut sheet = create_sheet(5, 5);

        // Basic dependency
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("B1 = A1").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(10));

        // Update source cell
        sheet.process_input("A1 = 20").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(20));

        // Multi-level dependencies
        sheet.process_input("C1 = B1 + 5").unwrap();
        assert_eq!(sheet.get_cell_value(0, 2), Some(25));

        // Update original source - should cascade
        sheet.process_input("A1 = 30").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(30)); // B1 updates
        assert_eq!(sheet.get_cell_value(0, 2), Some(35)); // C1 updates
    }

    #[test]
    fn test_function_dependency_updates() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 10").unwrap();
        sheet.process_input("A2 = 20").unwrap();
        sheet.process_input("B1 = SUM(A1:A2)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(30));

        // Update one cell in the range
        sheet.process_input("A1 = 15").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(35));
    }

    #[test]
    fn test_sleep_functions() {
        let mut sheet = create_sheet(5, 5);

        // Test with literal value
        sheet.process_input("A1 = SLEEP(0)").unwrap(); // Use 0 to keep test fast
        assert_eq!(sheet.get_cell_value(0, 0), Some(0));

        // Test with cell reference
        sheet.process_input("B1 = 0").unwrap();
        sheet.process_input("B2 = SLEEP(B1)").unwrap();
        assert_eq!(sheet.get_cell_value(1, 1), Some(0));

        // Test with negative value (should not actually sleep)
        sheet.process_input("C1 = SLEEP(-1)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 2), Some(-1));
        let result = sheet.process_input("C3 = SLEEP(A1:B2)");
        assert!(result.is_err(), " invalid function argument");
    }
    #[test]
    fn test_sum_function() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 1").unwrap();
        sheet.process_input("A2 = 2").unwrap();
        sheet.process_input("A3 = 3").unwrap();
        sheet.process_input("B1 = SUM(A1:A3)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(6));
    }

    #[test]
    fn test_avg_function() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 4").unwrap();
        sheet.process_input("A2 = 2").unwrap();
        sheet.process_input("A3 = 6").unwrap();
        sheet.process_input("B1 = AVG(A1:A3)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(4));
        let result = sheet.process_input("B1 = AVG()");
        assert!(result.is_err());
    }

    #[test]
    fn test_min_max_functions() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 4").unwrap();
        sheet.process_input("A2 = -1").unwrap();
        sheet.process_input("A3 = 6").unwrap();

        // Test MIN function
        sheet.process_input("C1 = MIN(A1:B3)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 2), Some(-1));

        let result = sheet.process_input("C1 = MIN()");
        assert!(result.is_err());
        // Test MAX function
        sheet.process_input("C2 = MAX(A1:B3)").unwrap();
        assert_eq!(sheet.get_cell_value(1, 2), Some(6));

        let result = sheet.process_input("B1 = MAX()");
        assert!(result.is_err());
    }

    #[test]
    fn test_stdev_function() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 2").unwrap();
        sheet.process_input("A2 = 4").unwrap();
        sheet.process_input("A3 = 6").unwrap();
        sheet.process_input("B1 = STDEV(A1:A3)").unwrap();
        // Standard deviation of 2,4,6 is 2
        assert_eq!(sheet.get_cell_value(0, 1), Some(2));

        let result = sheet.process_input("A1 = SUM(A1:A3");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid function declaration");

        // STDEV with one element
        sheet.process_input("A1 = 5").unwrap();
        let result = sheet.process_input("B1 = STDEV(A1:A1)");
        assert!(result.is_ok());
        assert_eq!(sheet.get_cell_value(0, 1), Some(0));

        // STDEV with empty range
        let result = sheet.process_input("B2 = STDEV()");
        assert!(result.is_err());

        let result = sheet.process_input("A1 = (A1:A3)");
        assert_eq!(result.unwrap_err(), "empty function name");

        let result = sheet.process_input("A1 = S@M(A1:A3)");
        assert_eq!(result.unwrap_err(), "invalid function name: 'S@M'");
    }

    #[test]
    fn test_column_conversion() {
        assert_eq!(Spreadsheet::column_index_to_label(0), "A");
        assert_eq!(Spreadsheet::column_index_to_label(25), "Z");
        assert_eq!(Spreadsheet::column_index_to_label(26), "AA");
        assert_eq!(Spreadsheet::column_index_to_label(701), "ZZ");
        assert_eq!(Spreadsheet::column_index_to_label(702), "AAA");

        // Column label to index
        assert_eq!(Spreadsheet::column_label_to_index("A"), Some(0));
        assert_eq!(Spreadsheet::column_label_to_index("Z"), Some(25));
        assert_eq!(Spreadsheet::column_label_to_index("AA"), Some(26));
        assert_eq!(Spreadsheet::column_label_to_index("ZZ"), Some(701));

        // Invalid labels
        assert_eq!(Spreadsheet::column_label_to_index("a"), None); // Lowercase
        assert_eq!(Spreadsheet::column_label_to_index("123"), None); // Number
    }

    #[test]
    fn test_edge_cases() {
        let mut sheet = create_sheet(5, 5);

        // Empty expression
        let result = sheet.process_input("A1 = ");
        assert!(result.is_err());

        // Range out of bounds
        let result = sheet.process_input("A1 = SUM(A1:Z100)");
        assert!(result.is_err());

        // STDEV with insufficient values
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("B1 = STDEV(A1:A1)").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(0));

        // Function case insensitivity
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("A2 = 10").unwrap();
        sheet.process_input("B1 = sum(A1:A2)").unwrap(); // lowercase function name
        assert_eq!(sheet.get_cell_value(0, 1), Some(15));

        // Multiple operators (not allowed)
        let result = sheet.process_input("A1 = 5 + 3 * 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_scroll_boundary_conditions() {
        let mut sheet = create_sheet(20, 20);

        // Test scrolling beyond grid boundaries
        sheet.view_top = 15;
        sheet.scroll("s");
        assert_eq!(sheet.view_top, 15); // Shouldn't scroll past row 15

        sheet.view_left = 15;
        sheet.scroll("d");
        assert_eq!(sheet.view_left, 15); // Shouldn't scroll past column 15
    }

    #[test]
    fn test_complex_dependency_ordering() {
        let mut sheet = create_sheet(5, 5);
        sheet.process_input("A1 = 5").unwrap();
        sheet.process_input("B1 = A1").unwrap();
        sheet.process_input("C1 = B1").unwrap();
        sheet.process_input("D1 = C1").unwrap();

        // Update root cell and verify propagation order
        sheet.process_input("A1 = 10").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(10));
        assert_eq!(sheet.get_cell_value(0, 2), Some(10));
        assert_eq!(sheet.get_cell_value(0, 3), Some(10));
    }

    #[test]
    fn test_binary_operations() {
        let mut sheet = create_sheet(5, 5);
        // Addition
        sheet.process_input("A1 = 2").unwrap();
        sheet.process_input("A2 = 3").unwrap();
        sheet.process_input("B1 = A1 + A2").unwrap();
        assert_eq!(sheet.get_cell_value(0, 1), Some(5));

        // Subtraction
        sheet.process_input("B2 = A2 - A1").unwrap();
        assert_eq!(sheet.get_cell_value(1, 1), Some(1));

        // Multiplication
        sheet.process_input("B3 = A1 * A2").unwrap();
        assert_eq!(sheet.get_cell_value(2, 1), Some(6));

        // Division
        sheet.process_input("B4 = A2 / A1").unwrap();
        assert_eq!(sheet.get_cell_value(3, 1), Some(1));
    }

    #[test]
    fn test_complex_sleep_and_dependency_chain() {
        let mut sheet = Spreadsheet::new(4, 4);
        sheet.output_enabled = false; // equivalent to "disable_output"

        sheet.process_input("B1 = SLEEP(A1)").unwrap();
        sheet.process_input("C1 = A1 + B1").unwrap();
        sheet.process_input("C2 = B1 + A1").unwrap();
        sheet.process_input("D1 = SLEEP(C1)").unwrap();
        sheet.process_input("D2 = SLEEP(C2)").unwrap();
        sheet.process_input("A1 = 1").unwrap();

        assert_eq!(sheet.get_cell_value(0, 0), Some(1)); // A1
        assert_eq!(sheet.get_cell_value(0, 1), Some(1)); // B1
        assert_eq!(sheet.get_cell_value(0, 2), Some(2)); // C1
        assert_eq!(sheet.get_cell_value(1, 2), Some(2)); // C2
        assert_eq!(sheet.get_cell_value(0, 3), Some(2)); // D1
        assert_eq!(sheet.get_cell_value(1, 3), Some(2)); // D2
    }

    #[test]
    fn test_stats_functions_sleep_and_recalculation() {
        let mut sheet = create_sheet(26, 26);

        sheet.output_enabled = false;

        sheet.process_input("C1 = MAX(A1:A26)").unwrap();
        sheet.process_input("B1 = SLEEP(C1)").unwrap();

        sheet.process_input("C2 = MIN(A1:A26)").unwrap();
        sheet.process_input("B2 = SLEEP(C2)").unwrap();

        sheet.process_input("C3 = AVG(A1:A26)").unwrap();
        sheet.process_input("B3 = SLEEP(C3)").unwrap();

        sheet.process_input("C4 = STDEV(A1:A26)").unwrap();
        sheet.process_input("B4 = SLEEP(C4)").unwrap();

        sheet.process_input("D1 = MAX(A1:B4)").unwrap();
        sheet.process_input("E1 = SLEEP(D1)").unwrap();

        sheet.process_input("A1 = 2").unwrap();

        assert_eq!(sheet.get_cell_value(0, 0), Some(2)); // A1
        assert_eq!(sheet.get_cell_value(0, 2), Some(2)); // C1
        assert_eq!(sheet.get_cell_value(0, 1), Some(2)); // B1
        assert_eq!(sheet.get_cell_value(1, 2), Some(0)); // C2
        assert_eq!(sheet.get_cell_value(1, 1), Some(0)); // B2
        assert_eq!(sheet.get_cell_value(2, 2), Some(0)); // C3
        assert_eq!(sheet.get_cell_value(2, 1), Some(0)); // B3
        assert_eq!(sheet.get_cell_value(3, 2), Some(0)); // C4
        assert_eq!(sheet.get_cell_value(3, 1), Some(0)); // B4
        assert_eq!(sheet.get_cell_value(0, 3), Some(2)); // D1
        assert_eq!(sheet.get_cell_value(0, 4), Some(2)); // E1
    }
}
