//! Console table rendering with Unicode/CJK-aware alignment, truncation,
//! and ANSI color support.
//!
//! # Example
//!
//! ```
//! use console_table::{Align, Style, Table};
//!
//! let rows = vec![
//!     vec!["#".to_string(), "source".to_string(), "latency".to_string()],
//!     vec!["1".to_string(), "rsproxy".to_string(), "57 ms".to_string()],
//!     vec!["2".to_string(), "tuna".to_string(), "16 ms".to_string()],
//! ];
//!
//! let table = Table::new(rows)
//!     .style(Style::Boxed)
//!     .aligns(vec![Align::Right, Align::Left, Align::Auto]);
//!
//! println!("{}", table.render());
//! ```

#![warn(missing_docs)]

use unicode_width::UnicodeWidthStr;

// ============================================================
// Public types
// ============================================================

/// Border style of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Header row, a single separator, then data rows.
    Simple,
    /// Full box drawing: top, header separator, row separators, bottom.
    Boxed,
}

/// Horizontal alignment of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Left align.
    Left,
    /// Right align.
    Right,
    /// Right align if the first whitespace-separated token parses as `f64`,
    /// otherwise left align.
    Auto,
}

/// ANSI foreground color.
///
/// The first eight are the standard colors, the `Bright*` variants are
/// the corresponding high-intensity colors. Mapping to ANSI codes:
/// `Black`=30 ... `White`=37, `BrightBlack`=90 ... `BrightWhite`=97.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// ANSI color 30.
    Black,
    /// ANSI color 31.
    Red,
    /// ANSI color 32.
    Green,
    /// ANSI color 33.
    Yellow,
    /// ANSI color 34.
    Blue,
    /// ANSI color 35.
    Magenta,
    /// ANSI color 36.
    Cyan,
    /// ANSI color 37.
    White,
    /// ANSI color 90 (high-intensity black / gray).
    BrightBlack,
    /// ANSI color 91.
    BrightRed,
    /// ANSI color 92.
    BrightGreen,
    /// ANSI color 93.
    BrightYellow,
    /// ANSI color 94.
    BrightBlue,
    /// ANSI color 95.
    BrightMagenta,
    /// ANSI color 96.
    BrightCyan,
    /// ANSI color 97.
    BrightWhite,
}

impl Color {
    /// The ANSI foreground color code.
    fn fg_code(self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
        }
    }
}

/// Table configuration.
#[derive(Debug, Clone)]
pub struct Table {
    rows: Vec<Vec<String>>,
    style: Style,
    aligns: Vec<Align>,
    /// Maximum display width of a data cell (padding excluded).
    /// `None` means no truncation.
    max_cell_width: Option<usize>,
    /// Foreground color for the header row.
    header_color: Option<Color>,
    /// Foreground color for data rows.
    body_color: Option<Color>,
}

impl Table {
    /// Create a table from `rows`. The first row is treated as the header.
    ///
    /// Rows may have different lengths; missing cells are treated as empty.
    pub fn new(rows: Vec<Vec<String>>) -> Self {
        Self {
            rows,
            style: Style::Simple,
            aligns: Vec::new(),
            max_cell_width: None,
            header_color: None,
            body_color: None,
        }
    }

    /// Set the border style. Default is [`Style::Simple`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set per-column alignment. Missing columns default to [`Align::Left`].
    pub fn aligns(mut self, aligns: Vec<Align>) -> Self {
        self.aligns = aligns;
        self
    }

    /// Truncate data cells to at most `width` display columns, appending
    /// ASCII `...` on overflow. The header is never truncated.
    pub fn max_cell_width(mut self, width: usize) -> Self {
        self.max_cell_width = Some(width);
        self
    }

    /// Set the header foreground color. Default is no color.
    pub fn header_color(mut self, color: Color) -> Self {
        self.header_color = Some(color);
        self
    }

    /// Set the data-row foreground color. Default is no color.
    pub fn body_color(mut self, color: Color) -> Self {
        self.body_color = Some(color);
        self
    }

    fn align_of(&self, col: usize) -> Align {
        self.aligns.get(col).copied().unwrap_or(Align::Left)
    }

    /// Number of columns: the maximum row length.
    fn column_count(&self) -> usize {
        self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
    }

