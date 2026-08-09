# TaskFlow 开发计划

## 总体安排

- **总工期：** 4 周
- **阶段划分：** 3 个阶段（基础 → 增强 → 完善）
- **里程碑：** 每阶段结束可演示一个完整功能集

---

## 阶段一：基础功能（第 1~2 周）

> 目标：完成核心 CRUD，可通过命令行增删改查任务

### 任务清单

- [✅] **T1.1 项目初始化** ✅
  - 配置 `Cargo.toml`，添加所有依赖
  - 建立项目目录结构
  - 确保 `cargo build` 通过
  - **产出：** 可编译的空项目骨架

- [✅] **T1.2 数据模型定义** ✅
  - 实现 `Task` 结构体
  - 实现 `Status`、`Priority` 枚举
  - 为所有类型派生 `Serialize`、`Deserialize`
  - 实现 `Display` trait 用于友好展示
  - 编写单元测试验证序列化/反序列化
  - **产出：** `src/models/` 模块，单元测试通过

- [✅] **T1.3 存储层实现** ✅
  - 实现 `Store` trait 定义接口
  - 实现 `JsonFileStore`：
    - 读取/写入 JSON 文件
    - 自动创建 `~/.taskflow/` 目录
    - 写入前备份旧文件
  - 实现 CRUD 方法：`create`、`read_all`、`read_by_id`、`update`、`delete`
  - 编写单元测试（使用临时文件）
  - **产出：** `src/store.rs`，单元测试通过

- [✅] **T1.4 CLI 参数解析** ✅
  - 使用 `clap` derive 定义子命令：
    - `add`：创建任务
    - `list`：列出任务
    - `update`：更新任务
    - `delete`：删除任务
  - 定义所有参数和选项
  - **产出：** `src/cli.rs`，参数解析正确

- [✅] **T1.5 业务逻辑层** ✅
  - 实现 `TaskService`：
    - `add_task()`：校验标题长度，生成 ID 和时间戳，调用 store
    - `list_tasks()`：支持筛选条件
    - `update_task()`：查找任务，更新字段，刷新 updated_at
    - `delete_task()`：查找并删除
  - **产出：** `src/service.rs`

- [x] **T1.6 端到端串联** ✅
  - `main.rs` 中解析 CLI 参数，调用 service，输出结果
  - 实现基本的成功/错误输出
  - 手动测试所有命令
  - **产出：** 基础功能端到端可用
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.7](docs/TECH_SOLUTION.md)
  - **实现要点：** dispatch 已覆盖 Add/List/Update/Delete/Search，Stats/Export 走 `anyhow::bail!` stub（按技术方案约定）

**阶段一验收：**

```bash
taskflow add "学习Rust" -p high
taskflow list
taskflow update <id> --status done
taskflow delete <id>
```

---

## 阶段二：增强功能（第 3 周）

> 目标：提升可用性，支持搜索、统计、美观输出

### 任务清单

- [✅] **T2.1 列表筛选增强**
  - 在 `list` 命令中支持 `--status`、`--priority`、`--tag` 筛选
  - 在 service 层实现筛选逻辑
  - **产出：** 筛选功能可用

- [x] **T2.2 搜索功能** ✅
  - 新增 `search` 子命令
  - 实现关键字模糊匹配（标题 + 描述）
  - 以表格展示结果
  - **产出：** `taskflow search <keyword>` 可用
  - **实现要点：** 大小写不敏感（双方 `to_lowercase()`），title/description 任一命中即返回；空 keyword 返回所有任务

- [x] **T2.3 表格与彩色输出** ✅
  - 使用 `comfy-table` 渲染表格
  - 使用 `colored` 添加颜色：
    - 优先级颜色：High=红，Medium=黄，Low=绿
    - 状态颜色：Done=绿（删除线），InProgress=蓝，Todo=灰
  - 成功/错误/警告信息带颜色
  - **产出：** 美观的彩色表格输出
  - **实现要点：** `display.rs` 提供 `print_task_table` + `print_success`/`print_error`/`print_info`/`print_warning`，main.rs 已接入；`print_warning` 当前无 caller，留给 T3.4

