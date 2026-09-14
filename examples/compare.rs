//! Side-by-side comparison: manual `format!` alignment vs `console-table`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example compare
//! ```

use console_table::{Align, Color, Style, Table};

fn main() {
    let headers = ["序号", "镜像源", "平均延迟", "可用", "serde最新版"];
    let data = [
        ["1", "rsproxy (字节跳动)", "57 ms", "是", "1.0.229"],
        ["2", "tuna (清华)", "16 ms", "是", "1.0.229"],
        ["8", "cqu (重大)", "失败", "否", "-"],
    ];

    println!("=== 1. 手写 format!，按 char 个数补空格 ===\n");
    manual_align(&headers, &data);

    println!("\n=== 2. 手写 format!，汉字手动补 2 空格 ===\n");
    manual_align_cjk_aware(&headers, &data);

    println!("\n=== 3. console-table，Style::Boxed + 颜色 ===\n");
    let rows = build_rows(&headers, &data);
    let table = Table::new(rows.clone())
        .style(Style::Boxed)
        .aligns(vec![
            Align::Right,
            Align::Left,
            Align::Auto,
            Align::Left,
            Align::Left,
        ])
        .header_color(Color::BrightCyan)
        .body_color(Color::White);
    print!("{}", table.render());

    println!("\n=== 4. console-table，加地址列 + 截断 40 列 ===\n");
    let mut rows_with_url = rows.clone();
    rows_with_url[0].push("地址".to_string());
    let urls = [
        "sparse+https://rsproxy.cn/index/",
        "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/",
        "sparse+https://mirrors.cqu.edu.cn/crates.io-index/",
    ];
    for (i, url) in urls.iter().enumerate() {
        rows_with_url[i + 1].push((*url).to_string());
    }

    let table = Table::new(rows_with_url)
        .style(Style::Boxed)
        .aligns(vec![
            Align::Right,
            Align::Left,
            Align::Auto,
            Align::Left,
            Align::Left,
            Align::Left,
        ])
        .max_cell_width(40)
        .header_color(Color::BrightCyan)
        .body_color(Color::White);
    print!("{}", table.render());
}

/// Manual alignment: plain `{:<N}`, padding by `char` count.
/// Chinese characters count as 1 `char`, so columns collapse.
fn manual_align(headers: &[&str; 5], data: &[[&str; 5]; 3]) {
    println!(
        "{:<4} {:<20} {:<10} {:<6} {:<12}",
        headers[0], headers[1], headers[2], headers[3], headers[4]
    );
    println!("{}", "-".repeat(60));
    for row in data {
        println!(
            "{:<4} {:<20} {:<10} {:<6} {:<12}",
            row[0], row[1], row[2], row[3], row[4]
        );
    }
}

/// Manual alignment: pad each string by its display width.
/// Requires a hand-written width function and a fixed width table.
fn manual_align_cjk_aware(headers: &[&str; 5], data: &[[&str; 5]; 3]) {
    let widths = [4usize, 22, 12, 8, 14];

    let fmt_row = |cells: &[&str]| {
        let mut out = String::new();
        for (i, cell) in cells.iter().enumerate() {
            let w = display_width(cell);
            out.push_str(cell);
            if w < widths[i] {
                out.push_str(&" ".repeat(widths[i] - w));
            }
            out.push_str("  ");
        }
        out
    };

    println!("{}", fmt_row(headers));
    println!("{}", "-".repeat(70));
    for row in data {
        println!("{}", fmt_row(row));
    }
}

/// Minimal display width: CJK ranges count as 2 columns, everything else 1.
/// For demonstration only; production code should use `unicode-width`.
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (0x1100..=0x115F).contains(&cp)
                || (0x2E80..=0xA4CF).contains(&cp)
                || (0xAC00..=0xD7A3).contains(&cp)
                || (0xF900..=0xFAFF).contains(&cp)
                || (0xFE30..=0xFE4F).contains(&cp)
                || (0xFF00..=0xFF60).contains(&cp)
                || (0xFFE0..=0xFFE6).contains(&cp)
            {
                2
            } else {
                1
            }
        })
        .sum()
}

/// Convert `&[&str]` rows into `Vec<Vec<String>>` for `Table::new`.
fn build_rows(headers: &[&str; 5], data: &[[&str; 5]; 3]) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(data.len() + 1);
    rows.push(headers.iter().map(|s| s.to_string()).collect());
    for row in data {
        rows.push(row.iter().map(|s| s.to_string()).collect());
    }
    rows
}
