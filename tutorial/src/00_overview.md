# Rust 新手实战教程：从零构建 TaskFlow 命令行任务管理工具

> 本教程以 `myapp` 项目（TaskFlow）为参考，带你从零构建一个完整的 Rust 命令行应用。
> 每一章对应项目开发的一个阶段，边讲概念边写代码，最终产出一个功能完整、可测试的 CLI 工具。

---

## 你将学到什么

通过本教程，你将掌握以下 Rust 核心技能：

| 技能领域 | 具体内容 |
|---------|---------|
| **项目结构** | Cargo 工作流、模块系统（`mod`/`use`）、多文件组织 |
| **数据建模** | `struct`、`enum`、`Option`/`Vec`、`derive` 宏 |
| **序列化** | `serde` + `serde_json` 派生、JSON 文件读写 |
| **错误处理** | `thiserror`（库层）+ `anyhow`（应用层）双层方案 |
| **CLI 解析** | `clap` derive 模式、子命令定义、自动 help 生成 |
| **Trait 设计** | 用 `trait` 抽象存储接口、面向接口编程 |
| **终端输出** | `comfy-table` 表格渲染、`colored` 彩色输出 |
| **测试** | 单元测试（`#[cfg(test)]`）、集成测试（`assert_cmd`） |
| **文件 I/O** | 读写 JSON 文件、备份机制、跨平台路径处理 |
| **实用库** | `chrono`（时间）、`uuid`（ID 生成）、`csv`（导出）、`dirs`（目录） |

## 前置要求

- 已安装 Rust（推荐通过 [rustup](https://rustup.rs/) 安装）
- 了解基本编程概念（变量、函数、条件分支）
- **不需要**有 Rust 经验——教程会逐步讲解每个概念

## 教程目录

| 章节 | 标题 | 核心内容 | 对应文件 |
|------|------|---------|---------|
| [第 1 章](./01_project_setup.md) | 项目初始化与架构设计 | `cargo new`、Cargo.toml、分层架构 | 项目骨架 |
| [第 2 章](./02_data_model.md) | 数据模型定义 | `struct`、`enum`、`serde`、`Display` | `models/` |
| [第 3 章](./03_storage_layer.md) | 存储层实现 | `trait`、文件 I/O、错误处理 | `store.rs`、`error.rs` |
| [第 4 章](./04_cli_parsing.md) | CLI 参数解析 | `clap` derive、子命令、参数校验 | `cli.rs` |
| [第 5 章](./05_service_layer.md) | 业务逻辑层 | CRUD、数据校验、UUID 生成 | `service.rs` |
| [第 6 章](./06_display_layer.md) | 展示层：表格与彩色输出 | `comfy-table`、`colored` | `display.rs` |
| [第 7 章](./07_main_dispatch.md) | 入口串联与调度 | `main.rs`、`match` 分发、错误收敛 | `main.rs` |
| [第 8 章](./08_testing.md) | 测试：从单元到集成 | `#[test]`、`assert_cmd`、临时目录 | `tests/` |
| [第 9 章](./09_enhanced_features.md) | 增强功能 | 搜索、统计、CSV 导出 | 全模块 |
| [第 10 章](./10_summary.md) | 总结与进阶 | 架构回顾、最佳实践、下一步 | — |

## 最终效果

完成全部章节后，你将拥有一个这样的命令行工具：

```bash
# 创建任务
$ taskflow add "学习Rust所有权" -p high -t rust,study --due 2026-09-01
✓ 任务创建成功：(a1b2c3d4)[高] 学习Rust所有权 (待办) - 2026-09-01

# 查看任务列表（彩色表格）
$ taskflow list
╭────┬──────────────┬────────┬────────┬───────┬────────────╮
│ ID │ 标题         │ 状态   │ 优先级 │ 标签  │ 截止日期   │
├────┼──────────────┼────────┼────────┼───────┼────────────┤
│ a1 │ 学习Rust所有权│ 未完成 │ 高     │ rust  │ 2026-09-01 │
╰────┴──────────────┴────────┴────────┴───────┴────────────╯

# 搜索、统计、导出
$ taskflow search "Rust"
$ taskflow stats
$ taskflow export -o tasks.csv
```

## 如何学习

1. **按顺序阅读**：每章建立在前一章的基础上
2. **动手写代码**：不要只看不写，每章的代码都要自己敲一遍
3. **运行验证**：每章结尾都有验证步骤，确保你的代码能跑
4. **阅读注释**：代码中的注释解释了"为什么这样做"
5. **参考原项目**：遇到困难时可以查看 `myapp/src/` 中的完整实现

---

[开始第 1 章 →](./01_project_setup.md)

---

📧 联系作者：pebblerwon@qq.com
