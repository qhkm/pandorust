/// Pre-processor that converts grid tables and `\newpage` commands to formats
/// that comrak (GFM markdown parser) can understand.
///
/// Grid tables look like:
/// ```text
/// +-----+--------+----------+
/// | No. | Modul  | Kos (RM) |
/// +=====+========+==========+
/// | 1   | POS    | 3,500    |
/// +-----+--------+----------+
/// ```
///
/// They are converted to GFM pipe tables:
/// ```text
/// | No. | Modul | Kos (RM) |
/// | --- | --- | --- |
/// | POS | 3,500 |
/// ```

/// Preprocess the input markdown string, converting grid tables to GFM pipe
/// tables and `\newpage` to an HTML page-break div.
pub fn preprocess_grid_tables(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let lines: Vec<&str> = input.lines().collect();
    let len = lines.len();
    let mut i = 0;

    while i < len {
        let trimmed = lines[i].trim();

        // Handle \newpage as standalone paragraph
        if trimmed == "\\newpage" {
            output.push_str("<div style=\"page-break-after: always;\"></div>\n");
            i += 1;
            continue;
        }

        // Handle standalone backslash (LaTeX line break) — skip it
        if trimmed == "\\" {
            output.push('\n');
            i += 1;
            continue;
        }

        // Handle pandoc fenced divs ::: {custom-style="..."} ... :::
        // Strip the ::: markers and pass through the inner content
        if trimmed.starts_with(":::") {
            if trimmed.len() > 3 {
                // Opening ::: with attributes — skip this line
                i += 1;
                continue;
            } else {
                // Closing ::: — skip this line
                i += 1;
                continue;
            }
        }

        // Check if this line starts a grid table
        if is_border_line(trimmed) {
            // Collect all lines that are part of this grid table
            let start = i;
            let mut table_lines = Vec::new();
            table_lines.push(lines[i]);
            i += 1;

            while i < len {
                let t = lines[i].trim();
                if is_border_line(t) || is_data_line(t) {
                    table_lines.push(lines[i]);
                    i += 1;
                } else {
                    break;
                }
            }

            // Only convert if we have a valid grid table (at least 3 lines:
            // border, data, border)
            if table_lines.len() >= 3 && is_border_line(table_lines.last().unwrap().trim()) {
                let gfm = convert_grid_to_gfm(&table_lines);
                output.push_str(&gfm);
                // Don't add extra newline if the gfm already ends with one
                if !gfm.ends_with('\n') {
                    output.push('\n');
                }
            } else {
                // Not a valid grid table, output lines as-is
                for line in &table_lines {
                    output.push_str(line);
                    output.push('\n');
                }
            }
            // Skip the i increment at the bottom since we already advanced i
            // inside the while loop
            let _ = start; // suppress unused warning
            continue;
        }

        output.push_str(lines[i]);
        output.push('\n');
        i += 1;
    }

    // Remove trailing newline if the original input didn't have one
    if !input.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }

    output
}

/// Check if a line is a grid table border line: starts with `+` and contains
/// only `+`, `-`, and `=` characters.
fn is_border_line(line: &str) -> bool {
    if !line.starts_with('+') || !line.ends_with('+') {
        return false;
    }
    if line.len() < 3 {
        return false;
    }
    line.chars().all(|c| c == '+' || c == '-' || c == '=')
}

/// Check if a line is a grid table data line: starts and ends with `|`.
fn is_data_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|')
}

/// Check if a border line is a header separator (uses `=` instead of `-`).
fn is_header_separator(line: &str) -> bool {
    let trimmed = line.trim();
    is_border_line(trimmed) && trimmed.contains('=')
}

/// Find column boundary positions from a border line.
/// Returns byte positions of `+` characters.
fn find_column_boundaries(border_line: &str) -> Vec<usize> {
    border_line
        .char_indices()
        .filter(|(_, c)| *c == '+')
        .map(|(i, _)| i)
        .collect()
}

/// Extract cell content from a data line given column boundary positions.
fn extract_cell(line: &str, start: usize, end: usize) -> String {
    if start + 1 < end && end <= line.len() {
        // The data line uses `|` at column boundaries instead of `+`
        line[start + 1..end].trim().to_string()
    } else {
        String::new()
    }
}

/// A single logical row may consist of multiple data lines (multiline cells).
/// This struct accumulates content for each cell across those lines.
struct GridRow {
    cells: Vec<String>,
}

impl GridRow {
    fn new(num_cols: usize) -> Self {
        GridRow {
            cells: vec![String::new(); num_cols],
        }
    }

    /// Append content from a data line to the cells.
    fn add_line(&mut self, line: &str, boundaries: &[usize]) {
        let num_cols = self.cells.len();
        for col in 0..num_cols {
            if col + 1 < boundaries.len() {
                let content = extract_cell(line, boundaries[col], boundaries[col + 1]);
                if !content.is_empty() {
                    if !self.cells[col].is_empty() {
                        self.cells[col].push(' ');
                    }
                    self.cells[col].push_str(&content);
                }
            }
        }
    }
}

