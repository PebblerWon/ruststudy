# 第 4 章：字符串与集合类型深入

## 本章目标

- 彻底分清 `String` / `&str` / `str` / `OsString` / `PathBuf` 等"字符串家族"
- 掌握 `String` 的容量与增长策略
- 熟悉 `HashMap`、`BTreeMap`、`HashSet`、`VecDeque`、`LinkedList` 的取舍
- 理解 `Hash` / `Ord` / `Eq` trait 是集合的基石
- 解释 TaskFlow 里 `Vec<String>`、`PathBuf` 的选择

## 4.1 字符串家族全景

| 类型 | 拥有? | 编码 | 用途 |
|------|------|------|------|
| `String` | ✓ 拥有，堆 | UTF-8 | 可变字符串，最常用 |
| `&str` | ✗ 借用 | UTF-8 | 字符串切片，函数参数首选 |
| `str` | （unsized） | UTF-8 | 最底层的字符串类型，几乎只以 `&str` 出现 |
| `OsString` / `OsStr` | 拥有/借用 | 平台原生 | 文件名、环境变量（可能是非 UTF-8） |
| `PathBuf` / `Path` | 拥有/借用 | 平台原生 | 跨平台路径，封装 `OsString` |
| `CString` / `CStr` | 拥有/借用 | 字节+NUL | FFI 调用 C |
| `Cow<'a, str>` | 二选一 | UTF-8 | "可能借用、可能拥有"，第 8 章详谈 |

> 📖 对照：TaskFlow 用 `PathBuf` 存数据文件路径（`store.rs`），用 `String` 存任务标题，
> 用 `&str` 做函数参数。这三个的选择都不是偶然。

## 4.2 `String` 的内部：`(ptr, len, capacity)`

```rust
let mut s = String::with_capacity(10);
s.push_str("hello");
// s.ptr  → 堆上 "hello"
// s.len  = 5（已用字节数）
// s.cap  = 10（已分配容量）
```

- `len()`：已用字节数（**不是字符数**）
- `capacity()`：已分配字节数
- `push` / `push_str`：若 `len + 新增 > capacity`，重新分配（通常翻倍）并拷贝

```rust
let s: String = "abc".repeat(100);
println!("{}", s.capacity()); // 取决于增长策略
```

**性能提示**：已知最终大小时，用 `String::with_capacity(n)` 一次性分配，避免多次扩容。

## 4.3 `String` ↔ `&str` 转换

```rust
// &str → String
let s1 = "hi".to_string();
let s2 = String::from("hi");
let s3: String = "hi".into();
let s4 = "hi".to_owned();           // to_owned 等价于 to_string

// String → &str（自动 deref）
let s = String::from("hi");
takes_str(&s);     // &String → &str
takes_str(&s[..]); // 显式切片
takes_str(s.as_str());
```

`to_owned()` 来自 `ToOwned` trait，`&str` 的 `to_owned` 返回 `String`，`&[T]` 的
返回 `Vec<T>`——它是"借用到拥有"的通用接口。

## 4.4 字符串与字节、字符

```rust
let s = "中文A";
s.len();                 // 7（中3 + 文3 + A1 字节）
s.chars().count();       // 3（字符数）
s.bytes().count();       // 7（字节数）

for c in s.chars() { /* '中','文','A' */ }
for b in s.bytes() { /* 228,184,173,... */ }

// 索引 s[0] ✗ 编译错误：Rust 不允许按字节索引字符串（会破坏 UTF-8）
```

为什么不能 `s[i]`？因为 UTF-8 下第 `i` 个字节不一定是第 `i` 个字符的开头，
允许索引会鼓励写出 O(n) 看似 O(1) 的代码。

### `char` 是 4 字节 Unicode 标量值

```rust
let c: char = '中';
std::mem::size_of::<char>(); // 4
```

注意"字符"和"字素簇（grapheme cluster）"不同：`é` 可能是 1 个 `char`，
也可能是 `e` + `\u{301}`（组合重音）两个 `char`。要按字素簇切分用 `unicode-segmentation` crate。

## 4.5 格式化与拼接

```rust
let name = "Ann";
let s1 = format!("Hello, {name}!");
let s2 = ["a", "b", "c"].join(", ");          // "a, b, c"
let mut s3 = String::from("x");
s3 += "y";                                     // push_str 的语法糖
s3.push_str("z");
```

`format!` 性能：会分配新 `String`。在热路径里能用 `write!(s, ...)` 写到已有 `String`。

## 4.6 集合类型选型表

| 集合 | 查找 | 有序? | 适用 |
|------|------|------|------|
| `Vec<T>` | O(n) | 按插入 | 顺序数据、随机访问 |
| `VecDeque<T>` | O(n) | 双端 | 队列 / 双端栈 |
| `LinkedList<T>` | O(n) | 按 | 很少用，除非频繁中间 splice |
| `HashMap<K, V>` | O(1) 均摊 | 无 | 字典 / 计数 / 缓存 |
| `BTreeMap<K, V>` | O(log n) | 按键序 | 需要有序遍历 / 范围查询 |
| `HashSet<T>` / `BTreeSet<T>` | 同上 | 同上 | 去重、集合运算 |
| `BinaryHeap<T>` | O(1) 顶 | 堆序 | 优先队列（默认最大堆） |

> 经验：默认用 `Vec` 和 `HashMap`，需要有序遍历才上 `BTreeMap`，需要优先级才上堆。

## 4.7 `HashMap` 详解

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, i32> = HashMap::new();
scores.insert(String::from("Ann"), 10);
scores.insert(String::from("Bob"), 7);

// 读取：返回 Option<&V>
if let Some(v) = scores.get("Ann") {
    println!("Ann: {v}");
}

