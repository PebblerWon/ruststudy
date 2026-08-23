# 第 11 章：宏入门

## 本章目标

- 区分两种宏：声明宏 `macro_rules!` 与过程宏（proc-macro）
- 理解宏 vs 函数的本质差异
- 能读懂并写简单 `macro_rules!` 宏
- 知道何时该用宏、何时该用泛型/函数
- 解释 TaskFlow 里 `println!`/`vec!`/`format!`/`derive` 背后的机制

## 11.1 宏 vs 函数

| | 函数 | 宏 |
|---|------|------|
| 何时展开 | 运行期调用 | **编译期**展开 |
| 参数数量 | 固定 | 可变 |
| 参数类型 | 严格类型 | 接受任意 token |
| 能否拿 `Self` | ✓ | 通常 ✗（`derive` 例外） |
| 递归 | 函数递归 | 宏递归（编译期） |
| 调用语法 | `f(a, b)` | `m!(a, b)` 或 `m!` 或 `m![]` |
| 性能 | 一份代码 | 每处调用展开一份 |

宏是"**写代码的代码**"。当你看到"语法看起来不像函数能干的"，多半就是宏。

> 📖 对照：`println!("任务：{task}")` 接受任意数量参数、字符串里嵌入 `{task}` 语法——
> 函数做不到，所以 `println!` 是宏。`vec![1, 2, 3]` 同理。

## 11.2 两种宏

1. **声明宏（`macro_rules!`）**：用模式匹配写，类似 `match` 但匹配的是 token。
2. **过程宏（proc-macro）**：写 Rust 程序处理 token 流，分三类：
   - `#[derive(...)]`：自动实现 trait（serde、Debug 等）
   - 属性宏 `#[my_attr]`：装饰项
   - 函数式宏 `my_macro!(...)`：像声明宏但更强

本章只讲声明宏；过程宏超出新手范围，第 15 章给出学习路径。

## 11.3 `vec!` 宏长什么样（简化版）

```rust
macro_rules! vec {
    ( $( $x:expr ),* $(,)? ) => {
        {
            let mut v = Vec::new();
            $(
                v.push($x);
            )*
            v
        }
    };
}

fn main() {
    let v = vec![1, 2, 3];
}
```

逐行解读：
- `$( $x:expr ),*`：匹配"零或多个用逗号分隔的表达式"，每个绑定为 `$x`
- `$(,)?`：允许末尾多余一个逗号
- `$(` `)*`：重复展开，每次 `v.push($x);`

`$x:expr` 里的 `:expr` 是 **fragment specifier**，常见：

| 标识 | 匹配 |
|------|------|
| `expr` | 表达式 |
| `ident` | 标识符 |
| `ty` | 类型 |
| `pat` | 模式 |
| `stmt` | 语句 |
| `block` | 代码块 |
| `literal` | 字面量 |
| `tt` | 单个 token tree（最通用） |
| `vis` | 可见性修饰符 `pub` 等 |
| `meta` | 属性内容 |

## 11.4 写第一个宏：`hashmap!`

```rust
macro_rules! hashmap {
    ( $( $k:expr => $v:expr ),* $(,)? ) => {
        {
            let mut m = std::collections::HashMap::new();
            $(
                m.insert($k, $v);
            )*
            m
        }
    };
}

fn main() {
    let m = hashmap! {
        "a" => 1,
        "b" => 2,
    };
    println!("{:?}", m);
}
```

## 11.5 多分支宏

宏可以有多个分支，按顺序匹配：

```rust
macro_rules! greet {
    () => { println!("Hello!") };
    ($name:expr) => { println!("Hello, {}!", $name) };
    ($name:expr, $greeting:expr) => { println!("{}, {}!", $greeting, $name) };
}

greet!();                  // Hello!
greet!("Ann");             // Hello, Ann!
greet!("Ann", "Hi");       // Hi, Ann!
```

## 11.6 重复的三种形式

- `$(...)*`：0 次或多次
- `$(...)+`：1 次或多次
- `$(...)?`：0 次或 1 次

```rust
macro_rules! sum {
    () => { 0 };
    ( $first:expr $(, $rest:expr)* ) => {
        $first $(+ $rest)*
    };
}

fn main() {
    assert_eq!(sum!(), 0);
    assert_eq!(sum!(1), 1);
    assert_eq!(sum!(1, 2, 3), 6);
}
```

## 11.7 卫生宏（Hygiene）

Rust 宏是"卫生"的：宏内引入的标识符不会与调用处的同名变量冲突。

```rust
macro_rules! using_x {
    ($e:expr) => {
        let x = 10; // 宏内的 x
        println!("x = {}, e = {}", x, $e);
    };
}

fn main() {
    let x = 99;
    using_x!(x + 1); // 这里的 x 是外面的 99
    // 输出：x = 10, e = 100
}
```

宏内的 `x` 和外面的 `x` 在不同"语法上下文"，互不干扰。