    /// Prepare a cell for rendering.
    ///
    /// Steps: strip ANSI escape sequences, then truncate to
    /// `max_cell_width`. Returns `(final_text, display_width)`.
    ///
    /// Both width computation and rendering call this, so they always see
    /// the same content.
    fn prepare_cell(&self, content: &str) -> (String, usize) {
        let stripped = strip_ansi(content);
        let truncated = match self.max_cell_width {
            Some(max) => truncate_visible(&stripped, max),
            None => stripped,
        };
        let w = display_width(&truncated);
        (truncated, w)
    }

    /// Compute the display width of each column.
    ///
    /// Each column is `max(display_width of all cells in that column,
    /// including the header) + 2`, where `+2` reserves one space of padding
    /// on each side. Data cells are prepared first via [`Self::prepare_cell`].
    fn column_widths(&self) -> Vec<usize> {
        let cols = self.column_count();
        let mut widths = vec![0usize; cols];
        for (r, row) in self.rows.iter().enumerate() {
            for (i, cell) in row.iter().enumerate() {
                let w = if r == 0 {
                    // Header: never truncated, but ANSI is still stripped
                    // before measuring.
                    display_width(cell)
                } else {
                    // Data rows go through the same preparation path.
                    let (_, w) = self.prepare_cell(cell);
                    w
                };
                if w > widths[i] {
                    widths[i] = w;
                }
            }
        }
        // Reserve one space of padding on each side.
        widths.into_iter().map(|w| w + 2).collect()
    }

    /// Render one data cell: one space of padding on each side, then
    /// alignment, then `body_color`.
    fn render_cell(&self, content: &str, width: usize, align: Align) -> String {
        let (text, vis) = self.prepare_cell(content);

        // `width` already includes one space of padding on each side.
        let inner = width.saturating_sub(2);
        let pad = inner.saturating_sub(vis);

        let aligned = match align {
            Align::Left => format!("{}{}", text, " ".repeat(pad)),
            Align::Right => format!("{}{}", " ".repeat(pad), text),
            Align::Auto => {
                // Decide numeric-ness from the original content (ANSI stripped).
                if is_numeric_like(&strip_ansi(content)) {
                    format!("{}{}", " ".repeat(pad), text)
                } else {
                    format!("{}{}", text, " ".repeat(pad))
                }
            }
        };

        let padded = format!(" {} ", aligned);
        colorize(&padded, self.body_color)
    }

    /// Render one header cell: left aligned, never truncated, one space of
    /// padding on each side, then `header_color`.
    fn render_header_cell(&self, content: &str, width: usize) -> String {
        let stripped = strip_ansi(content);
        let vis = display_width(&stripped);
        let inner = width.saturating_sub(2);
        let pad = inner.saturating_sub(vis);
        let padded = format!(" {}{} ", stripped, " ".repeat(pad));
        colorize(&padded, self.header_color)
    }

    /// Render one row (without the leading and trailing `│`).
    fn render_row(&self, row: &[String], widths: &[usize], is_header: bool) -> String {
        let mut cells = Vec::with_capacity(widths.len());
        for (i, w) in widths.iter().enumerate() {
            let content = row.get(i).map(String::as_str).unwrap_or("");
            if is_header {
                cells.push(self.render_header_cell(content, *w));
            } else {
                let align = self.align_of(i);
                cells.push(self.render_cell(content, *w, align));
            }
        }
        cells.join("│")
    }

    /// Build a horizontal line, e.g. `├───┼───┤`.
    fn horizontal(widths: &[usize], left: &str, mid: &str, right: &str, fill: &str) -> String {
        let segments: Vec<String> = widths.iter().map(|w| fill.repeat(*w)).collect();
        format!("{}{}{}", left, segments.join(mid), right)
    }

    /// Render the table to a `String`. Every line ends with `\n`.
    pub fn render(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }
        let widths = self.column_widths();
        let mut out = String::new();

