# 第 8 章：测试——从单元到集成

## 本章目标

- 理解 Rust 测试的两种形式（单元测试 / 集成测试）
- 用 `assert_cmd` 编写 CLI 集成测试
- 用 `tempfile` 实现测试数据隔离
- 覆盖所有子命令的正常和异常路径

## 8.1 Rust 测试基础

### 单元测试 vs 集成测试

| 类型 | 位置 | 可见性 | 用途 |
|------|------|--------|------|
| 单元测试 | 各模块内的 `#[cfg(test)] mod tests` | 可访问私有成员 | 测试单个函数/方法 |
| 集成测试 | `tests/` 目录 | 只能访问公开 API | 测试完整用户场景 |

### 运行测试

```bash
cargo test              # 运行所有测试
cargo test test_add     # 只运行名字包含 test_add 的测试
cargo test -- --nocapture  # 显示 println! 输出
```

## 8.2 单元测试回顾

前面章节已经写了不少单元测试，例如 `service.rs` 中的：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_task() {
        let service = create_temp_services("service_add_task").unwrap();
        let task = service
            .add_task("测试", Some("描述"), None, vec!["tag1"], None)
            .unwrap();
        assert_eq!(task.title, "测试");
        assert!(!task.id.is_empty());
    }
}
```

单元测试的重点是**隔离测试**——每个函数独立验证。

## 8.3 集成测试：为什么需要？

单元测试验证了各个模块，但用户是通过命令行使用程序的。集成测试验证**完整的用户流程**：

```
用户输入命令 → CLI 解析 → Service 处理 → Store 读写 → 终端输出
```

## 8.4 关键改造：环境变量隔离

集成测试调用真实二进制，会走 `JsonFileStore::new()` → 读写 `~/.taskflow/data.json`。
如果不隔离，测试会**污染真实用户数据**！

**解决方案**：让 `store.rs` 支持环境变量覆盖数据目录：

```rust
// store.rs - JsonFileStore::new()
pub fn new() -> Result<JsonFileStore> {
    let data_dir = std::env::var("TASKFLOW_DATA_DIR")  // ← 新增：优先读环境变量
        .map(PathBuf::from)
        .or_else(|_| dirs::home_dir().ok_or(TaskError::HomeDirNotFound))
        .map(|d| d.join(".taskflow"))?;
    create_dir_all(&data_dir)?;
    let data_path = data_dir.join("data.json");
    Ok(JsonFileStore { file_path: data_path })
}
```

- 设置了 `TASKFLOW_DATA_DIR` → 用它的值作为数据目录
- 没设置 → 回退到 `~/.taskflow/`（正常行为不变）

## 8.5 集成测试基础设施

创建 `tests/cli_test.rs`：

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// 创建一个注入了临时数据目录的 taskflow Command
fn taskflow_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("taskflow").unwrap();
    cmd.env("TASKFLOW_DATA_DIR", temp_dir.path());
    cmd
}

/// 从 add 命令的输出中提取任务 ID
fn get_id_from_stdout(add_stdout: &str) -> String {
    add_stdout
        .split('(')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .unwrap()
        .to_string()
}
```

### 基础设施解读

| 组件 | 作用 |
|------|------|
| `Command::cargo_bin("taskflow")` | 找到并运行项目编译的二进制文件 |
| `.env("TASKFLOW_DATA_DIR", ...)` | 注入环境变量，隔离测试数据 |
| `TempDir::new()` | 创建唯一临时目录，drop 时自动清理 |
| `get_id_from_stdout` | 从 `"✓ 任务创建成功：(a1b2c3d4)..."` 中提取 ID |

> **为什么用 `TempDir` 而不是 `std::env::temp_dir()`？**
> `temp_dir()` 返回系统共享临时目录，多个测试会互相冲突。
> `TempDir::new()` 每次创建唯一路径，测试结束自动删除。

## 8.6 编写测试用例

### 测试 1：添加 + 列出

```rust
#[test]
fn test_add_and_list() {
    let temp = TempDir::new().unwrap();

    // 添加任务
    taskflow_cmd(&temp)
        .arg("add")
        .arg("集成测试任务")
        .assert()
        .success()
        .stdout(predicate::str::contains("创建成功"));

    // 列出任务
    taskflow_cmd(&temp)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("集成测试任务"));
}
```

### 测试 2：空标题报错

```rust
#[test]
fn test_add_empty_title() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("add")
        .arg("")
        .assert()
        .failure()                              // 退出码非 0
        .stderr(predicate::str::contains("标题不能为空"));  // 错误输出到 stderr
}
```

### 测试 3：强制删除

```rust
#[test]
fn test_delete_with_force() {
    let temp = TempDir::new().unwrap();

    // 先创建任务
    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("待删除")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let id = get_id_from_stdout(&add_stdout);

    // 用 --force 跳过确认
    taskflow_cmd(&temp)
        .arg("delete")
        .arg(&id)
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
}
```