/// Convert collected grid table lines into a GFM pipe table string.
fn convert_grid_to_gfm(table_lines: &[&str]) -> String {
    // Find column boundaries from the first border line
    let first_border = table_lines[0].trim();
    let boundaries = find_column_boundaries(first_border);

    if boundaries.len() < 2 {
        // Not enough columns, return lines as-is
        return table_lines.join("\n");
    }

    let num_cols = boundaries.len() - 1;

    // Determine if there's a header separator
    let header_sep_index = table_lines
        .iter()
        .position(|line| is_header_separator(line.trim()));

    // Parse rows: collect data lines between border lines into logical rows
    let mut header_rows: Vec<GridRow> = Vec::new();
    let mut body_rows: Vec<GridRow> = Vec::new();
    let mut current_row = GridRow::new(num_cols);
    let mut in_header = header_sep_index.is_some(); // Start in header if there's a header separator
    let mut past_first_border = false;

    for line in table_lines.iter() {
        let trimmed = line.trim();

        if is_border_line(trimmed) {
            if past_first_border {
                // End of a logical row
                let has_content = current_row.cells.iter().any(|c| !c.is_empty());
                if has_content {
                    if in_header {
                        header_rows.push(current_row);
                    } else {
                        body_rows.push(current_row);
                    }
                }
                current_row = GridRow::new(num_cols);

                // Check if this is the header separator
                if is_header_separator(trimmed) {
                    in_header = false;
                }
            }
            past_first_border = true;
        } else if is_data_line(trimmed) {
            current_row.add_line(trimmed, &boundaries);
        }
    }

    // Build GFM output
    let mut gfm = String::new();

    // If there are header rows, use the first one as the GFM header
    // If no header rows (no === separator), use the first body row as header
    let (gfm_header, gfm_body) = if !header_rows.is_empty() {
        (header_rows, body_rows)
    } else if !body_rows.is_empty() {
        // First body row becomes the header
        let header = vec![body_rows.remove(0)];
        (header, body_rows)
    } else {
        return table_lines.join("\n");
    };

    // Write header row(s) - GFM only supports one header row, use the first
    if let Some(header) = gfm_header.first() {
        gfm.push_str("| ");
        gfm.push_str(&header.cells.join(" | "));
        gfm.push_str(" |\n");
    }

    // Write separator
    let sep_cells: Vec<String> = (0..num_cols).map(|_| "---".to_string()).collect();
    gfm.push_str("| ");
    gfm.push_str(&sep_cells.join(" | "));
    gfm.push_str(" |\n");

    // Write body rows
    for row in &gfm_body {
        gfm.push_str("| ");
        gfm.push_str(&row.cells.join(" | "));
        gfm.push_str(" |\n");
    }

    gfm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_border_line() {
        assert!(is_border_line("+-----+-----+"));
        assert!(is_border_line("+=====+=====+"));
        assert!(is_border_line("+---+---+---+"));
        assert!(!is_border_line("| A | B |"));
        assert!(!is_border_line("not a border"));
        assert!(!is_border_line("+"));
        assert!(!is_border_line("++"));
    }

    #[test]
    fn test_is_data_line() {
        assert!(is_data_line("| A   | B   |"));
        assert!(is_data_line("|     | test |"));
        assert!(!is_data_line("+---+---+"));
        assert!(!is_data_line("no pipes"));
    }

    #[test]
    fn test_is_header_separator() {
        assert!(is_header_separator("+=====+=====+"));
        assert!(is_header_separator("+===+===+===+"));
        assert!(!is_header_separator("+-----+-----+"));
        assert!(!is_header_separator("| A | B |"));
    }

    #[test]
    fn test_find_column_boundaries() {
        assert_eq!(find_column_boundaries("+---+---+"), vec![0, 4, 8]);
        assert_eq!(
            find_column_boundaries("+-----+--------+----------+"),
            vec![0, 6, 15, 26]
        );
    }

    #[test]
    fn test_preprocess_simple_grid_table() {
        let input = "\
+-----+-----+
| A   | B   |
+=====+=====+
| 1   | 2   |
+-----+-----+
| 3   | 4   |
+-----+-----+";
        let result = preprocess_grid_tables(input);
        assert!(result.contains("| A | B |"), "Got: {}", result);
        assert!(result.contains("| --- | --- |"), "Got: {}", result);
        assert!(result.contains("| 1 | 2 |"), "Got: {}", result);
        assert!(result.contains("| 3 | 4 |"), "Got: {}", result);
    }

    #[test]
    fn test_preprocess_newpage() {
        let input = "Above\n\n\\newpage\n\nBelow";
        let result = preprocess_grid_tables(input);
        assert!(
            result.contains("<div style=\"page-break-after: always;\"></div>"),
            "Got: {}",
            result
        );
        assert!(result.contains("Above"), "Got: {}", result);
        assert!(result.contains("Below"), "Got: {}", result);
    }

    #[test]
    fn test_preprocess_multiline_cells() {
        let input = "\
+-----+---------------------------+
| No. | Description               |
+=====+===========================+
| 1   | **First item**            |
|     | With extra detail         |
+-----+---------------------------+";
        let result = preprocess_grid_tables(input);
        // Multiline content should be joined with space
        assert!(
            result.contains("**First item** With extra detail"),
            "Got: {}",
            result
        );
    }

    #[test]
    fn test_preprocess_preserves_non_table_content() {
        let input = "# Title\n\nSome paragraph.\n\n- list item";
        let result = preprocess_grid_tables(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_preprocess_no_header_separator() {
        let input = "\
+-----+-----+
| A   | B   |
+-----+-----+
| 1   | 2   |
+-----+-----+";
        let result = preprocess_grid_tables(input);
        // First row becomes header
        assert!(result.contains("| A | B |"), "Got: {}", result);
        assert!(result.contains("| --- | --- |"), "Got: {}", result);
        assert!(result.contains("| 1 | 2 |"), "Got: {}", result);
    }
}