        match self.style {
            Style::Simple => {
                out.push('│');
                out.push_str(&self.render_row(&self.rows[0], &widths, true));
                out.push_str("│\n");
                out.push_str(&Self::horizontal(&widths, "├", "┼", "┤", "─"));
                out.push('\n');
                for row in &self.rows[1..] {
                    out.push('│');
                    out.push_str(&self.render_row(row, &widths, false));
                    out.push_str("│\n");
                }
            }
            Style::Boxed => {
                out.push_str(&Self::horizontal(&widths, "┌", "┬", "┐", "─"));
                out.push('\n');
                out.push('│');
                out.push_str(&self.render_row(&self.rows[0], &widths, true));
                out.push_str("│\n");
                out.push_str(&Self::horizontal(&widths, "├", "┼", "┤", "─"));
                out.push('\n');
                for (i, row) in self.rows[1..].iter().enumerate() {
                    out.push('│');
                    out.push_str(&self.render_row(row, &widths, false));
                    out.push_str("│\n");
                    if i + 1 < self.rows.len() - 1 {
                        out.push_str(&Self::horizontal(&widths, "├", "┼", "┤", "─"));
                        out.push('\n');
                    }
                }
                out.push_str(&Self::horizontal(&widths, "└", "┴", "┘", "─"));
                out.push('\n');
            }
        }
        out
    }
}

/// Convenience function: render a table directly.
pub fn render_table(rows: Vec<Vec<String>>, style: Style) -> String {
    Table::new(rows).style(style).render()
}

// ============================================================
// Color
// ============================================================

/// Wrap `s` in an ANSI foreground color. `None` returns `s` unchanged.
fn colorize(s: &str, color: Option<Color>) -> String {
    match color {
        Some(c) => format!("\x1b[{}m{}\x1b[0m", c.fg_code(), s),
        None => s.to_string(),
    }
}

// ============================================================
// Width
// ============================================================

/// Terminal display width of `s`, ignoring ANSI escape sequences.
pub fn display_width(s: &str) -> usize {
    let stripped = strip_ansi(s);
    UnicodeWidthStr::width(stripped.as_str())
}

/// Pad `s` on the right with spaces up to `width` display columns.
pub fn pad_right_with_spaces(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + (width - w));
    out.push_str(s);
    out.push_str(&" ".repeat(width - w));
    out
}

/// Pad `s` on the left with spaces up to `width` display columns.
pub fn pad_left_with_spaces(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + (width - w));
    out.push_str(&" ".repeat(width - w));
    out.push_str(s);
    out
}

// ============================================================
// ANSI
// ============================================================

/// Strip CSI-style ANSI escape sequences (e.g. `\x1b[...m`) from `s`.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ============================================================
// Truncation
// ============================================================

/// Truncate `s` to at most `max_width` display columns, appending ASCII
/// `...` (3 columns) on overflow.
///
/// ASCII `...` is used instead of `…` (U+2026) because the single-character
/// ellipsis renders as 1 or 2 columns depending on terminal and font, which
/// breaks alignment. ASCII `...` is always exactly 3 columns.
///
/// ANSI escape sequences are preserved (they do not consume width), but
/// color closure after truncation is not guaranteed. Prefer stripping ANSI
/// before calling this function.
pub fn truncate_visible(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let vis = display_width(s);
    if vis <= max_width {
        return s.to_string();
    }
    let suffix = "...";
    let suffix_w = 3usize;
    if max_width <= suffix_w {
        // Not enough room for any content; show the suffix only.
        return suffix.to_string();
    }
    let target = max_width - suffix_w;
    let mut out = String::new();
    let mut used = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push(c);
            if chars.peek() == Some(&'[') {
                out.push(chars.next().unwrap());
                for c2 in chars.by_ref() {
                    out.push(c2);
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        let cw = UnicodeWidthStr::width(c.to_string().as_str());
        if used + cw > target {
            break;
        }
        out.push(c);
        used += cw;
    }
    out.push_str(suffix);
    out
}

// ============================================================
// Numeric heuristic
// ============================================================

/// Return `true` if `s` looks like a number, for [`Align::Auto`].
///
/// Rule: take the first whitespace-separated token and try to parse it as
/// `f64`. For example, `"57 ms"` -> `"57"` -> `true`; `"失败"` -> `false`;
/// `"-3.14"` -> `true`.
pub fn is_numeric_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let head = s.split_whitespace().next().unwrap_or("");
    head.parse::<f64>().is_ok()
}