- [ ] **T2.4 统计面板**
  - 新增 `stats` 子命令
  - 实现统计计算逻辑：
    - 总数、各状态数量及占比
    - 各优先级数量
    - 已完成率
    - 逾期任务数
  - 格式化展示统计结果
  - **产出：** `taskflow stats` 可用

**阶段二验收：**

```bash
taskflow list --status done --priority high
taskflow search "Rust"
taskflow stats
```

---

## 阶段三：优化完善（第 4 周）

> 目标：健壮性、用户体验、代码质量

### 任务清单

- [ ] **T3.1 删除确认**
  - 删除前交互式确认提示
  - 支持 `--force` 跳过确认
  - **产出：** 防误删机制

- [ ] **T3.2 CSV 导出**
  - 新增 `export` 子命令
  - 使用 `csv` crate 导出
  - 支持 `-o` 指定输出文件
  - **产出：** `taskflow export -o tasks.csv` 可用

- [ ] **T3.3 错误处理优化**
  - 定义 `TaskError` 枚举（使用 `thiserror`）
  - 使用 `anyhow` 在 main 中统一处理
  - 所有错误路径有友好中文/英文提示
  - 消除所有 `unwrap()` 调用
  - **产出：** 无 panic，错误提示友好

- [ ] **T3.4 输入校验完善**
  - 标题长度校验（1~100 字符）
  - 日期格式校验
  - 标签数量限制（最多 10 个）
  - **产出：** 边界情况处理完善

- [ ] **T3.5 集成测试**
  - 使用 `assert_cmd` 编写 CLI 集成测试
  - 覆盖所有子命令的正常和异常路径
  - 使用临时目录隔离测试数据
  - **产出：** `tests/cli_test.rs`，`cargo test` 全部通过

- [ ] **T3.6 帮助文档**
  - 为每个子命令添加 `help` 文本
  - 添加使用示例
  - **产出：** `taskflow --help` 信息完整清晰

**阶段三验收：**

```bash
cargo test                    # 全部通过
taskflow export -o out.csv    # CSV 可打开
taskflow add ""               # 友好错误提示
taskflow --help               # 帮助信息完整
```

---

## 进度追踪

| 任务 | 状态      | 开始日期   | 完成日期   | 备注                           |
| ---- | --------- | ---------- | ---------- | ------------------------------ |
| T1.1 | ✅ 已完成 | 2026-08-08 | 2026-08-08 | Cargo.toml + 目录结构          |
| T1.2 | ✅ 已完成 | 2026-08-08 | 2026-08-08 | Task/Status/Priority + 测试    |
| T1.3 | ✅ 已完成 | 2026-08-08 | 2026-08-08 | JsonFileStore + 临时目录测试   |
| T1.4 | ✅ 已完成 | 2026-08-08 | 2026-08-08 | clap derive 子命令齐全         |
| T1.5 | ✅ 已完成 | 2026-08-08 | 2026-08-08 | TaskService 增删改查 + 校验    |
| T1.6 | ✅ 已完成 | 2026-08-09 | 2026-08-09 | main.rs dispatch：Add/List/Update/Delete/Search |
| T2.1 | ⬜ 待开始 |            |            |                                |
| T2.2 | ✅ 已完成 | 2026-08-09 | 2026-08-09 | search_task：大小写不敏感 + 单元测试 |
| T2.3 | ✅ 已完成 | 2026-08-09 | 2026-08-09 | display.rs + main.rs 接入，print_warning 待 T3.4 |
| T2.4 | ⬜ 待开始 |            |            |                                |
| T3.1 | ⬜ 待开始 |            |            |                                |
| T3.2 | ⬜ 待开始 |            |            |                                |
| T3.3 | ⬜ 待开始 |            |            |                                |
| T3.4 | ⬜ 待开始 |            |            |                                |
| T3.5 | ⬜ 待开始 |            |            |                                |
| T3.6 | ⬜ 待开始 |            |            |                                |

**状态说明：** ⬜ 待开始 | 🔵 进行中 | ✅ 已完成 | ⏸ 暂停
