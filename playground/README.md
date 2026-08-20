# Rust Playground — 补强基础练习集

> 目标：补强 myapp (TaskFlow) 项目中未涉及的 Rust 核心特性，用小练习逐个攻克。

## 前置要求

- 完成 Rust Book 前 1-10 章（所有权、借用、错误处理基础）
- 或完成 myapp (TaskFlow) 项目

## 练习总览（共 19 个练习，建议按序完成）

```
Phase 1 泛型与生命周期          →  Phase 2 智能指针          →  Phase 3 多线程
  01 泛型栈 Stack<T>               01 Box 二叉树               01 thread 并行计算
  02 泛型缓存 Cache<K,V>           02 Rc 共享只读               02 Mutex + Arc 计数器
  03 生命周期标注                   03 RefCell 内部可变性         03 Channel 生产者消费者
       │                               │                            │
       ▼                               ▼                            ▼
Phase 4 async/await              →  Phase 5 高级 Trait        →  Phase 6 宏
  01 async/await 基础              01 自定义 Iterator            01 vec_of_strings! 宏
  02 并发 HTTP 请求                 02 运算符重载                02 assert_approx_eq! 宏
  03 异步通道                      03 From/TryFrom
  04 并发文件词频统计                04 Drop trait
```

---

## Phase 1: 泛型与生命周期 `generics` ✅ 已就绪

| 顺序 | 文件                           | 练习内容     | 核心概念                                | 测试数 |
| :--: | ------------------------------ | ------------ | --------------------------------------- | :----: |
|  1   | `generics/src/01_stack.rs`     | 泛型栈       | `struct<T>`、泛型方法定义               |   6    |
|  2   | `generics/src/02_cache.rs`     | 泛型缓存     | 双泛型 `<K,V>`、返回引用与生命周期入门  |   7    |
|  3   | `generics/src/03_lifetimes.rs` | 生命周期标注 | `'a` 显式标注、结构体生命周期、省略规则 |   14   |

```bash
cargo test -p generics              # 运行全部 27 个测试
cargo test -p generics stack_01     # 只测练习 01
cargo test -p generics cache_02     # 只测练习 02
cargo test -p generics lifetimes_03 # 只测练习 03
```

**为什么先学这个：** 泛型和生命周期是 Rust 类型系统的核心。myapp 里你用了 `Option<T>`、`Result<T,E>` 但没写过自己的泛型。这里从零实现，帮你理解 `<T>` 到底在做什么。

---

## Phase 2: 智能指针与内部可变性 `smart_ptrs` ✅ 已就绪

| 顺序 | 文件                                | 练习内容           | 核心概念                                       |
| :--: | ----------------------------------- | ------------------ | ---------------------------------------------- |
|  4   | `smart_ptrs/src/01_binary_tree.rs`  | Box 二叉树         | `Box<T>`、递归类型为什么需要堆分配             |
|  5   | `smart_ptrs/src/02_graph.rs`        | Rc 共享只读        | `Rc<T>`、引用计数、`strong_count`              |
|  6   | `smart_ptrs/src/03_refcell_demo.rs` | RefCell 内部可变性 | `RefCell<T>`、运行时借用检查、`Rc<RefCell<T>>` |

```bash
cargo test -p smart_ptrs                # 全部
cargo test -p smart_ptrs binary_tree_01 # 只测练习 04
cargo test -p smart_ptrs graph_02       # 只测练习 05
cargo test -p smart_ptrs refcell_demo_03 # 只测练习 06
```

**为什么学这个：** myapp 里所有数据要么 owned 要么引用，没用过堆分配和共享所有权。智能指针是 Rust 内存管理的进阶武器。

---

## Phase 3: 多线程与并发 `concurrency` ✅ 已就绪

| 顺序 | 文件                              | 练习内容         | 核心概念                                       |
| :--: | --------------------------------- | ---------------- | ---------------------------------------------- |
|  7   | `concurrency/src/01_threads.rs`   | 并行计算斐波那契 | `thread::spawn`、`move` 闭包、`JoinHandle`     |
|  8   | `concurrency/src/02_mutex_arc.rs` | 共享计数器       | `Mutex<T>`、`Arc<T>`、`lock()`、`Send`/`Sync`  |
|  9   | `concurrency/src/03_channels.rs`  | 生产者-消费者    | `mpsc::channel`、`send`/`recv`、多线程词频统计 |

```bash
cargo test -p concurrency                # 全部
cargo test -p concurrency threads_01     # 只测练习 07
cargo test -p concurrency mutex_arc_02   # 只测练习 08
cargo test -p concurrency channels_03    # 只测练习 09
```

**为什么学这个：** Rust 的王牌特性——「无畏并发」。编译器在编译期保证线程安全，这是 Rust 区别于其他语言最大的优势之一。

---

## Phase 4: async/await 与 tokio `async_basics` ✅ 已就绪

