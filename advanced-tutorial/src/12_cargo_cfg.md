# 第 12 章：Cargo 进阶与条件编译

## 本章目标

- 学会用 **Cargo Workspace** 管理多 crate 项目
- 理解 **features** 做条件依赖与条件编译
- 掌握 `#[cfg(...)]` / `cfg!` / `target_os` 等条件编译
- 配置 `[profile.*]` 优化与调试选项
- 解释 TaskFlow 为何只用单 crate + 标准依赖

## 12.1 Cargo 回顾

> 📖 对照：TaskFlow 的 `Cargo.toml` 只有 `[package]` 和 `[dependencies]`，
> 是最朴素的形态。本章我们扩展。

`Cargo.toml` 主要字段：

```toml
[package]
name = "taskflow"
version = "0.1.0"
edition = "2021"
authors = ["..."]
license = "MIT"
description = "..."
rust-version = "1.70"      # MSRV
keywords = ["cli", "task"]
categories = ["command-line-utilities"]

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = "1"

[dev-dependencies]        # 仅测试时
assert_cmd = "2"

[build-dependencies]      # 编译 build.rs 时
cc = "1"
```

## 12.2 依赖版本语法

```toml
serde = "1"               # = ^1.0.0（≥1.0.0, <2.0.0）
serde = "=1.0.150"        # 精确版本
serde = ">=1.0, <2.0"     # 范围
serde = "~1.0"            # = ≥1.0.0, <1.1.0
serde = "*"               # 任意（别用）
serde = { git = "https://..." }           # git 仓库
serde = { path = "../serde" }             # 本地路径
serde = { version = "1", optional = true } # 可选依赖
```

`^` 是默认（最左非零位锁住）：`^1.2.3` ≡ `≥1.2.3, <2.0.0`；`^0.2.3` ≡ `≥0.2.3, <0.3.0`。

## 12.3 Cargo Workspace（工作区）

当项目有多个 crate（如 `taskflow-core`、`taskflow-cli`、`taskflow-tui`），
Workspace 共享 `Cargo.lock` 和 `target/` 目录：

```
ruststudy/
├── Cargo.toml          # workspace 根
├── crates/
│   ├── core/
│   │   └── Cargo.toml
│   ├── cli/
│   │   └── Cargo.toml
│   └── tui/
│       └── Cargo.toml
```

**根 `Cargo.toml`**：

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]   # 共享依赖版本
serde = { version = "1", features = ["derive"] }
anyhow = "1"
```

**子 crate `crates/cli/Cargo.toml`**：

```toml
[package]
name = "taskflow-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
taskflow-core = { path = "../core" }   # 本地 crate
serde = { workspace = true }           # 引用 workspace 依赖
anyhow = { workspace = true }
```

好处：
- 统一版本管理（改一处全 crate 跟着）
- 共享编译产物（避免重复编译依赖）
- 适合做"内核 + 多前端"架构

> 思考：TaskFlow 现在是单 crate。若以后加 `taskflow-tui`（终端 UI）和
> `taskflow-sync`（云同步），可以拆成 workspace：`core` + `cli` + `tui` + `sync`。

## 12.4 features：条件依赖

让用户按需启用功能：

```toml
[features]
default = ["csv"]
csv = ["dep:csv"]                # 启用 csv 依赖
json = []
sqlite = ["dep:rusqlite"]
full = ["csv", "json", "sqlite"]

[dependencies]
csv = { version = "1", optional = true }
rusqlite = { version = "0.29", optional = true }
```

```bash
cargo build                       # 默认 features（csv）
cargo build --no-default-features --features json,sqlite
```

代码里用 `#[cfg(feature = "...")]` 条件编译：

```rust
#[cfg(feature = "csv")]
pub fn export_csv(...) { /* ... */ }

#[cfg(not(feature = "csv"))]
pub fn export_csv(...) -> Result<()> {
    Err(anyhow!("csv feature 未启用"))
}
```

> 📖 对照：TaskFlow 直接把 `csv` 列为强依赖。若想做成可选，可改成 feature——
> 不需要导出功能的用户编译出来的二进制更小。

### feature 联合/互斥

- 联合：`full = ["csv", "json"]`
- 互斥：Cargo features **不直接支持互斥**，要靠运行期检查或拆 crate。
  这是 Cargo 长期被诟病的设计。

## 12.5 条件编译 `#[cfg]`

```toml
[target.'cfg(windows)'.dependencies]
winapi = "0.3"
```

```rust
#[cfg(target_os = "windows")]
fn clear_screen() { /* cls */ }

#[cfg(target_os = "macos")]
fn clear_screen() { /* clear */ }

#[cfg(target_os = "linux")]
fn clear_screen() { /* clear */ }
```

常用条件：

| 条件 | 含义 |
|------|------|
| `target_os = "windows"` / `"macos"` / `"linux"` | 操作系统 |
| `target_arch = "x86_64"` / `"aarch64"` | CPU 架构 |
| `target_pointer_width = "64"` | 指针宽度 |
| `unix` / `windows` | 平台族 |
| `feature = "xxx"` | feature 标志 |
| `test` | `cargo test` 时为真 |
| `debug_assertions` | debug 构建时为真 |

逻辑组合：

```rust
#[cfg(all(unix, not(target_os = "macos")))]
fn f() { /* Linux/*BSD */ }

#[cfg(any(feature = "a", feature = "b"))]
fn g() {}
```

### `cfg!` 宏 vs `#[cfg]` 属性

- `#[cfg(...)]`：编译期移除代码
- `cfg!(...)`：返回 `bool`，代码仍在

```rust
if cfg!(target_os = "windows") {
    println!("Windows"); // 非 Windows 时这段仍在二进制里
}
```

