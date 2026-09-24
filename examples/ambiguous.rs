//! 手动对比 Auto / Wide / Narrow 三种模式下，含 `Φ` 的表格是否对齐，
//! 并在英文环境、中文环境下各跑一遍，观察 `Auto` 的行为。
//!
//! 运行：
//! ```text
//! cargo run --example ambiguous
//! ```
//!
//! 肉眼看哪一段的竖线真正对齐，就知道你的终端把 `Φ` 渲染成几列。

use console_table::{Style, Table};

/// 打印当前环境变量。
fn print_env() {
    println!("LANG     = {:?}", std::env::var("LANG").ok());
    println!("LC_ALL   = {:?}", std::env::var("LC_ALL").ok());
    println!("LC_CTYPE = {:?}", std::env::var("LC_CTYPE").ok());
}

/// 渲染三段：Auto / Wide / Narrow。
fn render_all() {
    let rows = vec![
        vec!["名称".into(), "规格".into(), "材料".into()],
        vec!["电杆".into(), "Φ170".into(), "".into()],
        vec!["横担".into(), "03D103-133".into(), "2II2".into()],
        vec!["接地线".into(), "Φ8".into(), "".into()],
        vec!["拉线棒".into(), "03D103-187".into(), "Φ16  III".into()],
    ];

    println!("--- Auto（按环境变量判断） ---");
    println!(
        "{}",
        Table::new(rows.clone()).style(Style::Boxed).render()
    );

    println!("--- Wide（Φ 算 2 列，cjk_context=true） ---");
    println!(
        "{}",
        Table::new(rows.clone())
            .style(Style::Boxed)
            .cjk_context(true)
            .render()
    );

    println!("--- Narrow（Φ 算 1 列，cjk_context=false） ---");
    println!(
        "{}",
        Table::new(rows)
            .style(Style::Boxed)
            .cjk_context(false)
            .render()
    );
}

/// 临时设置 CJK locale，运行一段闭包，然后恢复。
///
/// 注意：`std::env::set_var` 在 Rust 2024 edition 里是 `unsafe` 的，
/// 且会修改进程全局状态。这里只用于手动观察的 example，不用于生产代码。
fn with_cjk_locale<F: FnOnce()>(f: F) {
    let saved = [
        ("LANG", std::env::var("LANG").ok()),
        ("LC_ALL", std::env::var("LC_ALL").ok()),
        ("LC_CTYPE", std::env::var("LC_CTYPE").ok()),
    ];

    // 模拟中文环境
    unsafe {
        std::env::set_var("LANG", "zh_CN.UTF-8");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LC_CTYPE");
    }

    f();

    // 恢复
    for (k, v) in saved {
        unsafe {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }
}

fn main() {
    println!("========== 当前环境（原样） ==========");
    print_env();
    println!();
    render_all();

    println!();
    println!("========== 模拟中文环境（LANG=zh_CN.UTF-8） ==========");
    with_cjk_locale(|| {
        print_env();
        println!();
        render_all();
    });
}