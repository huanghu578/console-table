# console-table

[![Crates.io](https://img.shields.io/crates/v/console-table.svg)](https://crates.io/crates/console-table)
[![Docs.rs](https://docs.rs/console-table/badge.svg)](https://docs.rs/console-table)
[![License](https://img.shields.io/crates/l/console-table.svg)](#license)

A small, dependency-light Rust library for rendering aligned console tables.

It is Unicode-aware (CJK characters count as 2 columns), supports optional
per-cell truncation, ANSI foreground colors, and two border styles.

---

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Before and after](#before-and-after)
- [API overview](#api-overview)
- [Design notes](#design-notes)
- [Windows note](#windows-note)
- [FAQ](#faq)
- [License](#license)

---

## Features

- **Unicode / CJK-aware alignment** — uses [`unicode-width`] so that Chinese,
  Japanese, Korean, fullwidth, and zero-width characters are measured by their
  real terminal display width, not by `char` count.
- **Two border styles** — `Style::Simple` (header separator only) and
  `Style::Boxed` (full box drawing).
- **Per-column alignment** — `Align::Left`, `Align::Right`, or `Align::Auto`
  (numbers right-aligned, everything else left-aligned).
- **Truncation** — cap any data cell at `max_cell_width`; overflow becomes
  ASCII `...`. Headers are never truncated.
- **ANSI colors** — optional `header_color` and `body_color` from a small
  built-in palette.
- **ANSI-safe width math** — existing ANSI sequences inside cell text are
  stripped before width is measured, so colored input does not break layout.
- **Only one dependency** — `unicode-width`.

[`unicode-width`]: https://crates.io/crates/unicode-width

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
console-table = "0.1"
```

---

## Quick start

```rust
use console_table::{Align, Color, Style, Table};

fn main() {
    let rows = vec![
        vec!["#".into(), "source".into(), "latency".into()],
        vec!["1".into(), "rsproxy (ByteDance)".into(), "57 ms".into()],
        vec!["2".into(), "tuna (Tsinghua)".into(), "16 ms".into()],
        vec!["8".into(), "cqu (Chongqing)".into(), "failed".into()],
    ];

    let table = Table::new(rows)
        .style(Style::Boxed)
        .aligns(vec![Align::Right, Align::Left, Align::Auto])
        .max_cell_width(40)
        .header_color(Color::BrightCyan)
        .body_color(Color::White);

    println!("{}", table.render());
}
```

Output (colors not shown):

```text
┌──────┬─────────────────────┬──────────┐
│ #    │ source              │ latency  │
├──────┼─────────────────────┼──────────┤
│    1 │ rsproxy (ByteDance) │   57 ms  │
│    2 │ tuna (Tsinghua)     │   16 ms  │
│    8 │ cqu (Chongqing)     │ failed   │
└──────┴─────────────────────┴──────────┘
```

---

## Before and after

The same CJK data, rendered three ways. The first two use plain `format!` and
break or bloat on Chinese text; the third uses `console-table` and stays
aligned with almost no boilerplate.

The runnable version of all three is in
[`examples/compare.rs`](examples/compare.rs).

```rust
let headers = ["序号", "镜像源", "平均延迟", "可用", "serde最新版"];
let data = [
    ["1", "rsproxy (字节跳动)", "57 ms", "是", "1.0.229"],
    ["2", "tuna (清华)", "16 ms", "是", "1.0.229"],
    ["8", "cqu (重大)", "失败", "否", "-"],
];
```

### Before — manual `format!`

The common first attempt. `{:<20}` pads by `char` count, so Chinese characters
(which occupy 2 terminal columns each) collapse the columns.

```rust
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
```

Output:

```text
序号 镜像源               平均延迟   可用 serde最新版
------------------------------------------------------------
1    rsproxy (字节跳动)   57 ms      是   1.0.229
2    tuna (清华)          16 ms      是   1.0.229
8    cqu (重大)           失败       否   -
```

Columns drift because `"镜像源"` is 3 `char`s but 6 columns wide.

### Before — manual width handling

To fix it by hand, you need your own display-width function, a fixed width
table, and manual padding. It works, but it is a lot of boilerplate, and every
new column means editing several places.

```rust
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
```

Output:

```text
序号    镜像源                    平均延迟      可用    serde最新版
----------------------------------------------------------------------
1       rsproxy (字节跳动)        57 ms         是      1.0.229
2       tuna (清华)               16 ms         是      1.0.229
8       cqu (重大)                失败          否      -
```

Aligned, but no borders, no per-column alignment, no truncation, no colors.

### After — `console-table`

```rust
use console_table::{Align, Color, Style, Table};

let rows = vec![
    vec!["序号".into(), "镜像源".into(), "平均延迟".into(), "可用".into(), "serde最新版".into()],
    vec!["1".into(), "rsproxy (字节跳动)".into(), "57 ms".into(), "是".into(), "1.0.229".into()],
    vec!["2".into(), "tuna (清华)".into(), "16 ms".into(), "是".into(), "1.0.229".into()],
    vec!["8".into(), "cqu (重大)".into(), "失败".into(), "否".into(), "-".into()],
];

let table = Table::new(rows)
    .style(Style::Boxed)
    .aligns(vec![
        Align::Right, // 序号
        Align::Left,  // 镜像源
        Align::Auto,  // 平均延迟：数字右对齐，"失败" 左对齐
        Align::Left,  // 可用
        Align::Left,  // serde最新版
    ])
    .header_color(Color::BrightCyan)
    .body_color(Color::White);

println!("{}", table.render());
```

Output:

```text
┌──────┬────────────────────┬──────────┬──────┬─────────────┐
│ 序号 │ 镜像源             │ 平均延迟 │ 可用 │ serde最新版 │
├──────┼────────────────────┼──────────┼──────┼─────────────┤
│    1 │ rsproxy (字节跳动) │   57 ms  │ 是   │ 1.0.229     │
│    2 │ tuna (清华)        │   16 ms  │ 是   │ 1.0.229     │
│    8 │ cqu (重大)         │ 失败     │ 否   │ -           │
└──────┴────────────────────┴──────────┴──────┴─────────────┘
```

### After — with truncation

Long values are capped at `max_cell_width` and end with ASCII `...`.
Truncation never breaks alignment.

```rust
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
        Align::Right, Align::Left, Align::Auto,
        Align::Left, Align::Left, Align::Left,
    ])
    .max_cell_width(40)
    .header_color(Color::BrightCyan)
    .body_color(Color::White);

println!("{}", table.render());
```

Output:

```text
┌──────┬────────────────────┬──────────┬──────┬─────────────┬──────────────────────────────────────────┐
│ 序号 │ 镜像源             │ 平均延迟 │ 可用 │ serde最新版 │ 地址                                     │
├──────┼────────────────────┼──────────┼──────┼─────────────┼──────────────────────────────────────────┤
│    1 │ rsproxy (字节跳动) │   57 ms  │ 是   │ 1.0.229     │ sparse+https://rsproxy.cn/index/         │
│    2 │ tuna (清华)        │   16 ms  │ 是   │ 1.0.229     │ sparse+https://mirrors.tuna.tsinghua.edu… │
│    8 │ cqu (重大)         │ 失败     │ 否   │ -           │ sparse+https://mirrors.cqu.edu.cn/crates… │
└──────┴────────────────────┴──────────┴──────┴─────────────┴──────────────────────────────────────────┘
```

### Summary

| | manual `format!` | manual width fn | `console-table` |
| --- | --- | --- | --- |
| CJK-aware width | ✗ | ✓ | ✓ |
| Column widths auto-computed | ✗ | ✗ | ✓ |
| Borders | ✗ | ✗ | ✓ |
| Per-column alignment | ✗ | ✗ | ✓ |
| Truncation | ✗ | ✗ | ✓ |
| Colors | ✗ | ✗ | ✓ |
| Lines of code | ~6 | ~30 | ~10 |

---

## API overview

### `Table`

| Method | Description |
| --- | --- |
| `Table::new(rows)` | Create a table. The first row is treated as the header. Rows may have different lengths; missing cells are treated as empty. |
| `.style(Style)` | `Style::Simple` (default) or `Style::Boxed`. |
| `.aligns(Vec<Align>)` | Per-column alignment. Missing columns default to `Align::Left`. |
| `.max_cell_width(usize)` | Truncate data cells to at most this display width. Headers are not truncated. |
| `.header_color(Color)` | Foreground color for the header row. |
| `.body_color(Color)` | Foreground color for data rows. |
| `.render() -> String` | Render the table. Every line ends with `\n`. |

### `Style`

| Variant | Description |
| --- | --- |
| `Simple` | Header row, a single separator, then data rows. |
| `Boxed` | Full box drawing with top, header separator, row separators, and bottom border. |

### `Align`

| Variant | Description |
| --- | --- |
| `Left` | Left align. |
| `Right` | Right align. |
| `Auto` | Right align if the first whitespace-separated token parses as `f64`, otherwise left align. |

### `Color`

16 ANSI foreground colors:

- `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`
- `BrightBlack`, `BrightRed`, `BrightGreen`, `BrightYellow`,
  `BrightBlue`, `BrightMagenta`, `BrightCyan`, `BrightWhite`

### Free functions

| Function | Description |
| --- | --- |
| `render_table(rows, style)` | Convenience: `Table::new(rows).style(style).render()`. |
| `display_width(&str) -> usize` | Terminal display width, ignoring ANSI sequences. |
| `truncate_visible(&str, usize) -> String` | Truncate to a display width, appending ASCII `...`. |
| `strip_ansi(&str) -> String` | Remove CSI-style ANSI escape sequences. |
| `pad_right_with_spaces(&str, usize)` | Pad on the right by display width. |
| `pad_left_with_spaces(&str, usize)` | Pad on the left by display width. |
| `is_numeric_like(&str) -> bool` | Heuristic used by `Align::Auto`. |

---

## Design notes

- **Widths are measured, not assumed.** Every column is
  `max(display_width of all cells in that column, including the header) + 2`,
  where `+2` reserves one space of padding on each side.
- **`...` instead of `…`.** The single-character ellipsis `…` (U+2026) renders
  as 1 or 2 columns depending on the terminal and font, which breaks alignment.
  The ASCII `...` is always exactly 3 columns.
- **Colors are applied at render time.** Width computation strips ANSI
  sequences, so colored cells never widen a column.
- **Existing ANSI in cell text is stripped.** If a cell already contains color
  codes, they are removed and replaced by `body_color` (or no color). Do not mix
  per-cell colors with `body_color`.
- **Every rendered line has equal display width.** This includes the header,
  separators, and data rows, so the table stays rectangular even when the
  terminal renders wide characters at 2 columns.

---

## Windows note

Older `cmd.exe` consoles may not interpret ANSI escape sequences. Use
[Windows Terminal](https://aka.ms/terminal) or enable virtual terminal
processing in the console.

When redirecting output to a file, prefer disabling colors yourself:

```rust
use std::io::IsTerminal;

let table = Table::new(rows);
let table = if std::io::stdout().is_terminal() {
    table.header_color(Color::BrightCyan).body_color(Color::White)
} else {
    table
};
```

This keeps the redirected file free of `\x1b[...m` sequences.

---

## FAQ

**Why does my CJK table look misaligned in some terminals?**

Some terminals and fonts render `…` (U+2026) as 2 columns, and a few render it
as 1. This library uses ASCII `...` for truncation so the width is always 3.
If you still see misalignment, check that your terminal uses a font where
CJK characters are exactly twice the width of ASCII characters.

**Can I color individual cells?**

Not directly. `body_color` applies to the whole data row, and any ANSI codes
inside a cell are stripped before rendering. If you need per-cell colors,
render the row yourself or use a lower-level API.

**Why is `max_cell_width` applied to data cells only?**

Headers are typically short labels, and truncating them can hide useful
information. If you need header truncation, apply `truncate_visible` to the
header string before passing it to `Table::new`.

**Does the library write to stdout?**

No. `render()` returns a `String`. Printing is up to the caller.

---

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.