### 测试 4：删除确认（stdin 交互）

```rust
#[test]
fn test_delete_confirm_y() {
    let temp = TempDir::new().unwrap();

    // 创建任务并获取 ID
    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("待删除")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let id = get_id_from_stdout(&add_stdout);

    // 模拟用户输入 y
    let mut cmd = taskflow_cmd(&temp);
    cmd.arg("delete").arg(&id);
    cmd.write_stdin("y\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
}
```

### 测试 5：更新任务

```rust
#[test]
fn test_update() {
    let temp = TempDir::new().unwrap();

    // 创建
    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("新增")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let id = get_id_from_stdout(&add_stdout);

    // 更新
    taskflow_cmd(&temp)
        .arg("update")
        .arg(&id)
        .arg("--title").arg("待修改")
        .arg("--status").arg("done")
        .arg("--priority").arg("low")
        .arg("--tag").arg("1,2")
        .assert()
        .success()
        .stdout(predicate::str::contains("任务已更新"))
        .stdout(predicate::str::contains("待修改"));

    // 更新不存在的 ID
    taskflow_cmd(&temp)
        .arg("update")
        .arg("badid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("任务不存在"));
}
```

### 测试 6：搜索

```rust
#[test]
fn test_search() {
    let temp = TempDir::new().unwrap();

    taskflow_cmd(&temp)
        .arg("add").arg("新增").arg("--description").arg("描述")
        .assert().success();

    taskflow_cmd(&temp)
        .arg("search").arg("描述")
        .assert()
        .success()
        .stdout(predicate::str::contains("搜索到 1 条结果"))
        .stdout(predicate::str::contains("新增"));
}
```

### 测试 7：统计

```rust
#[test]
fn test_stats() {
    let temp = TempDir::new().unwrap();

    // 空数据
    taskflow_cmd(&temp)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("总任务数：0"));

    // 添加 3 个任务后
    for _ in 0..3 {
        taskflow_cmd(&temp).arg("add").arg("新增").assert().success();
    }
    taskflow_cmd(&temp)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("总任务数：3"));
}
```

### 测试 8：导出

```rust
#[test]
fn test_export() {
    let temp = TempDir::new().unwrap();
    let csv_path = temp.path().join("test.csv");

    taskflow_cmd(&temp)
        .arg("export")
        .arg("--output")
        .arg(csv_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("已导出任务到"));

    assert!(csv_path.exists());
}

#[test]
fn test_export_unsupported_format() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("export")
        .arg("--format").arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不支持的导出格式"));
}
```

### 测试 9：帮助文档

```rust
#[test]
fn test_help() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("命令行任务管理工具"))
        .stdout(predicate::str::contains("使用示例"))
        .stdout(predicate::str::contains("taskflow add"));
}
```

## 8.7 assert_cmd API 速查

| 方法 | 作用 |
|------|------|
| `.arg("xxx")` | 添加命令行参数 |
| `.assert()` | 执行命令并返回断言对象 |
| `.success()` | 断言退出码为 0 |
| `.failure()` | 断言退出码非 0 |
| `.stdout(predicate::str::contains("..."))` | 断言 stdout 包含文本 |
| `.stderr(predicate::str::contains("..."))` | 断言 stderr 包含文本 |
| `.write_stdin("y\n")` | 模拟 stdin 输入 |
| `.get_output()` | 获取原始输出（用于解析） |

## 8.8 测试覆盖矩阵

| 子命令 | 正常路径 | 异常路径 |
|--------|---------|---------|
| add | 创建成功 ✓ | 空标题 ✗ |
| list | 有数据显示 ✓ | 空列表"暂无任务" ✓ |
| update | 更新成功 ✓ | ID 不存在 ✗ |
| delete | --force 删除 ✓ | — |
| delete | 确认 y 删除 ✓ | — |
| search | 命中关键字 ✓ | 无匹配 ✓ |
| stats | 空/非空统计 ✓ | — |
| export | 导出文件 ✓ | 不支持格式 ✗ |
| help | 帮助信息 ✓ | — |

## 8.9 验证

```bash
cargo test
```

应该看到所有单元测试和集成测试通过：

```
running 16 unit tests ... ok
running 11 integration tests ... ok

test result: ok. 27 passed; 0 failed
```

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `#[cfg(test)]` | 条件编译，测试代码不影响生产构建 |
| `assert_cmd` | 运行真实二进制、断言退出码和输出 |
| `predicates` | 灵活的断言匹配（contains、starts_with 等） |
| `TempDir` | 独占临时目录，自动清理 |
| 环境变量隔离 | `TASKFLOW_DATA_DIR` 防止测试污染真实数据 |
| `write_stdin` | 模拟交互式输入 |

---

[← 上一章](./07_main_dispatch.md) | [返回目录](./00_overview.md) | [下一章 →](./09_enhanced_features.md)

---

📧 联系作者：pebblerwon@qq.com