// 遍历（顺序不确定！）
for (k, v) in &scores {
    println!("{k}={v}");
}
```

### `entry`：不存在才插入

```rust
scores.entry(String::from("Ann")).or_insert(100); // 已存在不动
scores.entry(String::from("Cy")).or_insert(50);   // 不存在才插
```

### 计数惯用法

```rust
let text = "hello world hello rust";
let mut counts: HashMap<&str, i32> = HashMap::new();
for word in text.split_whitespace() {
    *counts.entry(word).or_insert(0) += 1;
}
// {"hello":2, "world":1, "rust":1}
```

### Key 需要 `Hash + Eq`

`String`、`i32`、`&str` 都实现了。自定义类型需 `#[derive(Hash, Eq, PartialEq)]`：

```rust
#[derive(Hash, Eq, PartialEq, Debug)]
struct Point { x: i32, y: i32 }
let mut m = HashMap::new();
m.insert(Point { x: 1, y: 2 }, "a");
```

> 默认 `HashMap` 用 SipHash（抗 HashDoS）。如果 key 可信且要极致性能，
> 可换 `ahash` / `fxhash` 等。

## 4.8 `BTreeMap`：有序字典

```rust
use std::collections::BTreeMap;
let mut m = BTreeMap::new();
m.insert("banana", 2);
m.insert("apple", 5);
m.insert("cherry", 1);

for (k, v) in &m { /* apple, banana, cherry —— 按键序 */ }
for (k, v) in m.range("apple".."cherry") { /* apple, banana */ }
```

键需要 `Ord`（不仅是 `Hash + Eq`）。

## 4.9 `VecDeque`：双端队列

```rust
use std::collections::VecDeque;
let mut q = VecDeque::new();
q.push_back(1);
q.push_back(2);
q.push_front(0);
assert_eq!(q.pop_front(), Some(0)); // FIFO 队列
```

底层是环形缓冲区，两端 O(1)。FIFO 队列、滑动窗口常用。

## 4.10 `BinaryHeap`：优先队列

```rust
use std::collections::BinaryHeap;
let mut h = BinaryHeap::new();
h.push(3);
h.push(1);
h.push(5);
assert_eq!(h.pop(), Some(5)); // 默认最大堆
```

最小堆：把元素取负，或用 `Reverse` 包装：
```rust
use std::cmp::Reverse;
let mut h: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
h.push(Reverse(3));
```

## 4.11 `PathBuf` 与 `Path`

跨平台路径操作。`PathBuf` 是拥有型（类似 `String`），`Path` 是借用型（类似 `str`）。

```rust
use std::path::{Path, PathBuf};

let mut p = PathBuf::from("/Users/whn");
p.push("Desktop");
p.push("ruststudy");
p.set_extension("json"); // 仅当末段是文件名

let path: &Path = &p;
path.exists();
path.is_file();
path.parent();           // Option<&Path>
path.file_name();        // Option<&OsStr>
```

> 📖 对照：TaskFlow 的 `JsonFileStore` 字段 `file_path: PathBuf`，因为它是
> 结构体长期持有的拥有型路径。函数参数若只读，写 `&Path` 更灵活。

**永远别用 `String` 拼路径**——Windows 路径分隔符是 `\`，Unix 是 `/`，
`PathBuf::push` 自动处理。

## 4.12 `Cow`：可能借可能拥有

`Cow<'a, T>`（Clone-on-Write）表示"要么借一个 `&T`，要么自己 own 一个 `T`"。
常用于"大多数情况直接返回输入切片，少数情况需要改写"：

```rust
use std::borrow::Cow;

fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains('\t') {
        Cow::Owned(input.replace('\t', "    ")) // 拥有
    } else {
        Cow::Borrowed(input)                    // 借用，零拷贝
    }
}
```

> 📖 对照：TaskFlow 的 `export_tasks` 返回 `String`，每次都分配。
> 若改为 `Cow<str>`，无任务时可以直接借用一个空切片，省一次分配。

## 4.13 练习

1. 写一个 `word_count(text: &str) -> HashMap<String, u32>`，统计每个单词出现次数
   （大小写不敏感）。用 `entry().or_insert()` 惯用法。

2. 解释下面两段代码哪个更高效，为什么：
   ```rust
   // A
   let mut s = String::new();
   for _ in 0..1000 { s.push_str("ab"); }
   // B
   let mut s = String::with_capacity(2000);
   for _ in 0..1000 { s.push_str("ab"); }
   ```

3. 给 TaskFlow 的 `Task.tags: Vec<String>` 设计一个统计函数
   `tag_frequency(tasks: &[Task]) -> BTreeMap<String, usize>`，返回按标签名排序的频次表。

4. 为什么不应该用 `String` 拼接文件路径？写一段代码同时验证 `PathBuf::push` 在
   macOS 和概念上 Windows 行为一致。

## 4.14 小结

| 概念 | 一句话 |
|------|--------|
| `String` / `&str` | own / borrow 的 UTF-8 字符串 |
| `PathBuf` / `Path` | 跨平台路径，别用 `String` |
| `HashMap` | O(1) 字典，key 需 `Hash + Eq` |
| `BTreeMap` | O(log n) 有序字典，key 需 `Ord` |
| `VecDeque` / `BinaryHeap` | 队列 / 优先队列 |
| `Cow` | 可能借可能 own，避免无谓分配 |

> 下一章我们学**泛型**——让你写的代码不再为每种类型重复一遍。

---

[← 第 3 章](./03_lifetimes.md) | [下一章 →](./05_generics.md)

---

📧 联系作者：pebblerwon@qq.com
