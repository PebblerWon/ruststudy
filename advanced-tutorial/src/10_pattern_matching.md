# 第 10 章：模式匹配与运算符重载

## 本章目标

- 系统掌握 `match` / `if let` / `while let` / `let-else`
- 学会解构 struct / enum / tuple / 引用
- 理解绑定模式 `@`、守卫 `if`、`..`
- 学会用 `std::ops` 重载运算符（`+` `-` `*` `==` `[]` `Deref` ...）
- 解释 TaskFlow 里 `match self.status` 的全部分支写法

## 10.1 模式匹配：Rust 的瑞士军刀

> 📖 对照：TaskFlow 的 `Display for Status` 用 `match self { Todo => "待办", ... }`。
> 这只是模式匹配的入门。它能做的远不止此。

### `match` 必须穷尽

```rust
enum Color { Red, Green, Blue }

fn to_str(c: Color) -> &'static str {
    match c {
        Color::Red => "红",
        Color::Green => "绿",
        Color::Blue => "蓝",
    }
}
```

漏掉任一变体，编译错误。`_` 是通配：

```rust
match c {
    Color::Red => "红",
    _ => "其它",
}
```

## 10.2 解构（Destructuring）

### 解构 struct

```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 1, y: 2 };
let Point { x, y } = p;          // 字段简写：变量名等于字段名
let Point { x: a, y: b } = p;    // 重命名
let Point { x, .. } = p;         // 只取部分
```

### 解构 enum（带数据）

```rust
enum Msg {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

fn handle(m: Msg) {
    match m {
        Msg::Quit => println!("quit"),
        Msg::Move { x, y } => println!("move to {x},{y}"),
        Msg::Write(text) => println!("write {text}"),
        Msg::ChangeColor(r, g, b) => println!("rgb {r},{g},{b}"),
    }
}
```

`Move` 是 struct 变体（用 `{}`），`Write`/`ChangeColor` 是 tuple 变体（用 `()`）。

### 解构 tuple

```rust
let (a, b, c) = (1, 2, 3);
let (a, .., z) = (1, 2, 3, 4); // a=1, z=4，忽略中间
let (first, _) = (1, 2);
```

### 解构嵌套

```rust
let ((a, b), Point { x, y }) = ((1, 2), Point { x: 3, y: 4 });
```

## 10.3 `if let` / `while let` / `let-else`

### `if let`：只关心一种情况

```rust
let v: Option<i32> = Some(5);
if let Some(n) = v {
    println!("是 {n}");
} else {
    println!("None");
}
```

> 📖 对照：TaskFlow 的 `if let Some(t) = title { ... }`、`if let Some(s) = status`。

### `while let`：循环到不匹配

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("{top}");
}
```

### `let-else`：Rust 1.65+，早返回的好工具

```rust
fn parse(s: &str) -> u32 {
    let n: u32 = s.parse().ok() else {
        return 0; // 不匹配时执行
    };
    n * 2
}
```

替代了过去的 `if let Some(n) = ... else { return; }` 啰嗦写法。

## 10.4 绑定 `@`、守卫 `if`、`..`

### `@` 绑定值范围

```rust
match age {
    0 => println!("婴儿"),
    n @ 1..=12 => println!("儿童 {n}"),
    n @ 13..=17 => println!("少年 {n}"),
    n @ 18.. => println!("成人 {n}"),
}
```

`n @ 范围` 既匹配范围又把值绑定到 `n`。

### 守卫 `if`

```rust
match (a, b) {
    (x, y) if x == y => println!("相等"),
    (x, y) if x > y => println!("x 大"),
    _ => println!("其它"),
}
```

### `..` 忽略部分

```rust
let arr = [1, 2, 3, 4, 5];
match arr {
    [first, .., last] => println!("首 {first} 尾 {last}"),
    [single] => println!("单 {single}"),
    [] => println!("空"),
}
```

## 10.5 匹配引用

```rust
let v = &Some(5);
match v {
    &Some(n) => println!("{n}"), // 解引用
    &None => println!("none"),
}

// 或更地道：在模式里用 ref / 直接匹配
let v = Some(5);
match v {
    Some(ref n) => println!("{n}"), // 借用，不移动 v
    None => {}
}
```

现代 Rust 里 `ref` 已较少用——编译器会自动处理匹配引用。

## 10.6 运算符重载

Rust 通过 `std::ops` 里的 trait 让你自定义运算符：

| 运算符 | trait | 示例 |
|--------|-------|------|
| `+` `-` `*` `/` `%` | `Add` `Sub` `Mul` `Div` `Rem` | `a + b` |
| `-x` `!x` | `Neg` `Not` | 一元 |
| `&` `\|` `^` `<<` `>>` | `BitAnd` `BitOr` `BitXor` `Shl` `Shr` | 位运算 |
| `==` `!=` `<` `>` `<=` `>=` | `PartialEq` `PartialOrd` | 比较 |
| `a += b` | `AddAssign` 等 | 复合赋值 |
| `a[i]` | `Index` `IndexMut` | 索引 |
| `*a`（解引用） | `Deref` `DerefMut` | 智能指针 |
| `a..b` | `Range` | 范围（非 trait） |

### 例：自定义复数加法

```rust
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
struct Complex { re: f64, im: f64 }

