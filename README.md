# console-table

[![Crates.io](https://img.shields.io/crates/v/console-table.svg)](https://crates.io/crates/console-table)
[![Docs.rs](https://docs.rs/console-table/badge.svg)](https://docs.rs/console-table)
[![License](https://img.shields.io/crates/l/console-table.svg)](#license)

A small, dependency-light Rust library for rendering aligned console tables.

It is Unicode-aware (CJK characters count as 2 columns), handles
ambiguous-width characters like `Φ` predictably, supports optional per-cell
truncation, ANSI foreground colors, and two border styles.

---

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Ambiguous-width characters](#ambiguous-width-characters)
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
- **Predictable ambiguous-width handling** — characters like `Φ`, `±`, `Ⅱ`, `①`
  have Unicode `East_Asian_Width=Ambiguous` and render as 1 or 2 columns
  depending on the terminal. By default they are counted as **2 columns**,
  which matches CJK terminals. See
  [Ambiguous-width characters](#ambiguous-width-characters) to override.
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
console-table = "0.2"
Quick start
rust
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
Output (colors not shown):

text
┌──────┬─────────────────────┬──────────┐
│ #    │ source              │ latency  │
├──────┼─────────────────────┼──────────┤
│    1 │ rsproxy (ByteDance) │   57 ms  │
│    2 │ tuna (Tsinghua)     │   16 ms  │
│    8 │ cqu (Chongqing)     │ failed   │
└──────┴─────────────────────┴──────────┘
Ambiguous-width characters
Some characters have Unicode East_Asian_Width=Ambiguous. The most common one
in engineering data is Φ (U+03A6, Greek capital Phi), but the category also
includes ±, ×, ÷, ≤, Ⅱ, ① and others.

There is no single correct width for these characters. Terminals render
them as either 1 or 2 columns, depending on the terminal software, the font,
and user settings — not on the operating system language. For example:

Terminal	Φ renders as
Traditional CJK console (e.g. Windows cmd.exe with a CJK font)	2 columns
Windows Terminal (default narrow)	1 column
Windows Terminal (wide setting)	2 columns
VS Code integrated terminal	1 column
iTerm2 on macOS	1 column
Because this crate targets CJK output by default, ambiguous characters are
counted as 2 columns by default (AmbiguousWidth::Wide). This is a fixed
default, not a guess. If your terminal renders Φ as 1 column, override it:

rust
use console_table::{AmbiguousWidth, Table};

// Your terminal renders Φ as 1 column:
let table = Table::new(rows).cjk_context(false);
// equivalently:
let table = Table::new(rows).ambiguous_width(AmbiguousWidth::Narrow);
The three modes
Mode	Φ counted as	When to use
AmbiguousWidth::Wide (default)	2 columns	CJK terminals, Windows cmd.exe with a CJK font, most Chinese/Japanese/Korean consoles
AmbiguousWidth::Narrow	1 column	Western terminals, Windows Terminal default, VS Code, iTerm2
AmbiguousWidth::Auto	guessed from LANG/LC_ALL/LC_CTYPE	You want locale-based guessing; not reliable, see below
Why Auto is not the default
Auto inspects LANG, LC_ALL, and LC_CTYPE for a CJK locale prefix
(zh, ja, ko). But the environment locale does not determine how the
terminal renders ambiguous characters. A common case:

LANG=en_US.UTF-8 (English environment), but the terminal still renders Φ
as 2 columns because the user is on a CJK console with a CJK font.

In this case Auto guesses Narrow (1 column) and the table misaligns,
while the fixed default Wide (2 columns) stays aligned.

Because Auto can silently produce wrong output, the default is Wide.
If you have verified that Auto matches your deployment, opt in explicitly:

rust
use console_table::{AmbiguousWidth, Table};

let table = Table::new(rows).ambiguous_width(AmbiguousWidth::Auto);
If you don't know which mode your terminal needs
Render a small table with Φ under both modes and see which one aligns:

rust
use console_table::{Style, Table};

let rows = vec![
    vec!["名称".into(), "规格".into(), "材料".into()],
    vec!["电杆".into(), "Φ170".into(), "".into()],
    vec!["横担".into(), "03D103-133".into(), "2II2".into()],
];

println!("--- Wide ---");
println!(
    "{}",
    Table::new(rows.clone()).style(Style::Boxed).cjk_context(true).render()
);

println!("--- Narrow ---");
println!(
    "{}",
    Table::new(rows).style(Style::Boxed).cjk_context(false).render()
);
Whichever section has all │ aligned is the mode your terminal needs.

Compile-time note
The free functions display_width and truncate_visible use the same default
(Wide). If you need an explicit mode outside of Table, use the _with
variants:

rust
use console_table::{display_width, truncate_visible};

display_width("Φ170");                    // 5  (Φ = 2)
truncate_visible("Φ170CD1XYZ", 6);        // "Φ1..."  (Φ = 2)
For an explicit mode, the _with functions take the ambiguous width directly:

rust
use console_table::{display_width_with, truncate_visible_with};

display_width_with("Φ170", 1);            // 4  (Φ = 1, Narrow)
truncate_visible_with("Φ170CD1XYZ", 6, 1); // "Φ17..."
Note: display_width_with and truncate_visible_with are public but
lower-level. Most callers should prefer the Table builder.

Before and after
The same CJK data, rendered three ways. The first two use plain format! and
break or bloat on Chinese text; the third uses console-table and stays
aligned with almost no boilerplate.

The runnable version of all three is in
examples/compare.rs.

rust
let headers = ["序号", "镜像源", "平均延迟", "可用", "serde最新版"];
let data = [
    ["1", "rsproxy (字节跳动)", "57 ms", "是", "1.0.229"],
    ["2", "tuna (清华)", "16 ms", "是", "1.0.229"],
    ["8", "cqu (重大)", "失败", "否", "-"],
];
Before — manual format!
The common first attempt. {:<20} pads by char count, so Chinese characters
(which occupy 2 terminal columns each) collapse the columns.

rust
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
Output:

text
序号 镜像源               平均延迟   可用 serde最新版
------------------------------------------------------------
1    rsproxy (字节跳动)   57 ms      是   1.0.229
2    tuna (清华)          16 ms      是   1.0.229
8    cqu (重大)           失败       否   -
Columns drift because "镜像源" is 3 chars but 6 columns wide.

Before — manual width handling
To fix it by hand, you need your own display-width function, a fixed width
table, and manual padding. It works, but it is a lot of boilerplate, and every
new column means editing several places.

rust
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
Output:

text
序号    镜像源                    平均延迟      可用    serde最新版
----------------------------------------------------------------------
1       rsproxy (字节跳动)        57 ms         是      1.0.229
2       tuna (清华)               16 ms         是      1.0.229
8       cqu (重大)                失败          否      -
Aligned, but no borders, no per-column alignment, no truncation, no colors.

After — console-table
rust
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
Output:

text
┌──────┬────────────────────┬──────────┬──────┬─────────────┐
│ 序号 │ 镜像源             │ 平均延迟 │ 可用 │ serde最新版 │
├──────┼────────────────────┼──────────┼──────┼─────────────┤
│    1 │ rsproxy (字节跳动) │   57 ms  │ 是   │ 1.0.229     │
│    2 │ tuna (清华)        │   16 ms  │ 是   │ 1.0.229     │
│    8 │ cqu (重大)         │ 失败     │ 否   │ -           │
└──────┴────────────────────┴──────────┴──────┴─────────────┘
After — with truncation
Long values are capped at max_cell_width and end with ASCII ....
Truncation never breaks alignment.

rust
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
Output:

text
┌──────┬────────────────────┬──────────┬──────┬─────────────┬──────────────────────────────────────────┐
│ 序号 │ 镜像源             │ 平均延迟 │ 可用 │ serde最新版 │ 地址                                     │
├──────┼────────────────────┼──────────┼──────┼─────────────┼──────────────────────────────────────────┤
│    1 │ rsproxy (字节跳动) │   57 ms  │ 是   │ 1.0.229     │ sparse+https://rsproxy.cn/index/         │
│    2 │ tuna (清华)        │   16 ms  │ 是   │ 1.0.229     │ sparse+https://mirrors.tuna.tsinghua.edu… │
│    8 │ cqu (重大)         │ 失败     │ 否   │ -           │ sparse+https://mirrors.cqu.edu.cn/crates… │
└──────┴────────────────────┴──────────┴──────┴─────────────┴──────────────────────────────────────────┘
Summary
manual format!	manual width fn	console-table
CJK-aware width	✗	✓	✓
Ambiguous-width handling	✗	✗	✓
Column widths auto-computed	✗	✗	✓
Borders	✗	✗	✓
Per-column alignment	✗	✗	✓
Truncation	✗	✗	✓
Colors	✗	✗	✓
Lines of code	~6	~30	~10
API overview
Table
Method	Description
Table::new(rows)	Create a table. The first row is treated as the header. Rows may have different lengths; missing cells are treated as empty.
.style(Style)	Style::Simple (default) or Style::Boxed.
.aligns(Vec<Align>)	Per-column alignment. Missing columns default to Align::Left.
.max_cell_width(usize)	Truncate data cells to at most this display width. Headers are not truncated.
.header_color(Color)	Foreground color for the header row.
.body_color(Color)	Foreground color for data rows.
.ambiguous_width(AmbiguousWidth)	How to count ambiguous-width characters. Default Wide.
.cjk_context(bool)	Shorthand for .ambiguous_width(Wide) / .ambiguous_width(Narrow).
.render() -> String	Render the table. Every line ends with \n.
AmbiguousWidth
Variant	Φ counted as	Description
Wide (default)	2 columns	Matches CJK terminals.
Narrow	1 column	Matches Western terminals.
Auto	guessed	Guesses from LANG/LC_ALL/LC_CTYPE. Not reliable; opt in only if you have verified it.
Style
Variant	Description
Simple	Header row, a single separator, then data rows.
Boxed	Full box drawing with top, header separator, row separators, and bottom border.
Align
Variant	Description
Left	Left align.
Right	Right align.
Auto	Right align if the first whitespace-separated token parses as f64, otherwise left align.
Color
16 ANSI foreground colors:

Black, Red, Green, Yellow, Blue, Magenta, Cyan, White

BrightBlack, BrightRed, BrightGreen, BrightYellow,
BrightBlue, BrightMagenta, BrightCyan, BrightWhite

Free functions
Function	Description
render_table(rows, style)	Convenience: Table::new(rows).style(style).render().
display_width(&str) -> usize	Terminal display width, ignoring ANSI sequences. Uses the default Wide mode.
display_width_with(&str, aw: usize)	Same, with explicit ambiguous width (1 or 2).
truncate_visible(&str, usize) -> String	Truncate to a display width, appending ASCII .... Uses the default Wide mode.
truncate_visible_with(&str, usize, aw: usize)	Same, with explicit ambiguous width (1 or 2).
strip_ansi(&str) -> String	Remove CSI-style ANSI escape sequences.
pad_right_with_spaces(&str, usize)	Pad on the right by display width.
pad_left_with_spaces(&str, usize)	Pad on the left by display width.
is_numeric_like(&str) -> bool	Heuristic used by Align::Auto.
Design notes
Widths are measured, not assumed. Every column is
max(display_width of all cells in that column, including the header) + 2,
where +2 reserves one space of padding on each side.

Ambiguous characters default to 2 columns. Φ, ±, Ⅱ, ① and
similar characters are counted as 2 by default, matching CJK terminals.
Override with .cjk_context(false) if your terminal renders them as 1.
See Ambiguous-width characters.

... instead of …. The single-character ellipsis … (U+2026) renders
as 1 or 2 columns depending on the terminal and font, which breaks alignment.
The ASCII ... is always exactly 3 columns.

Colors are applied at render time. Width computation strips ANSI
sequences, so colored cells never widen a column.

Existing ANSI in cell text is stripped. If a cell already contains color
codes, they are removed and replaced by body_color (or no color). Do not mix
per-cell colors with body_color.

Every rendered line has equal display width. This includes the header,
separators, and data rows, so the table stays rectangular even when the
terminal renders wide characters at 2 columns.

Windows note
Older cmd.exe consoles may not interpret ANSI escape sequences. Use
Windows Terminal or enable virtual terminal
processing in the console.

When redirecting output to a file, prefer disabling colors yourself:

rust
use std::io::IsTerminal;

let table = Table::new(rows);
let table = if std::io::stdout().is_terminal() {
    table.header_color(Color::BrightCyan).body_color(Color::White)
} else {
    table
};
This keeps the redirected file free of \x1b[...m sequences.

Windows Terminal and Φ. Windows Terminal defaults to narrow for
ambiguous-width characters (one column). If you are rendering into Windows
Terminal, you may want .cjk_context(false). If you are rendering into
classic cmd.exe with a CJK font, keep the default Wide. See
Ambiguous-width characters.

FAQ
Why does my CJK table look misaligned in some terminals?

Two common causes:

Ambiguous-width characters. Φ, ±, Ⅱ, ① and similar are 1 or 2
columns depending on the terminal. The default is 2 (Wide). If your
terminal renders them as 1, call .cjk_context(false). See
Ambiguous-width characters.

Font mismatch. Some terminals and fonts render … (U+2026) as 2
columns, and a few render it as 1. This library uses ASCII ... for
truncation so the width is always 3.

If you still see misalignment, check that your terminal uses a font where
CJK characters are exactly twice the width of ASCII characters, and that
.cjk_context(...) matches your terminal.

Why is Auto not the default?

Because environment locale does not determine how the terminal renders
ambiguous characters. A machine can have LANG=en_US.UTF-8 and still render
Φ as 2 columns (CJK console with a CJK font). Auto would guess Narrow
and misalign, while the fixed default Wide stays correct. Auto is opt-in
for deployments that have verified it works.

Can I color individual cells?

Not directly. body_color applies to the whole data row, and any ANSI codes
inside a cell are stripped before rendering. If you need per-cell colors,
render the row yourself or use a lower-level API.

Why is max_cell_width applied to data cells only?

Headers are typically short labels, and truncating them can hide useful
information. If you need header truncation, apply truncate_visible to the
header string before passing it to Table::new.

Does the library write to stdout?

No. render() returns a String. Printing is up to the caller.

License
Licensed under either of

Apache License, Version 2.0
(LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0)

MIT license
(LICENSE-MIT or
https://opensource.org/licenses/MIT)

at your option.

Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

text

## 主要改动

| 位置 | 改动 |
|------|------|
| 顶部简介 | 加"handles ambiguous-width characters like `Φ` predictably" |
| Features | 新增 **Predictable ambiguous-width handling** 一条 |
| Installation | 版本从 `0.1` → `0.2`（默认行为变了，算破坏性） |
| 新增章节 | **Ambiguous-width characters**：解释 `Φ` 类字符、终端差异表、三种模式、为什么 `Auto` 不是默认、如何自测 |
| Before and after 汇总表 | 加一行 "Ambiguous-width handling" |
| API overview | `Table` 加 `.ambiguous_width()`、`.cjk_context()`；新增 `AmbiguousWidth` 枚举表；Free functions 加 `display_width_with`、`truncate_visible_with` |
| Design notes | 加 **Ambiguous characters default to 2 columns** 一条 |
| Windows note | 加 Windows Terminal vs `cmd.exe` 的 `Φ` 差异提示 |
| FAQ | 更新"为什么错位"，加"为什么 `Auto` 不是默认" |

## 一句话

> README 新增了 **Ambiguous-width characters** 章节，明确说明 `Φ` 类字符在终端里是 1 还是 2 列**不取决于操作系统语言**，本库**默认 `Wide`（2 列）**，并给出 `.cjk_context(false)` / `.ambiguous_width(Auto)` 的覆盖方式，以及"怎么自测终端需要哪种模式"的示例。API overview、Design notes、FAQ、Windows note 同步更新。