//! Console table rendering with Unicode/CJK-aware alignment, truncation,
//! and ANSI color support.
//!
//! # Ambiguous-width characters
//!
//! Characters like `Φ` (U+03A6) have Unicode `East_Asian_Width=Ambiguous`.
//! Terminals render them as 1 or 2 columns depending on locale and terminal
//! settings. `unicode_width` reports them as 1. To align with the terminal,
//! this crate counts them as 2 when it believes the output target is a CJK
//! context, and 1 otherwise.
//!
//! **Default behavior**: ambiguous characters are counted as **2 columns**
//! (`AmbiguousWidth::Wide`). This matches CJK terminals, which are the
//! primary target of this crate. If your terminal renders `Φ` as 1 column,
//! call [`Table::cjk_context(false)`] or [`Table::ambiguous_width`] to
//! override.
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
// Ambiguous-width context
// ============================================================

/// How to treat Unicode `East_Asian_Width=Ambiguous` characters.
///
/// Examples: `Φ`, `±`, `×`, `Ⅱ`, `①`. These are 1 column in most Western
/// terminals and 2 columns in most CJK terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbiguousWidth {
    /// Count ambiguous characters as 1 column (Western terminal default).
    Narrow,
    /// Count ambiguous characters as 2 columns (CJK terminal default).
    Wide,
    /// Decide from environment variables at render time.
    Auto,
}

impl AmbiguousWidth {
    /// Resolve to a concrete width using the environment.
    fn resolve(self) -> usize {
        match self {
            AmbiguousWidth::Narrow => 1,
            AmbiguousWidth::Wide => 2,
            AmbiguousWidth::Auto => {
                if is_cjk_locale() {
                    2
                } else {
                    1
                }
            }
        }
    }
}

impl Default for AmbiguousWidth {
    /// Default is [`AmbiguousWidth::Wide`], matching CJK terminals.
    ///
    /// Rationale: the crate targets CJK output by default. `Auto` guesses
    /// from `LANG`/`LC_*`, but environment locale does not necessarily match
    /// how the terminal renders ambiguous characters. A fixed default is
    /// more predictable; callers can override per [`Table`].
    fn default() -> Self {
        AmbiguousWidth::Wide
    }
}

/// Heuristic: is the current process likely targeting a CJK terminal?
///
/// Checks `LC_ALL`, `LC_CTYPE`, `LANG` for a CJK locale prefix
/// (`zh`, `ja`, `ko`). If nothing matches, returns `false` (narrow).
fn is_cjk_locale() -> bool {
    for key in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            let v = v.to_ascii_lowercase();
            if v.starts_with("zh")
                || v.starts_with("ja")
                || v.starts_with("ko")
                || v.contains("utf-8.zh")
                || v.contains("utf-8.ja")
                || v.contains("utf-8.ko")
            {
                return true;
            }
        }
    }
    false
}

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
    max_cell_width: Option<usize>,
    header_color: Option<Color>,
    body_color: Option<Color>,
    ambiguous_width: AmbiguousWidth,
}

impl Table {
    /// Create a table from `rows`. The first row is treated as the header.
    ///
    /// Ambiguous-width characters default to [`AmbiguousWidth::Wide`]
    /// (2 columns). Override with [`Table::ambiguous_width`] or
    /// [`Table::cjk_context`].
    pub fn new(rows: Vec<Vec<String>>) -> Self {
        Self {
            rows,
            style: Style::Simple,
            aligns: Vec::new(),
            max_cell_width: None,
            header_color: None,
            body_color: None,
            ambiguous_width: AmbiguousWidth::default(),
        }
    }

    /// Create a table from an `ndarray::Array2<String>`.
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

    /// Truncate data cells to at most `width` display columns.
    pub fn max_cell_width(mut self, width: usize) -> Self {
        self.max_cell_width = Some(width);
        self
    }

    /// Set the header foreground color.
    pub fn header_color(mut self, color: Color) -> Self {
        self.header_color = Some(color);
        self
    }

    /// Set the data-row foreground color.
    pub fn body_color(mut self, color: Color) -> Self {
        self.body_color = Some(color);
        self
    }

