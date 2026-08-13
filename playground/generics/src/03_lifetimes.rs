//! ## 练习 3: 生命周期标注
//!
//! ### 学习目标
//!
//! - 理解为什么需要生命周期：编译器要确保引用不会指向已释放的数据
//! - 学会手动标注生命周期 `'a`
//! - 理解生命周期省略规则
//! - 理解结构体中的生命周期
//!
//! ### 概念讲解
//!
//! #### 为什么需要生命周期？
//!
//! Rust 的核心安全保证是「没有悬垂引用」。考虑：
//!
//! ```rust,ignore
//! let r;
//! {
//!     let x = 5;
//!     r = &x;
//! }  // x 在这里被销毁
//! println!("{}", r);  // r 指向了已销毁的 x！这是悬垂引用！
//! ```
//!
//! 生命周期标注就是告诉编译器「这个引用应该活多久」，让编译器检查安全。
//!
//! #### 生命周期省略规则
//!
//! 编译器有 3 条规则自动推断生命周期，大部分时候不需要手写：
//! 1. 每个引用参数都有自己的生命周期参数
//! 2. 如果只有一个输入生命周期，它赋给所有输出
//! 3. 如果有 `&self`/`&mut self`，self 的生命周期赋给所有输出
//!
//! 当规则不够用时，编译器会报错，这时需要手动标注。
//!
//! ### 你的任务
//!
//! 完成下面 3 个练习。每个练习会让你体会不同的生命周期场景。
//! 把 `todo!()` 替换为你的实现。

// ═══════════════════════════════════════
// 练习 3a: longest 函数
// ═══════════════════════════════════════

/// 返回两个字符串中较长的一个。
///
/// ### 生命周期问题
///
/// 这个函数返回一个引用（`&str`），但编译器不知道这个引用的生命周期
/// 应该跟 `a` 一样长还是跟 `b` 一样长。
///
/// ```rust,ignore
/// // 不加生命周期标注，编译器会报错：
/// fn longest(a: &str, b: &str) -> &str  // ❌ 编译错误
/// ```
///
/// ### 你的任务
///
/// 1. 给函数签名加上生命周期标注 `'a`，让它通过编译
/// 2. 实现逻辑：返回较长的字符串
/// 3. 理解标注的含义：返回的引用生命周期不超过 `a` 和 `b` 中较短的那个
///
/// 提示：
/// - 标注形式：`fn longest<'a>(x: &'a ..., y: &'a ...) -> &'a ...`
/// - `'a` 不是改变数据的存活时间，只是告诉编译器「它们的生命周期有关联」
/// - 实现逻辑用 `if x.len() > y.len() { x } else { y }`
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    todo!("实现：比较 x 和 y 的长度，返回较长的一个")
}

// ────── longest 的测试 ──────

#[cfg(test)]
mod longest_tests {
    use super::*;

    #[test]
    fn test_longest_basic() {
        let s1 = "long string";
        let s2 = "short";

        // s1 和 s2 是字符串字面量，生命周期是 'static（整个程序运行期）
        let result = longest(s1, s2);
        assert_eq!(result, "long string");
    }

    #[test]
    fn test_longest_equal_length() {
        let s1 = "abcd";
        let s2 = "xyz0";

        // 长度相等时返回 y（else 分支）
        let result = longest(s1, s2);
        assert_eq!(result, "xyz0");
    }

    #[test]
    fn test_longest_with_owned_strings() {
        let s1 = String::from("hello world");
        let s2 = String::from("hi");

        // 需要先获取 &str，因为 longest 接收 &str
        let result = longest(s1.as_str(), s2.as_str());
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_lifetime_constraint() {
        // 这个测试展示了生命周期的约束作用

        let s1 = String::from("long string");
        let s2 = String::from("hi");

        let result;
        {
            // s2 的生命周期比 s1 短
            let s2 = String::from("x");
            result = longest(s1.as_str(), s2.as_str());
            // result 的生命周期受 s2 约束，在 s2 还活着时使用是安全的
            assert_eq!(result, "long string");
        }
        // 这里 result 不能再使用，因为 s2 已经销毁（但编译器会阻止你）
    }
}

// ═══════════════════════════════════════
// 练习 3b: 带生命周期的结构体
// ═══════════════════════════════════════

/// 一个持有字符串引用的结构体。
///
/// ### 生命周期问题
///
/// 结构体如果包含引用字段，必须标注生命周期，告诉编译器
/// 「这个结构体不能比它引用的字符串活得更久」。
///
/// ```rust,ignore
/// // 不加生命周期标注，编译器会报错：
/// struct TextHolder {
///     text: &str,  // ❌ 编译错误：缺少生命周期标注
/// }
/// ```
///
/// ### 你的任务
///
/// 1. 给结构体加上生命周期标注 `'a`
/// 2. 实现 `new` 和 `get_text` 方法
/// 3. 理解：`TextHolder` 的实例不能比它引用的字符串活得更久
pub struct TextHolder<'a> {
    // 提示：字段类型是 &'a str
    text: &'a str,
}