impl Add for Complex {
    type Output = Complex; // 关联类型
    fn add(self, rhs: Complex) -> Complex {
        Complex { re: self.re + rhs.re, im: self.im + rhs.im }
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

fn main() {
    let a = Complex { re: 1.0, im: 2.0 };
    let b = Complex { re: 3.0, im: 4.0 };
    println!("{:?}", a + b);   // Complex { re: 4.0, im: 6.0 }
    println!("{:?}", a * b);   // Complex { re: -5.0, im: 10.0 }
}
```

> 📖 对照：`Add` 的关联类型 `type Output = Complex` 就是第 6 章讲的"关联类型"。

### 例：自定义 `Index`

```rust
use std::ops::{Index, IndexMut};

struct Matrix { data: Vec<Vec<i32>> }

impl Index<(usize, usize)> for Matrix {
    type Output = i32;
    fn index(&self, (r, c): (usize, usize)) -> &i32 {
        &self.data[r][c]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut i32 {
        &mut self.data[r][c]
    }
}

fn main() {
    let mut m = Matrix { data: vec![vec![1, 2], vec![3, 4]] };
    println!("{}", m[(0, 1)]); // 2
    m[(1, 0)] = 30;
}
```

## 10.7 比较：`PartialEq` / `Eq` / `PartialOrd` / `Ord`

| trait | 方法 | 含义 |
|-------|------|------|
| `PartialEq` | `eq` `ne` | `==` `!=`，可能不可比（NaN ≠ NaN） |
| `Eq` | （无新方法） | `==` 是自反的；标记 trait |
| `PartialOrd` | `partial_cmp` | `<` `>` 等，返回 `Option<Ordering>` |
| `Ord` | `cmp` | 全序，返回 `Ordering` |

浮点类型只实现 `PartialEq` / `PartialOrd`（NaN 不自反），不能做 `HashMap` 的 key。
整数、`String`、`&str` 全都实现了 `Eq` + `Ord`。

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Version(u32, u32);

fn main() {
    let v1 = Version(1, 0);
    let v2 = Version(1, 5);
    assert!(v1 < v2);
}
```

> 📖 对照：TaskFlow 的 `enum Status { Todo, InProgress, Done }` 只 derive 了
> `PartialEq`。若加上 `Eq` + `PartialOrd`，就能 `tasks.sort()` 按状态排序。

## 10.8 `Deref` 与 `DerefMut`

让自定义类型"像指针一样"被解引用：

```rust
use std::ops::{Deref, DerefMut};

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

fn main() {
    let mut b = MyBox(5);
    println!("{}", *b); // 5
    *b = 10;
}
```

**别滥用 `Deref`**：它是为智能指针设计的，不要拿来"继承"。要共享接口用 trait。

## 10.9 模式匹配实战：状态机

```rust
enum State { Idle, Running, Paused, Stopped }
enum Event { Start, Pause, Resume, Stop }

fn transition(s: State, e: Event) -> State {
    use State::*;
    use Event::*;
    match (s, e) {
        (Idle, Start) => Running,
        (Running, Pause) => Paused,
        (Paused, Resume) => Running,
        (_, Stop) => Stopped,
        (s, _) => s, // 其它组合不变
    }
}
```

模式匹配是写状态机、解析器、解释器最强大的工具。

## 10.10 常见陷阱

### 陷阱 1：`match` 不穷尽

```rust
match opt {
    Some(x) => x,
    // ✗ 漏了 None
}
```

修复：加 `None => ...` 或 `_ => default`。

### 陷阱 2：守卫副作用

```rust
match x {
    n if expensive_check(n) => /* ... */,
    _ => /* ... */,
}
```

守卫每次匹配都执行，且不能保证副作用顺序。**守卫里别写副作用**。

### 陷阱 3：`=` 与 `==`

```rust
match x {
    5 = 5 => // ✗ 这是赋值不是模式
}
```

模式里写值字面量，不写 `=`。

### 陷阱 4：运算符重载改变语义

`a + b` 习惯性是"加法"。如果你让 `+` 表示"字符串拼接"或"集合合并"，要符合直觉。
否则起个普通方法名更清楚。

## 10.11 练习

1. 用 `match` 解构 `Option<Result<i32, String>>`，处理四种情况：
   `Some(Ok)` / `Some(Err)` / `None`。

2. 给下面的 `Vec3` 实现 `Add`、`Sub`、`Mul<f64>`（标量乘）：
   ```rust
   struct Vec3 { x: f64, y: f64, z: f64 }
   ```

3. 用模式匹配写一个简易四则运算解析器：
   ```rust
   enum Expr { Num(f64), Add(Box<Expr>, Box<Expr>), Mul(Box<Expr>, Box<Expr>) }
   fn eval(e: &Expr) -> f64 { /* ... */ }
   ```

4. 用 `let-else` 重写下面代码：
   ```rust
   fn first_word(s: &str) -> &str {
       if let Some(i) = s.find(' ') {
           &s[..i]
       } else {
           s
       }
   }
   ```
   提示：找空格，找不到就返回 `s`。这题可能 `let-else` 不直接适用，
   思考另一种模式。

## 10.12 小结

| 概念 | 一句话 |
|------|--------|
| `match` | 必须穷尽，多分支匹配 |
| 解构 | struct/enum/tuple/嵌套都能拆 |
| `if let` / `while let` / `let-else` | 简化单分支匹配 |
| `@` / 守卫 / `..` | 范围绑定、条件过滤、忽略部分 |
| `std::ops` | 运算符重载入口 |
| `Deref` / `Index` | 让自定义类型像内置类型 |
| `PartialEq` vs `Eq` | 浮点不能 `Eq`，整数能 |

> 下一章我们触碰 Rust 的"元编程"——**宏**。从 `println!` 到 `vec!`，
> 看似函数却远比函数强大。

---

[← 第 9 章](./09_concurrency_async.md) | [下一章 →](./11_macros.md)

---

📧 联系作者：pebblerwon@qq.com