## 11.8 常用内置宏速查

| 宏 | 用途 |
|---|------|
| `println!` / `print!` / `eprintln!` | 输出（带换行/不换行/标准错误） |
| `format!` | 拼成 String |
| `vec!` | 创建 Vec |
| `assert!` / `assert_eq!` / `assert_ne!` | 测试断言 |
| `dbg!` | 调试打印（带位置） |
| `todo!` / `unimplemented!` | 占位，运行 panic |
| `panic!` | 直接终止 |
| `matches!` | 简化 match 返回 bool |
| `write!` / `writeln!` | 写入实现 Write 的目标 |
| `concat!` | 编译期字符串拼接 |
| `env!` | 编译期读环境变量 |
| `include_str!` / `include_bytes!` | 编译期嵌入文件 |

### `matches!` 实战

```rust
let s = Status::Done;
if matches!(s, Status::Done | Status::InProgress) {
    println!("进行中或已完成");
}
```

等价于写一个完整 `match`，更简洁。

### `dbg!` 调试

```rust
let x = 5;
let y = dbg!(x * 2) + 1; // 打印 [src/main.rs:2] x * 2 = 10，并返回 10
println!("{y}"); // 11
```

> 📖 对照：TaskFlow 测试里没用 `dbg!`，但调试时极有用——比 `println!` 多带文件位置，
> 还会把表达式本身打印出来。

## 11.9 `derive` 过程宏浅尝

`#[derive(Debug, Clone, Serialize)]` 让编译器调过程宏自动生成代码。

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: i32, y: i32 }
// 编译器自动实现 Debug, Clone, PartialEq trait
```

`Serialize`/`Deserialize` 来自 serde，是第三方 derive 过程宏——这也是 TaskFlow
`#[derive(Serialize, Deserialize)]` 能让 `Task` 自动转 JSON 的原理。

第三方 derive 还很多：`thiserror::Error`、`clap::Parser`、`Default` 等。

> 你也可以自己写 derive 过程宏，但需要 `proc-macro = true` 的 crate，
> 且要懂 `syn` / `quote` 库。新手阶段先用，第 15 章给学习路径。

## 11.10 何时用宏

| 场景 | 用什么 |
|------|--------|
| 同一段逻辑不同类型 | 泛型 |
| 不同调用点不同行为（trait） | 泛型 + trait bound |
| 参数数量/结构变化 | 宏 |
| 编译期生成代码（derive） | 过程宏 |
| 想要 DSL（领域特定语言） | 宏 |
| 简单工具函数 | 函数 |

**经验**：能用函数/泛型就别用宏。宏难写难读难调试，是最后手段。

## 11.11 常见陷阱

### 陷阱 1：宏内变量名冲突

虽然卫生，但若宏想"导出"内部变量给外部用，要用 `#[macro_export]` 或 `paste` crate。

### 陷阱 2：分号与语句

```rust
macro_rules! bad {
    () => { let x = 1; println!("{x}") }; // 没分号
}
fn main() {
    bad!();
    bad!(); // ✗ 第二次报错：let x 重定义（其实是因为没分号，两次合并成一段）
}
```

宏展开为表达式时，注意是否需要分号或括号包裹。

### 陷阱 3：宏递归深度

宏可递归，但有深度限制（默认 128）。递归太深会编译错误。

### 陷阱 4：在宏里调试

`cargo expand` 工具能看到宏展开后的代码，调试宏必备：

```bash
cargo install cargo-expand
cargo expand
```

## 11.12 练习

1. 写一个 `print!` 风格的宏 `say!`，支持 `say!()`、`say!("hi")`、`say!("hi", "ann")`
   三种形式。

2. 写一个宏 `map_of!`，等价于 `hashmap!` 但用 `=>` 分隔，重复利用已有宏。

3. 用 `matches!` 重写：
   ```rust
   match opt {
       Some(_) if cond => true,
       _ => false,
   }
   ```

4. 用 `cargo expand` 看 TaskFlow 项目的 `Task` 结构体经过 `#[derive(Serialize)]`
   展开后是什么样子（提示：在 myapp 目录运行 `cargo expand`）。

## 11.13 小结

| 概念 | 一句话 |
|------|--------|
| 声明宏 `macro_rules!` | 用模式匹配 token，编译期展开 |
| 过程宏 | 三类：derive / 属性 / 函数式 |
| fragment specifier | `expr` `ident` `tt` 等，匹配 token 种类 |
| 卫生 | 宏内外同名变量不冲突 |
| `cargo expand` | 调试宏必备 |
| 能用函数就别用宏 | 宏是最后手段 |

> 下一章我们聊工程化：**Cargo 工作区、features、条件编译、profile**——
> 让你管理更大的项目。

---

[← 第 10 章](./10_pattern_matching.md) | [下一章 →](./12_cargo_cfg.md)

---

📧 联系作者：pebblerwon@qq.com
