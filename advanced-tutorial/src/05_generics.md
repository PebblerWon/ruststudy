# 第 5 章：泛型

## 本章目标

- 学会用泛型参数 `<T>` 写一份代码、多类型复用
- 理解 `trait bound`（约束）的限制能力
- 掌握 `where` 子句、`impl Trait` 参数与返回值
- 区分"单态化"与"动态分发"
- 解释 TaskFlow 里为何几乎没用到泛型

## 5.1 没有泛型的世界

```rust
fn max_i32(a: i32, b: i32) -> i32 { if a > b { a } else { b } }
fn max_f64(a: f64, b: f64) -> f64 { if a > b { a } else { b } }
fn max_str(a: &str, b: &str) -> &str { if a > b { a } else { b } }
```

三个函数逻辑完全一样，只是类型不同。泛型就是来消灭这种重复的。

## 5.2 泛型函数

```rust
fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn main() {
    println!("{}", max(3, 7));           // i32
    println!("{}", max(2.5, 1.5));       // f64
    println!("{}", max("apple", "banana")); // &str
}
```

`<T: PartialOrd>` 表示"T 必须实现 `PartialOrd` trait（能比较大小）"。
这就是 **trait bound**——泛型不是"任意类型"，而是"满足某些能力的类型"。

## 5.3 泛型结构体与枚举

```rust
struct Pair<T> { first: T, second: T }

enum Opt<T> { Some(T), None }

impl<T> Pair<T> {
    fn new(a: T, b: T) -> Self { Pair { first: a, second: b } }
}
```

> 📖 对照：`Option<T>`、`Vec<T>`、`Result<T, E>` 都是泛型枚举/结构体。
> TaskFlow 用的 `Option<String>`、`Result<Task>` 就是别人写好的泛型。

### 多类型参数

```rust
struct Map<K, V> { key: K, value: V }
impl<K, V> Map<K, V> {
    fn into_key(self) -> K { self.key }
}
```

### `impl` 块带约束

```rust
impl<T: PartialEq> Pair<T> {
    fn same(&self) -> bool { self.first == self.second }
}
// 只有 T: PartialEq 的 Pair 才有 same 方法
```

## 5.4 trait bound 的几种写法

### 内联

```rust
fn sum<T: Add<Output = T> + Default + Copy>(xs: &[T]) -> T {
    let mut acc = T::default();
    for x in xs { acc = acc + *x; }
    acc
}
```

`+` 叠加多个约束。

### `where` 子句（推荐复杂约束用）

```rust
fn sum<T>(xs: &[T]) -> T
where
    T: Add<Output = T> + Default + Copy,
{
    /* 同上 */
}
```

更易读，尤其约束多时。

## 5.5 `impl Trait` 语法糖

```rust
// 参数位置：等价于泛型 + bound
fn print(x: impl std::fmt::Display) {
    println!("{x}");
}

// 返回位置：返回"某个实现了 Display 的类型"
fn make() -> impl std::fmt::Display {
    42 // 实际返回 i32，但调用方只看到 impl Display
}
```

**注意**：`impl Trait` 在返回位置**只能返回单一具体类型**，不能根据分支返回不同类型：

```rust
// ✗ 编译错误：i32 和 &str 是不同类型
fn cond(b: bool) -> impl Display {
    if b { 42 } else { "no" }
}
```

要返回"多态"得用 trait 对象 `Box<dyn Display>`（第 6 章讲）。

## 5.6 单态化（Monomorphization）：泛型的零成本

Rust 泛型在编译期为每种具体类型生成一份专用代码：

```rust
max(3, 7);          // 生成 max_i32
max(2.5, 1.5);      // 生成 max_f64
max("a", "b");      // 生成 max_str
```

这是**编译期完成**的，运行期没有"查找虚函数表"的开销——这就是"零成本抽象"。
代价是二进制体积变大（每种类型一份代码）。

> 与之相对的是**动态分发**（`dyn Trait`），第 6 章讲。

## 5.7 泛型与生命周期

泛型参数可以包含生命周期：

```rust
fn longest<'a, T: 'a>(x: &'a T, y: &'a T) -> &'a T {
    if std::ptr::eq(x, y) { x } else { y }
}
```