| 顺序 | 文件                                      | 练习内容         | 核心概念                                         |
| :--: | ----------------------------------------- | ---------------- | ------------------------------------------------ |
|  10  | `async_basics/src/01_hello_async.rs`      | async/await 基础 | `async fn`、`Future`、`#[tokio::main]`、惰性求值 |
|  11  | `async_basics/src/02_concurrent_fetch.rs` | 并发 HTTP 请求   | `tokio::spawn`、`join!`、`reqwest`               |
|  12  | `async_basics/src/03_async_channels.rs`   | 异步通道         | `tokio::sync::mpsc`、`select!` 宏                |
|  13  | `async_basics/src/04_word_counter.rs`     | 并发文件词频统计 | 综合 async + Mutex + Channel（小实战）           |

```bash
cargo test -p async_basics                   # 全部
cargo test -p async_basics hello_async_01    # 只测练习 10
cargo test -p async_basics concurrent_fetch_02 # 只测练习 11
cargo test -p async_basics async_channels_03   # 只测练习 12
cargo test -p async_basics word_counter_04     # 只测练习 13
```

**为什么学这个：** async 是现代 Rust 最重要的缺口，也是 myapp 完全没有涉及的领域。从基础到小型实战，带你理解 Future 的状态机本质。

---

## Phase 5: 高级 Trait 与运算符重载 `traits_adv` ✅ 已就绪

| 顺序 | 文件                                   | 练习内容     | 核心概念                                            |
| :--: | -------------------------------------- | ------------ | --------------------------------------------------- |
|  14  | `traits_adv/src/01_custom_iterator.rs` | 自定义迭代器 | `Iterator` trait、关联类型 `type Item`、迭代器组合  |
|  15  | `traits_adv/src/02_vec2d.rs`           | 运算符重载   | `std::ops::Add`/`Sub`/`Mul`、运算符重载本质         |
|  16  | `traits_adv/src/03_from_into.rs`       | From/TryFrom | 手动实现类型转换（myapp 只用了 `#[from]` 自动生成） |
|  17  | `traits_adv/src/04_drop_trait.rs`      | Drop trait   | RAII 模式、自定义资源释放                           |

```bash
cargo test -p traits_adv                       # 全部
cargo test -p traits_adv custom_iterator_01    # 只测练习 14
cargo test -p traits_adv vec2d_02               # 只测练习 15
cargo test -p traits_adv from_into_03           # 只测练习 16
cargo test -p traits_adv drop_trait_04          # 只测练习 17
```

**为什么学这个：** myapp 用了 trait（Store trait）和 derive 宏，但没有手动实现过运算符重载、自定义迭代器、Drop 等。这些是深入理解 Rust trait 系统的关键。

---

## Phase 6: 声明式宏 `macros` ✅ 已就绪

| 顺序 | 文件                                | 练习内容     | 核心概念                                                      |
| :--: | ----------------------------------- | ------------ | ------------------------------------------------------------- |
|  18  | `macros/src/01_vec_of_strings.rs`   | 字符串向量宏 | `macro_rules!`、`$x:expr` 片段说明符、重复模式 `$($x:expr),*` |
|  19  | `macros/src/02_assert_approx_eq.rs` | 近似断言宏   | 多分支匹配、宏卫生性、`format!` 在宏中使用                    |

```bash
cargo test -p macros                         # 全部
cargo test -p macros vec_of_strings_01       # 只测练习 18
cargo test -p macros assert_approx_eq_02     # 只测练习 19
```

**为什么学这个：** myapp 里大量用了 derive 宏（`#[derive(Serialize)]` 等）但没写过自己的宏。了解 `macro_rules!` 能帮你理解 Rust 元编程的基础。

---

## 练习规则

1. **按顺序完成** — 每个练习建立在前面的概念上
2. **不要删除已有测试** — 测试是你的验证工具
3. **找到 `todo!()` 标记** — 替换为你的实现代码
4. **遇到编译错误先读** — Rust 编译器的提示非常有价值
5. **每个练习做完后** — `cargo test` 全绿再进下一个

## 时间预估

| 阶段                   | 练习数 | 预计时间   | 优先级 |
| ---------------------- | :----: | ---------- | :----: |
| Phase 1 泛型与生命周期 |   3    | 2-3 天     |  必做  |
| Phase 2 智能指针       |   3    | 2-3 天     |  必做  |
| Phase 3 多线程         |   3    | 3-4 天     |  必做  |
| Phase 4 async/await    |   4    | 4-5 天     |  必做  |
| Phase 5 高级 Trait     |   4    | 2-3 天     |  推荐  |
| Phase 6 宏             |   2    | 1-2 天     |  可选  |
| **合计**               | **19** | **2-3 周** |        |

全部完成后，Rust 基础就相当扎实了，再去做任何项目都不会有概念障碍。
