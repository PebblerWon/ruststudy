//! ## 练习 16: 手动实现 From / TryFrom
//!
//! ### 学习目标
//! - 手动实现 `From` trait（myapp 里用了 `#[from]` 自动生成）
//! - 理解 `From` 和 `TryFrom` 的区别（`TryFrom` 返回 `Result`）
//! - 理解实现了 `From` 就自动获得 `Into`
//!
//! ### 背景
//!
//! Rust 的类型转换系统非常严谨。`From` 用于不会失败且开销很小的转换，而 `TryFrom` 用于可能失败的转换。
//! 当你为类型 `T` 实现了 `From<U>` 后，Rust 会自动为你实现 `Into<T>` for `U`。
//!
//! ```rust,ignore
//! impl From<String> for MyType { ... }
//! // 现在你可以这样写：let m: MyType = "hello".into();
//! ```
//!
//! ### 你的任务
//!
//! 1. 定义一个 `EmailAddress` 结构体。
//! 2. 实现 `From<String>`，假设输入总是合法的。
//! 3. 实现 `TryFrom<&str>`，检查字符串中是否包含 `@` 符号。
//! 4. 编写测试验证转换逻辑。

// ────────────── 实现区域 ──────────────

use std::convert::{From, TryFrom};

#[derive(Debug, PartialEq)]
pub struct EmailAddress {
    pub value: String,
}

impl From<String> for EmailAddress {
    fn from(s: String) -> Self {
        todo!("直接包装字符串")
    }
}

impl TryFrom<&str> for EmailAddress {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains('@') {
            Ok(EmailAddress { value: value.to_string() })
        } else {
            Err("Invalid email: missing @ symbol".to_string())
        }
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let s = "user@example.com".to_string();
        let email: EmailAddress = s.into(); // 使用 Into trait
        assert_eq!(email.value, "user@example.com");
    }

    #[test]
    fn test_try_from_valid() {
        let email = EmailAddress::try_from("admin@test.org").unwrap();
        assert_eq!(email.value, "admin@test.org");
    }

    #[test]
    fn test_try_from_invalid() {
        let result = EmailAddress::try_from("not-an-email");
        assert!(result.is_err());
    }
}
