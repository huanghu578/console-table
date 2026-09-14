use console_table::{Align, Color, Style, Table};

fn main() {
    let rows = vec![
        vec![
            "序号".into(),
            "镜像源".into(),
            "平均延迟".into(),
            "可用".into(),
            "serde最新版".into(),
            "地址".into(),
        ],
        vec![
            "1".into(),
            "rsproxy (字节跳动)".into(),
            "57 ms".into(),
            "是".into(),
            "1.0.229".into(),
            "sparse+https://rsproxy.cn/index/".into(),
        ],
        vec![
            "2".into(),
            "tuna (清华)".into(),
            "16 ms".into(),
            "是".into(),
            "1.0.229".into(),
            "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/".into(),
        ],
        vec![
            "8".into(),
            "cqu (重大)".into(),
            "失败".into(),
            "否".into(),
            "-".into(),
            "sparse+https://mirrors.cqu.edu.cn/crates.io-index/".into(),
        ],
    ];

    let aligns = vec![
        Align::Right, // 序号
        Align::Left,  // 镜像源
        Align::Auto,  // 平均延迟
        Align::Left,  // 可用
        Align::Left,  // serde最新版
        Align::Left,  // 地址
    ];

    // ---- 1. 无颜色，Simple ----
    println!("--- Simple (无颜色) ---\n");
    let t = Table::new(rows.clone())
        .style(Style::Simple)
        .aligns(aligns.clone())
        .max_cell_width(40);
    println!("{}", t.render());

    // ---- 2. 彩色表头 + 彩色正文，Simple ----
    println!("--- Simple (表头亮青 + 正文白) ---\n");
    let t = Table::new(rows.clone())
        .style(Style::Simple)
        .aligns(aligns.clone())
        .max_cell_width(40)
        .header_color(Color::BrightCyan)
        .body_color(Color::White);
    println!("{}", t.render());

    // ---- 3. 彩色表头 + 彩色正文，Boxed ----
    println!("--- Boxed (表头亮青 + 正文白) ---\n");
    let t = Table::new(rows.clone())
        .style(Style::Boxed)
        .aligns(aligns.clone())
        .max_cell_width(40)
        .header_color(Color::BrightCyan)
        .body_color(Color::White);
    println!("{}", t.render());

    // ---- 4. 彩色表头 + 无正文颜色，Boxed ----
    // 只给表头上色，正文保持终端默认色，对比一下效果
    println!("--- Boxed (仅表头亮黄) ---\n");
    let t = Table::new(rows)
        .style(Style::Boxed)
        .aligns(aligns)
        .max_cell_width(40)
        .header_color(Color::BrightYellow);
    println!("{}", t.render());
}