`T: 'a` 表示"T 不能含短于 `'a` 的引用"——通常是引用时才需要。

## 5.8 const 泛型

Rust 支持把编译期常量作为泛型参数：

```rust
fn first_n<T, const N: usize>(arr: &[T; N]) -> &[T] {
    &arr[..N.min(1)]
}

let a: [i32; 3] = [1, 2, 3];
first_n(&a);
```

最有名的应用是 `Vec<T>` 之外另有一个**定长数组**类型 `[T; N]`，
和 `Box<[T; N]>`、第三方 `Vec2`/`Mat3` 等。

> 📖 对照：TaskFlow 没用 `const` 泛型，因为定长数组在 CLI 应用里少见。
> 但写图形/物理/嵌入式时非常常用。

## 5.9 默认类型参数

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

`Rhs = Self` 表示不指定时默认 `Rhs` 就是 `Self`，所以 `2 + 3` 不用写 `i32: Add<i32>`。

## 5.10 泛型的常见陷阱

### 陷阱 1：忘了写 `<T>` 在 `impl` 上

```rust
struct Box<T>(T);

impl Box<T> { /* ✗ T 未定义 */ fn new(x: T) -> Self { Self(x) } }

// ✓
impl<T> Box<T> { fn new(x: T) -> Self { Self(x) } }
```

### 陷阱 2：约束写漏，方法用不了

```rust
fn sort_print<T>(xs: &mut [T]) { // 没写 Ord
    xs.sort(); // ✗ sort 要求 T: Ord
}

// ✓
fn sort_print<T: Ord>(xs: &mut [T]) { xs.sort(); }
```

### 陷阱 3：返回 `impl Trait` 想多态

```rust
// ✗
fn make(b: bool) -> impl Animal {
    if b { Dog } else { Cat }
}

// ✓ 用 trait 对象
fn make(b: bool) -> Box<dyn Animal> {
    if b { Box::new(Dog) } else { Box::new(Cat) }
}
```

## 5.11 一个完整例子：泛型栈

```rust
struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self { Stack { data: vec![] } }
    fn push(&mut self, x: T) { self.data.push(x); }
    fn pop(&mut self) -> Option<T> { self.data.pop() }
    fn peek(&self) -> Option<&T> { self.data.last() }
}

fn main() {
    let mut s: Stack<i32> = Stack::new();
    s.push(1); s.push(2);
    assert_eq!(s.pop(), Some(2));

    let mut strs = Stack::new();
    strs.push("hi");
    assert_eq!(strs.peek(), Some(&"hi"));
}
```

同一份代码服务 `i32`、`&str`、任意类型——这就是泛型的力量。

## 5.12 练习

1. 写一个泛型函数 `fn first_or_default<T: Default>(xs: &[T]) -> T`，
   返回第一个元素的拥有副本（要求 `T: Clone + Default`）。

2. 给下面的 `Pair<T>` 增加一个方法 `swap`（交换两元素），并思考是否需要额外约束。
   ```rust
   struct Pair<T> { a: T, b: T }
   ```

3. 写一个泛型 `fn min<T: PartialOrd>(xs: &[T]) -> Option<&T>`，返回切片中最小元素的引用。

4. 解释：为什么 TaskFlow 项目里几乎没有自定义泛型？
   提示：CRUD 应用的数据类型在编译期就固定。

## 5.13 小结

| 概念 | 一句话 |
|------|--------|
| `<T>` | 类型参数，写一次服务多类型 |
| trait bound | 约束 T 必须实现的能力 |
| `where` | 复杂约束的可读写法 |
| `impl Trait` | 参数/返回值的语法糖 |
| 单态化 | 编译期为每具体类型生成专用代码，零运行期开销 |
| `const N: usize` | 编译期常量泛型 |

> 下一章我们深入 **Trait**——Rust 抽象的核心：关联类型、默认方法、超 trait、
> trait 对象与动态分发。

---

[← 第 4 章](./04_strings_collections.md) | [下一章 →](./06_traits_advanced.md)

---

📧 联系作者：pebblerwon@qq.com
