use console_table::{pad_left_with_spaces, pad_right_with_spaces};

#[test]
fn pad_right_basic() {
    assert_eq!(pad_right_with_spaces("ab", 5), "ab   ");
}

#[test]
fn pad_right_cjk() {
    // "中文" 显示宽度 4，补到 6 应加 2 个空格
    assert_eq!(pad_right_with_spaces("中文", 6), "中文  ");
}

#[test]
fn pad_left_basic() {
    assert_eq!(pad_left_with_spaces("ab", 5), "   ab");
}

#[test]
fn pad_left_cjk() {
    assert_eq!(pad_left_with_spaces("中文", 6), "  中文");
}

#[test]
fn pad_does_not_shrink() {
    assert_eq!(pad_right_with_spaces("abcdef", 3), "abcdef");
    assert_eq!(pad_left_with_spaces("abcdef", 3), "abcdef");
}
