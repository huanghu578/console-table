use console_table::{render_table, Align, Color, Style, Table};

fn lines(s: &str) -> Vec<&str> {
    // 去掉末尾换行后按行切
    s.trim_end_matches('\n').split('\n').collect()
}

#[test]
fn empty_rows_renders_empty() {
    let t = Table::new(vec![]);
    assert_eq!(t.render(), "");
}

#[test]
fn single_header_no_data() {
    let t = Table::new(vec![vec!["A".into(), "B".into()]]);
    let out = t.render();
    let ls = lines(&out);
    // 表头 + 分隔线
    assert_eq!(ls.len(), 2);
    assert!(ls[1].starts_with('├'));
    assert!(ls[1].ends_with('┤'));
}

#[test]
fn simple_all_lines_same_width() {
    let rows = vec![
        vec!["序号".into(), "名称".into(), "值".into()],
        vec!["1".into(), "one".into(), "10".into()],
        vec!["2".into(), "二".into(), "20".into()],
    ];
    let out = render_table(rows, Style::Simple);
    let ls = lines(&out);
    // 每行的显示宽度应一致
    let widths: Vec<usize> = ls.iter().map(|l| console_table::display_width(l)).collect();
    let first = widths[0];
    for w in widths {
        assert_eq!(
            w, first,
            "all rendered lines should have equal display width"
        );
    }
}

#[test]
fn boxed_has_top_bottom_and_separators() {
    let rows = vec![
        vec!["A".into(), "B".into()],
        vec!["1".into(), "2".into()],
        vec!["3".into(), "4".into()],
    ];
    let out = render_table(rows, Style::Boxed);
    let ls = lines(&out);
    assert!(ls[0].starts_with('┌'));
    assert!(ls[0].ends_with('┐'));
    assert!(ls.last().unwrap().starts_with('└'));
    assert!(ls.last().unwrap().ends_with('┘'));
    // 表头分隔线 + 中间一条行分隔线
    let sep_count = ls.iter().filter(|l| l.starts_with('├')).count();
    assert_eq!(sep_count, 2);
}

#[test]
fn cjk_width_is_two_columns() {
    assert_eq!(console_table::display_width("中文"), 4);
    assert_eq!(console_table::display_width("abc"), 3);
    assert_eq!(console_table::display_width("中文abc"), 7);
}

#[test]
fn ansi_is_ignored_in_width() {
    assert_eq!(console_table::display_width("\x1b[32m是\x1b[0m"), 2);
    assert_eq!(console_table::strip_ansi("\x1b[32m是\x1b[0m"), "是");
}

#[test]
fn truncate_appends_ascii_ellipsis() {
    let s = "sparse+https://rsproxy.cn/index/";
    let t = console_table::truncate_visible(s, 20);
    assert!(t.ends_with("..."));
    assert!(console_table::display_width(&t) <= 20);
}

#[test]
fn truncate_short_string_unchanged() {
    let s = "abc";
    assert_eq!(console_table::truncate_visible(s, 10), "abc");
}

#[test]
fn truncate_zero_width() {
    assert_eq!(console_table::truncate_visible("abc", 0), "");
}

#[test]
fn truncate_exactly_suffix_width() {
    // max_width = 3，只放得下 "..."
    assert_eq!(console_table::truncate_visible("abcdef", 3), "...");
}

#[test]
fn align_right_pads_left() {
    let rows = vec![vec!["N".into()], vec!["1".into()], vec!["100".into()]];
    let out = Table::new(rows)
        .style(Style::Simple)
        .aligns(vec![Align::Right])
        .render();
    let ls = lines(&out);
    let data1 = ls[2];
    let data2 = ls[3];

    // 两行显示宽度一致
    assert_eq!(
        console_table::display_width(data1),
        console_table::display_width(data2)
    );

    // 去掉竖线后，短数字前导空格更多
    let inner1: String = data1.chars().filter(|c| *c != '│').collect();
    let inner2: String = data2.chars().filter(|c| *c != '│').collect();
    let spaces1 = inner1.chars().take_while(|c| *c == ' ').count();
    let spaces2 = inner2.chars().take_while(|c| *c == ' ').count();
    assert!(
        spaces1 > spaces2,
        "shorter number should have more leading spaces: data1={:?}, data2={:?}",
        data1,
        data2
    );
}

#[test]
fn align_auto_detects_numbers() {
    assert!(console_table::is_numeric_like("57 ms"));
    assert!(console_table::is_numeric_like("-3.14"));
    assert!(console_table::is_numeric_like("1e3"));
    assert!(!console_table::is_numeric_like("失败"));
    assert!(!console_table::is_numeric_like(""));
    assert!(!console_table::is_numeric_like("abc 1"));
}

#[test]
fn header_is_never_truncated() {
    let long_header = "这是一个非常非常非常长的表头名字";
    let rows = vec![vec![long_header.into()], vec!["short".into()]];
    let out = Table::new(rows)
        .style(Style::Simple)
        .max_cell_width(5)
        .render();
    assert!(out.contains(long_header), "header should appear in full");
}

#[test]
fn data_cells_are_truncated_when_over_limit() {
    let rows = vec![
        vec!["H".into()],
        vec!["this is a very long data cell".into()],
    ];
    let out = Table::new(rows)
        .style(Style::Simple)
        .max_cell_width(10)
        .render();
    assert!(out.contains("..."), "truncated cell should contain ...");
}

#[test]
fn colored_output_contains_ansi_codes() {
    let rows = vec![vec!["A".into()], vec!["1".into()]];
    let out = Table::new(rows)
        .header_color(Color::BrightCyan)
        .body_color(Color::Green)
        .render();
    assert!(out.contains("\x1b[96m"), "header color code missing");
    assert!(out.contains("\x1b[32m"), "body color code missing");
    assert!(out.contains("\x1b[0m"), "reset code missing");
}

#[test]
fn colored_cells_do_not_change_width() {
    let plain = vec![vec!["A".into(), "B".into()], vec!["x".into(), "y".into()]];
    let colored = plain.clone();
    let out_plain = render_table(plain, Style::Simple);
    let out_colored = Table::new(colored)
        .style(Style::Simple)
        .header_color(Color::Red)
        .body_color(Color::Blue)
        .render();
    // 去掉 ANSI 后两者应完全一致
    let stripped_plain = console_table::strip_ansi(&out_plain);
    let stripped_colored = console_table::strip_ansi(&out_colored);
    assert_eq!(stripped_plain, stripped_colored);
}

#[test]
fn ragged_rows_are_padded() {
    let rows = vec![
        vec!["A".into(), "B".into(), "C".into()],
        vec!["1".into()],
        vec!["x".into(), "y".into(), "z".into()],
    ];
    let out = render_table(rows, Style::Simple);
    let ls = lines(&out);
    let widths: Vec<usize> = ls.iter().map(|l| console_table::display_width(l)).collect();
    let first = widths[0];
    for w in widths {
        assert_eq!(w, first);
    }
}

#[test]
fn width_with_existing_ansi_in_cell_is_correct() {
    let rows = vec![vec!["H".into()], vec!["\x1b[31m红色\x1b[0m".into()]];
    let out = render_table(rows, Style::Simple);
    // 渲染后单元格里的红色被剥离，宽度按 "红色" = 4 列算
    assert!(!out.contains("\x1b[31m"), "cell ANSI should be stripped");
    let ls = lines(&out);
    let widths: Vec<usize> = ls.iter().map(|l| console_table::display_width(l)).collect();
    assert_eq!(widths[0], widths[1]);
}