impl<'a> TextHolder<'a> {
    /// 创建一个新的 TextHolder，持有传入的字符串引用
    pub fn new(text: &'a str) -> Self {
        todo!("构造 TextHolder，保存 text 引用")
    }

    /// 获取持有的字符串引用
    pub fn get_text(&self) -> &'a str {
        todo!("返回 self.text")
    }
}

// ────── TextHolder 的测试 ──────

#[cfg(test)]
mod text_holder_tests {
    use super::*;

    #[test]
    fn test_text_holder_basic() {
        let text = "hello world";
        let holder = TextHolder::new(text);

        assert_eq!(holder.get_text(), "hello world");
    }

    #[test]
    fn test_text_holder_with_owned_string() {
        let text = String::from("owned string content");
        let holder = TextHolder::new(&text);

        assert_eq!(holder.get_text(), "owned string content");
    }

    #[test]
    fn test_text_holder_lifetime_constraint() {
        // 这个测试展示了生命周期约束

        // 结构体不能比引用的数据活得更久
        let text = String::from("long lived");
        let holder = TextHolder::new(&text);

        // 在 text 还活着时，holder 是有效的
        assert_eq!(holder.get_text(), "long lived");

        // 如果 text 被销毁，holder 也不能再使用（编译器会阻止）
    }

    #[test]
    fn test_text_holder_update_reference() {
        let mut text = "first text".to_string();
        let mut holder = TextHolder::new(&text);

        assert_eq!(holder.get_text(), "first text");

        // 修改原始字符串
        text = "second text".to_string();
        holder = TextHolder::new(&text);

        assert_eq!(holder.get_text(), "second text");
    }
}

// ═══════════════════════════════════════
// 练习 3c: 第一个单词提取（理解省略规则）
// ═══════════════════════════════════════

/// 返回字符串中第一个单词（以空格分隔）。
///
/// ### 生命周期省略规则
///
/// 这个函数签名 `fn first_word(s: &str) -> &str` 没有显式生命周期标注，
/// 但能编译通过！因为编译器应用了「省略规则」：
///
/// 1. 每个引用参数获得自己的生命周期：`s: &'a str`
/// 2. 只有一个输入生命周期，赋给所有输出：`-> &'a str`
///
/// 等价于：`fn first_word<'a>(s: &'a str) -> &'a str`
///
/// ### 你的任务
///
/// 实现逻辑：找到第一个空格的位置，返回空格前的部分。
/// 如果没有空格，返回整个字符串。
///
/// 提示：
/// - 用 `s.bytes().position(|b| b == b' ')` 找空格位置
/// - 或用 `s.split_whitespace().next()`
/// - 注意：用 split 方式时，返回的是原始字符串的切片，生命周期正确
pub fn first_word(s: &str) -> &str {
    todo!("实现：找到第一个空格，返回空格前的部分")
}

// ────── first_word 的测试 ──────

#[cfg(test)]
mod first_word_tests {
    use super::*;

    #[test]
    fn test_first_word_basic() {
        let s = "hello world";
        assert_eq!(first_word(s), "hello");
    }

    #[test]
    fn test_first_word_multiple_spaces() {
        let s = "hello   world   rust";
        // 第一个空格前的部分
        assert_eq!(first_word(s), "hello");
    }

    #[test]
    fn test_first_word_no_space() {
        let s = "hello";
        // 没有空格，返回整个字符串
        assert_eq!(first_word(s), "hello");
    }

    #[test]
    fn test_first_word_empty_string() {
        let s = "";
        assert_eq!(first_word(s), "");
    }

    #[test]
    fn test_first_word_leading_space() {
        let s = "  hello";
        // 开头就是空格，返回空串
        assert_eq!(first_word(s), "");
    }

    #[test]
    fn test_first_word_returns_reference_to_original() {
        // 这个测试验证返回的切片指向原始数据

        let s = String::from("hello world");
        let word = first_word(&s);

        // word 是 s 的切片，不拥有自己的数据
        assert_eq!(word, "hello");

        // 修改 s 会影响 word 指向的数据（但 word 在修改前就不能再用了）
        // 下面这行如果取消注释会导致编译错误（生命周期约束）：
        // let mut s = String::from("hello world");
        // let word = first_word(&s);
        // s.clear();  // ❌ 不能在持有不可变引用时修改
        // println!("{}", word);
    }
}
