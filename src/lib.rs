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

use ndarray::Array2;
use unicode_width::UnicodeWidthChar;

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
    /// Create a table from `rows`.
    /// The first row is treated as the header.
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

    /// Create a table from an `ndarray::Array2<String>`.
    /// The first row is treated as the header.
    /// Rows may have different lengths; missing cells are treated as empty.
    pub fn from_array2(array: Array2<String>) -> Self {
        let rows = array
            .outer_iter()
            .map(|row| row.iter().cloned().collect())
            .collect();
        Self::new(rows)
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

impl Table {
    fn align_of(&self, col: usize) -> Align {
        self.aligns.get(col).copied().unwrap_or(Align::Left)
    }

    /// Number of columns: the maximum row length.
    fn column_count(&self) -> usize {
        self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
    }

    /// Prepare a cell for rendering.
    ///
    /// Steps: strip ANSI escape sequences, trim surrounding whitespace,
    /// then truncate to `max_cell_width`. Returns `(final_text, display_width)`.
    ///
    /// Both width computation and rendering call this, so they always see
    /// the same content.
    fn prepare_cell(&self, content: &str) -> (String, usize) {
        let stripped = strip_ansi(content);
        let trimmed = stripped.trim();
        let truncated = match self.max_cell_width {
            Some(max) => truncate_visible(trimmed, max),
            None => trimmed.to_string(),
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

/// Characters that the current terminal renders as 2 columns, but
/// `unicode_width` counts as 1 (even with `width_cjk`).
///
/// `Φ` (U+03A6) is the motivating example: `unicode_width` gives it width 1,
/// but CJK terminals render it 2 columns wide, causing misalignment.
/// Extend this list as new such characters appear in real data.
fn extra_wide(c: char) -> bool {
    matches!(
        c,
        'Φ' | 'φ'                 // 希腊字母 Phi
        | 'Ⅱ' | 'Ⅲ' | 'Ⅰ'       // 罗马数字
        | '±' | '×' | '÷'        // 数学符号
        | '≤' | '≥' | '≠'
        | '→' | '←' | '↑' | '↓'
        | '①'..='⑳'             // 带圈数字
    )
}

/// Display width of a single `char`, with [`extra_wide`] compensation.
fn char_width(c: char) -> usize {
    if extra_wide(c) {
        2
    } else {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }
}

/// Terminal display width of `s`, ignoring ANSI escape sequences.
///
/// Uses [`char_width`] per character so that characters which
/// `unicode_width` under-reports are counted correctly.
pub fn display_width(s: &str) -> usize {
    let stripped = strip_ansi(s);
    stripped.chars().map(char_width).sum()
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
        // Use `char_width` so compensation matches `display_width`.
        let cw = char_width(c);
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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn phi_is_ambiguous_width() {
        // `unicode_width` 把 `Φ` 算 1（即使 `width_cjk` 也是 1）。
        // 这是 crate 的行为；`display_width` 通过 `extra_wide` 补偿为 2。
        let s = "Φ170";
        assert_eq!(UnicodeWidthStr::width(s), 4, "crate 默认宽度：Φ(1)+170(3)=4");
        assert_eq!(
            UnicodeWidthStr::width_cjk(s),
            4,
            "crate cjk 宽度同样是 4"
        );
        assert_eq!(
            display_width(s),
            5,
            "display_width 补偿后：Φ(2)+170(3)=5"
        );
    }

    #[test]
    fn display_width_uses_cjk_for_phi() {
        assert_eq!(display_width("Φ"), 2);
        assert_eq!(display_width("Φ170"), 5);
        assert_eq!(display_width("Φ8"), 3);
        // "Φ16  III"：Φ(2) + 1(1) + 6(1) + 空格(1) + 空格(1) + I(1)*3 = 9
        assert_eq!(display_width("Φ16  III"), 9);
    }

    #[test]
    fn display_width_ascii_unchanged() {
        // ASCII 不受补偿影响。
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("170CD1"), 6);
        assert_eq!(display_width("NUT-1"), 5);
    }

    #[test]
    fn display_width_cjk_unchanged() {
        // 汉字本来就是 2，两种模式一致。
        assert_eq!(display_width("电杆"), 4);
        assert_eq!(display_width("名称"), 4);
        assert_eq!(display_width("电阻值设计定"), 12);
    }

    #[test]
    fn display_width_strips_ansi() {
        // ANSI 转义序列不计入宽度。
        let colored = "\x1b[93m电杆\x1b[0m";
        assert_eq!(display_width(colored), 4);
    }

    #[test]
    fn phi_rows_align() {
        // 构造含 Φ 和不含 Φ 的行，验证每行竖线数量一致。
        let rows = vec![
            vec!["名称".into(), "规格".into(), "材料".into()],
            vec!["电杆".into(), "Φ170".into(), "".into()],
            vec!["横担".into(), "03D103-133".into(), "2II2".into()],
            vec!["接地线".into(), "Φ8".into(), "".into()],
            vec!["拉线棒".into(), "03D103-187".into(), "Φ16  III".into()],
        ];
        let out = Table::new(rows).style(Style::Boxed).render();

        // 3 列 → 行首 1 + 列间 2 + 行尾 1 = 4 个 `│`。
        for line in out.lines() {
            if line.starts_with('│') {
                let n = line.matches('│').count();
                assert_eq!(n, 4, "行竖线数量异常: {:?}", line);
            }
        }
    }

    #[test]
    fn phi_align_matches_ascii_row() {
        // 含 Φ 的行和不含 Φ 的行，渲染后每列宽度应一致。
        let rows = vec![
            vec!["规格".into()],
            vec!["Φ170".into()],
            vec!["170C".into()], // 4 个 ASCII 字符，宽度 4
        ];
        let out = Table::new(rows).style(Style::Boxed).render();

        let data_lines: Vec<&str> = out
            .lines()
            .filter(|l| l.starts_with('│') && !l.contains("规格"))
            .collect();

        let widths: Vec<usize> = data_lines
            .iter()
            .map(|l| {
                let inner = l.trim_matches('│');
                display_width(inner)
            })
            .collect();

        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "含 Φ 的行与 ASCII 行宽度不一致: {:?}",
            widths
        );
    }

    #[test]
    fn truncate_phi_consistent() {
        // 允许 6 列，末尾 "..." 占 3 列，前缀最多 3 列。
        // Φ(2) + "1"(1) = 3 → "Φ1..."，总宽 6。
        let s = "Φ170CD1XYZ";
        let t = truncate_visible(s, 6);
        assert_eq!(t, "Φ1...");
        assert_eq!(display_width(&t), 6);
    }

    #[test]
    fn truncate_ascii_unchanged() {
        let s = "abcdefghij";
        let t = truncate_visible(s, 7);
        assert_eq!(t, "abcd...");
        assert_eq!(display_width(&t), 7);
    }

    #[test]
    fn truncate_no_overflow_returns_original() {
        let s = "Φ8";
        let t = truncate_visible(s, 10);
        assert_eq!(t, "Φ8");
    }
}