    /// Set how ambiguous-width characters (`Φ`, `±`, `Ⅱ`, ...) are counted.
    ///
    /// Default is [`AmbiguousWidth::Wide`] (2 columns, CJK terminals).
    /// Use [`AmbiguousWidth::Narrow`] for Western terminals, or
    /// [`AmbiguousWidth::Auto`] to guess from locale environment variables.
    pub fn ambiguous_width(mut self, mode: AmbiguousWidth) -> Self {
        self.ambiguous_width = mode;
        self
    }

    /// Convenience: set [`AmbiguousWidth::Narrow`] or [`AmbiguousWidth::Wide`].
    pub fn cjk_context(self, cjk: bool) -> Self {
        let mode = if cjk {
            AmbiguousWidth::Wide
        } else {
            AmbiguousWidth::Narrow
        };
        self.ambiguous_width(mode)
    }

    /// Render the table to a `String`. Every line ends with `\n`.
    pub fn render(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }
        let aw = self.ambiguous_width.resolve();
        let widths = self.column_widths(aw);
        let mut out = String::new();

        match self.style {
            Style::Simple => {
                out.push('│');
                out.push_str(&self.render_row(&self.rows[0], &widths, true, aw));
                out.push_str("│\n");
                out.push_str(&Self::horizontal(&widths, "├", "┼", "┤", "─"));
                out.push('\n');
                for row in &self.rows[1..] {
                    out.push('│');
                    out.push_str(&self.render_row(row, &widths, false, aw));
                    out.push_str("│\n");
                }
            }
            Style::Boxed => {
                out.push_str(&Self::horizontal(&widths, "┌", "┬", "┐", "─"));
                out.push('\n');
                out.push('│');
                out.push_str(&self.render_row(&self.rows[0], &widths, true, aw));
                out.push_str("│\n");
                out.push_str(&Self::horizontal(&widths, "├", "┼", "┤", "─"));
                out.push('\n');
                for (i, row) in self.rows[1..].iter().enumerate() {
                    out.push('│');
                    out.push_str(&self.render_row(row, &widths, false, aw));
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

    fn column_count(&self) -> usize {
        self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
    }

    fn prepare_cell(&self, content: &str, aw: usize) -> (String, usize) {
        let stripped = strip_ansi(content);
        let trimmed = stripped.trim();
        let truncated = match self.max_cell_width {
            Some(max) => truncate_visible_with(trimmed, max, aw),
            None => trimmed.to_string(),
        };
        let w = display_width_with(&truncated, aw);
        (truncated, w)
    }

    fn column_widths(&self, aw: usize) -> Vec<usize> {
        let cols = self.column_count();
        let mut widths = vec![0usize; cols];
        for (r, row) in self.rows.iter().enumerate() {
            for (i, cell) in row.iter().enumerate() {
                let w = if r == 0 {
                    display_width_with(cell, aw)
                } else {
                    let (_, w) = self.prepare_cell(cell, aw);
                    w
                };
                if w > widths[i] {
                    widths[i] = w;
                }
            }
        }
        widths.into_iter().map(|w| w + 2).collect()
    }

    fn render_cell(&self, content: &str, width: usize, align: Align, aw: usize) -> String {
        let (text, vis) = self.prepare_cell(content, aw);
        let inner = width.saturating_sub(2);
        let pad = inner.saturating_sub(vis);

        let aligned = match align {
            Align::Left => format!("{}{}", text, " ".repeat(pad)),
            Align::Right => format!("{}{}", " ".repeat(pad), text),
            Align::Auto => {
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

    fn render_header_cell(&self, content: &str, width: usize, aw: usize) -> String {
        let stripped = strip_ansi(content);
        let vis = display_width_with(&stripped, aw);
        let inner = width.saturating_sub(2);
        let pad = inner.saturating_sub(vis);
        let padded = format!(" {}{} ", stripped, " ".repeat(pad));
        colorize(&padded, self.header_color)
    }

    fn render_row(
        &self,
        row: &[String],
        widths: &[usize],
        is_header: bool,
        aw: usize,
    ) -> String {
        let mut cells = Vec::with_capacity(widths.len());
        for (i, w) in widths.iter().enumerate() {
            let content = row.get(i).map(String::as_str).unwrap_or("");
            if is_header {
                cells.push(self.render_header_cell(content, *w, aw));
            } else {
                let align = self.align_of(i);
                cells.push(self.render_cell(content, *w, align, aw));
            }
        }
        cells.join("│")
    }

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

fn colorize(s: &str, color: Option<Color>) -> String {
    match color {
        Some(c) => format!("\x1b[{}m{}\x1b[0m", c.fg_code(), s),
        None => s.to_string(),
    }
}

// ============================================================
// Width
// ============================================================

/// Is `c` in the Unicode `East_Asian_Width=Ambiguous` category?
fn is_ambiguous(c: char) -> bool {
    matches!(
        c,
        '\u{0391}'..='\u{03A9}'
        | '\u{03B1}'..='\u{03C9}'
        | '±' | '×' | '÷' | '≤' | '≥' | '≠' | '≈' | '∞' | '√'
        | '→' | '←' | '↑' | '↓' | '↔'
        | 'Ⅰ' | 'Ⅱ' | 'Ⅲ' | 'Ⅳ' | 'Ⅴ' | 'Ⅵ' | 'Ⅶ' | 'Ⅷ' | 'Ⅸ' | 'Ⅹ'
        | 'ⅰ' | 'ⅱ' | 'ⅲ' | 'ⅳ' | 'ⅴ' | 'ⅵ' | 'ⅶ' | 'ⅷ' | 'ⅸ' | 'ⅹ'
        | '①'..='⑳'
    )
}

/// Width of a single `char`, treating ambiguous characters as `aw` columns.
fn char_width(c: char, aw: usize) -> usize {
    if is_ambiguous(c) {
        aw
    } else {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }
}

/// Display width of `s` with a given ambiguous-character width `aw`.
fn display_width_with(s: &str, aw: usize) -> usize {
    let stripped = strip_ansi(s);
    stripped.chars().map(|c| char_width(c, aw)).sum()
}

/// Terminal display width of `s`, ignoring ANSI escape sequences.
///
/// Uses [`AmbiguousWidth::default`] (currently `Wide`).
pub fn display_width(s: &str) -> usize {
    display_width_with(s, AmbiguousWidth::default().resolve())
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
                chars.next();
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

/// Truncate `s` to at most `max_width` display columns.
///
/// Uses [`AmbiguousWidth::default`] (currently `Wide`). For an explicit
/// choice, use [`truncate_visible_with`].
pub fn truncate_visible(s: &str, max_width: usize) -> String {
    truncate_visible_with(s, max_width, AmbiguousWidth::default().resolve())
}

/// Truncate `s` to at most `max_width` display columns, using `aw` for
/// ambiguous characters.
pub fn truncate_visible_with(s: &str, max_width: usize, aw: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let vis = display_width_with(s, aw);
    if vis <= max_width {
        return s.to_string();
    }
    let suffix = "...";
    let suffix_w = 3usize;
    if max_width <= suffix_w {
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
        let cw = char_width(c, aw);
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
        let s = "Φ170";
        assert_eq!(UnicodeWidthStr::width(s), 4);
        assert_eq!(UnicodeWidthStr::width_cjk(s), 4);
        assert_eq!(display_width_with(s, 2), 5);
        assert_eq!(display_width_with(s, 1), 4);
    }

    #[test]
    fn ambiguous_width_respects_context() {
        assert_eq!(display_width_with("Φ", 2), 2);
        assert_eq!(display_width_with("Φ", 1), 1);
        assert_eq!(display_width_with("Φ170", 2), 5);
        assert_eq!(display_width_with("Φ170", 1), 4);
        assert_eq!(display_width_with("Φ8", 2), 3);
        assert_eq!(display_width_with("Φ16  III", 2), 9);
        assert_eq!(display_width_with("Φ16  III", 1), 8);
    }

    #[test]
    fn default_ambiguous_width_is_wide() {
        // 默认是 Wide。
        assert_eq!(AmbiguousWidth::default(), AmbiguousWidth::Wide);
        assert_eq!(display_width("Φ"), 2);
        assert_eq!(display_width("Φ170"), 5);
    }

    #[test]
    fn display_width_ascii_unchanged() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("170CD1"), 6);
        assert_eq!(display_width("NUT-1"), 5);
    }

    #[test]
    fn display_width_cjk_unchanged() {
        assert_eq!(display_width("电杆"), 4);
        assert_eq!(display_width("名称"), 4);
        assert_eq!(display_width("电阻值设计定"), 12);
    }

    #[test]
    fn display_width_strips_ansi() {
        let colored = "\x1b[93m电杆\x1b[0m";
        assert_eq!(display_width(colored), 4);
    }

    #[test]
    fn phi_rows_align_default_wide() {
        // 默认（Wide）下，含 Φ 的行应和其它行对齐。
        let rows = vec![
            vec!["名称".into(), "规格".into(), "材料".into()],
            vec!["电杆".into(), "Φ170".into(), "".into()],
            vec!["横担".into(), "03D103-133".into(), "2II2".into()],
            vec!["接地线".into(), "Φ8".into(), "".into()],
            vec!["拉线棒".into(), "03D103-187".into(), "Φ16  III".into()],
        ];
        let out = Table::new(rows).style(Style::Boxed).render();
        for line in out.lines() {
            if line.starts_with('│') {
                assert_eq!(line.matches('│').count(), 4, "行: {:?}", line);
            }
        }
    }

    #[test]
    fn phi_rows_align_narrow() {
        let rows = vec![
            vec!["名称".into(), "规格".into(), "材料".into()],
            vec!["电杆".into(), "Φ170".into(), "".into()],
            vec!["横担".into(), "03D103-133".into(), "2II2".into()],
            vec!["接地线".into(), "Φ8".into(), "".into()],
            vec!["拉线棒".into(), "03D103-187".into(), "Φ16  III".into()],
        ];
        let out = Table::new(rows)
            .style(Style::Boxed)
            .ambiguous_width(AmbiguousWidth::Narrow)
            .render();
        for line in out.lines() {
            if line.starts_with('│') {
                assert_eq!(line.matches('│').count(), 4, "行: {:?}", line);
            }
        }
    }

    #[test]
    fn truncate_phi_consistent_wide() {
        let s = "Φ170CD1XYZ";
        let t = truncate_visible_with(s, 6, 2);
        assert_eq!(t, "Φ1...");
        assert_eq!(display_width_with(&t, 2), 6);
    }

    #[test]
    fn truncate_phi_consistent_narrow() {
        let s = "Φ170CD1XYZ";
        let t = truncate_visible_with(s, 6, 1);
        assert_eq!(t, "Φ17...");
        assert_eq!(display_width_with(&t, 1), 6);
    }

    #[test]
    fn truncate_visible_two_arg_uses_default_wide() {
        // 两参数版本用默认（Wide）：Φ(2) + 1(1) = 3 → "Φ1..."
        let t = truncate_visible("Φ170CD1XYZ", 6);
        assert_eq!(t, "Φ1...");
    }

    #[test]
    fn truncate_ascii_unchanged() {
        let s = "abcdefghij";
        let t = truncate_visible_with(s, 7, 2);
        assert_eq!(t, "abcd...");
        assert_eq!(display_width_with(&t, 2), 7);
    }

    #[test]
    fn truncate_no_overflow_returns_original() {
        let s = "Φ8";
        let t = truncate_visible_with(s, 10, 2);
        assert_eq!(t, "Φ8");
    }

    #[test]
    fn cjk_context_builder_sets_mode() {
        let t = Table::new(vec![vec!["a".into()]]).cjk_context(true);
        assert_eq!(t.ambiguous_width, AmbiguousWidth::Wide);
        let t = Table::new(vec![vec!["a".into()]]).cjk_context(false);
        assert_eq!(t.ambiguous_width, AmbiguousWidth::Narrow);
    }
}