## 12.6 `#[cfg(test)]` 与测试隔离

> 📖 对照：TaskFlow 大量用 `#[cfg(test)] mod tests { ... }`。
> 这样测试代码不进发布二进制，且测试模块能访问私有项。

```rust
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() { assert_eq!(add(1, 2), 3); }
}
```

## 12.7 build.rs：构建脚本

某些库需要在编译前生成代码（如解析 protobuf、绑定 C 库）。在根目录建 `build.rs`：

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-lib=dylib=foo");
    println!("cargo:rerun-if-changed=wrapper.h");
}
```

TaskFlow 用不到 build.rs——它只依赖纯 Rust crate。学到 FFI / 绑定 C 库时才需要。

## 12.8 `[profile.*]`：优化与调试

```toml
[profile.dev]
opt-level = 0          # 默认 0，开发期不优化
debug = true           # 默认 true

[profile.release]
opt-level = 3          # 默认 3，最大优化
lto = "thin"           # 链接期优化，二进制更小更快
codegen-units = 1      # 单线程编译，优化更彻底（编译变慢）
strip = true           # 去符号表，二进制更小

[profile.dev.package."*"]
opt-level = 2          # 仅依赖在 dev 下也优化（让依赖快但主项目快编译）
```

常用命令：

```bash
cargo build                # dev profile
cargo build --release      # release profile
cargo run --release        # 跑 release
cargo test --release
```

### TaskFlow 优化建议

如果你发布 TaskFlow，建议在 `Cargo.toml` 加：

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

二进制能从 ~5MB 缩到 ~2MB。

## 12.9 版本与发布

### SemVer

`MAJOR.MINOR.PATCH`：
- MAJOR：破坏性变更（1.x → 2.0）
- MINOR：向后兼容新功能（1.0 → 1.1）
- PATCH：向后兼容 bug 修复（1.0.0 → 1.0.1）

Cargo 默认 `^` 兼容 MINOR/PATCH 升级。

### `cargo release`

```bash
cargo install cargo-release
cargo release patch  # 自动 bump 版本、打 tag、推送
```

### 发布到 crates.io

```bash
cargo login <token>
cargo publish --dry-run
cargo publish
```

## 12.10 工程化常用命令速查

```bash
cargo check                  # 快速检查不生成代码
cargo clippy                 # lint，提示 idiom 改进
cargo fmt                    # 格式化
cargo fmt -- --check         # CI 中检查格式
cargo test                   # 跑所有测试
cargo test --doc             # 文档测试
cargo doc --open             # 生成并打开文档
cargo tree                   # 依赖树
cargo update                 # 更新 Cargo.lock 内的依赖
cargo outdated               # 检查过期依赖（需 cargo install cargo-outdated）
cargo audit                  # 安全漏洞扫描（需 cargo install cargo-audit）
cargo expand                 # 看宏展开（需 cargo install cargo-expand）
cargo bloat --release        # 分析二进制体积
```

> 📖 对照：TaskFlow 的 `.github/workflows/deploy.yml` 已经在 CI 里用 `cargo fmt --check`
> 和 `cargo clippy`，这是工程化标配。

## 12.11 常见陷阱

### 陷阱 1：循环依赖

Cargo 不允许 crate 间循环依赖。若 A 依赖 B 又被 B 依赖，要么拆出公共 C，
要么用 trait 反转依赖（B 定义 trait，A 实现）。

### 陷阱 2：feature 泄漏

启用某依赖的 feature 会影响整个 workspace。例如 `serde` 在 A crate 启用
`derive`，B crate 也会看到——这叫"feature unification"。

### 陷阱 3：dev / release 行为不同

`debug_assertions` 在 release 关闭，`assert!`（基于 `panic!`）仍生效。
`debug_assert!` 只在 dev 触发——别把关键校验放 `debug_assert!`。

### 陷阱 4：`Cargo.lock` 该不该提交

- **二进制项目**：提交（保证可复现构建）
- **库项目**：不提交（让下游决定版本）

> TaskFlow 是二进制项目，所以提交 `Cargo.lock` 是对的。

## 12.12 练习

1. 把 TaskFlow 的 `csv` 依赖改成可选 feature `csv-export`，并在代码里用
   `#[cfg(feature = "csv-export")]` 保护 `export_tasks`。

2. 给 TaskFlow 加一个 `[profile.release]` 配置：`lto = true`、`strip = true`、
   `codegen-units = 1`，对比 `cargo build --release` 前后二进制大小。

3. 写一个 `cfg_if!` 风格的跨平台函数 `fn home_dir() -> PathBuf`，
   分别在 Windows（用 `env!("USERPROFILE")`）和 Unix（用 `env!("HOME")`）下返回家目录。

4. 假设要把 `ruststudy/` 改成 workspace，包含 `myapp`、`playground`、
   `advanced-tutorial`（虚构）三个成员，写出根 `Cargo.toml`。

## 12.13 小结

| 概念 | 一句话 |
|------|--------|
| Workspace | 多 crate 共享 lock/target/依赖版本 |
| features | 条件依赖与条件编译 |
| `#[cfg]` | 编译期条件，跨平台/feature/test |
| `[profile.*]` | dev/release 等构建配置 |
| `cargo clippy` / `fmt` / `audit` | 工程化三件套 |
| `Cargo.lock` | 二进制提交，库不提交 |

> 下一章我们触碰 Rust 的"暗面"——**Unsafe Rust**。
> 它的存在不是为了让你天天用，而是让你知道何时确实需要、能看懂别人写的。

---

[← 第 11 章](./11_macros.md) | [下一章 →](./13_unsafe.md)

---

📧 联系作者：pebblerwon@qq.com
