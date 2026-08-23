# 第 2 章：借用、引用与切片

## 本章目标

- 学会用 `&` / `&mut` 借用值而不夺走所有权
- 理解"同一时刻：多个不可变借用 **或** 一个可变借用"的规则
- 掌握"悬垂引用"为什么在 Rust 里不可能发生
- 理解切片 `&[T]` 和字符串切片 `&str` 的本质
- 解释 TaskFlow 里到处出现的 `&str` / `&[Task]`

## 2.1 引用：借用而不拥有

```rust
fn len(s: &String) -> usize { // s 是 String 的引用，不拥有它
    s.len()
}

fn main() {
    let s = String::from("hello");
    let n = len(&s);     // 把 s 的引用传进去
    println!("{s} {n}"); // ✓ s 还能用
}
```

`&s` 创建一个**指向 `s` 的引用**，函数借走"使用权"但不要所有权，函数返回后引用
自动作废，`s` 依旧归 `main` 所有。

> 📖 对照：TaskFlow 的 `service.list_tasks(&self, ...)`、`validate_title(title: &str)`
> 全是借用，所以调用方传完还能继续用。

## 2.2 可变引用 `&mut`

默认引用是只读的。要改值，得用 `&mut`：

```rust
fn push_bang(s: &mut String) {
    s.push('!');
}

fn main() {
    let mut s = String::from("hi");
    push_bang(&mut s);
    println!("{s}"); // hi!
}
```

注意三点：
1. 被借用的变量本身要声明 `mut`。
2. 借用时要写 `&mut s`。
3. 函数签名里参数类型是 `&mut String`。

## 2.3 借用规则（Rust 的灵魂）

> **在任意给定时刻，一个值要么有：**
> - **多个不可变引用（`&T`）**，**或者**
> - **唯一一个可变引用（`&mut T`）**。
>
> 二者不可同时存在。引用必须始终有效（无悬垂引用）。

```rust
let mut s = String::from("hi");

let r1 = &s;       // 不可变借用
let r2 = &s;       // 再来一个不可变借用，OK
println!("{r1} {r2}");
// 此后 r1/r2 不再使用（NLL 非词法生命周期）

let r3 = &mut s;   // 可变借用，OK（前面不可变借用已结束）
r3.push('!');
println!("{r3}");
```

```rust
let mut s = String::from("hi");
let r1 = &s;
let r2 = &mut s;   // ✗ 不可变借用 r1 仍在用，又来可变借用
println!("{r1} {r2}");
```

**为什么这样设计？** 多个 `&T` 并存是安全的（大家都只读）；但只要有 `&mut T` 在改，
其它任何引用都必须失效，否则就会出现"读到一半数据被改了"的数据竞争。

> 💡 Rust 的"非词法生命周期（NLL）"会让引用在**最后一次使用处**就失效，而不是
> 作用域结束。所以下面这样是合法的：
> ```rust
> let mut s = String::from("hi");
> let r1 = &s;
> println!("{r1}"); // r1 到这里就结束了
> let r2 = &mut s;  // OK
> ```

## 2.4 悬垂引用：编译期就拒绝

```rust
// ✗ 编译错误：返回了局部变量的引用
fn dangle() -> &String {
    let s = String::from("boom");
    &s
} // s 在这里被 drop，引用指向的内存已释放 → 悬垂！
```

修复方法：直接返回 `String`，把所有权交还调用者：

```rust
fn no_dangle() -> String {
    let s = String::from("ok");
    s // ✓
}
```

> 规则：**引用的生命周期不能超过被引用者的生命周期**。第 3 章我们用显式标注来描述这一点。

## 2.5 切片：不拥有数据的"视图"

切片 `&[T]` 是对一段连续数据的**借用视图**——只有 `(指针, 长度)`，不拥有数据。

```rust
fn sum(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

fn main() {
    let v = vec![1, 2, 3, 4];
    let arr = [10, 20, 30];

    println!("{}", sum(&v));        // Vec 借用成 &[i32]
    println!("{}", sum(&arr));      // 数组借用成 &[i32]
    println!("{}", sum(&v[1..3]));  // 子切片 &[i32]
}
```

切片的威力：**同一个函数既能吃 `Vec` 又能吃数组还能吃子段**。

> 📖 对照：TaskFlow 的 `store.save(&self, tasks: &[Task])` 就是切片参数，
> 所以你传 `&tasks`（`&Vec<Task>` 自动解引用成 `&[Task]`）即可。

### 切片字面量

```rust
let s: &[i32] = &[1, 2, 3]; // 切片字面量，指向一个临时数组
```

## 2.6 字符串切片 `&str`

`&str` 就是 `&[u8]` 的"我知道这是合法 UTF-8"特化版。

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5]; // 切片
let world: &str = &s[6..];

let literal: &str = "rust"; // 字符串字面量本身就是 &str
```

`"rust"` 这样的字面量被编译进二进制只读段，`&str` 指向那里，`'static` 生命周期。

### `String` vs `&str` 一图流

| | `String` | `&str` |
|---|----------|--------|
| 拥有所有权？ | ✓ | ✗（借用） |
| 可增长？ | ✓（`push_str` 等） | ✗ |
| 存在哪？ | 堆 | 任意（堆/栈/只读段） |
| 类比 C++ | `std::string` | `std::string_view` |
| 用作函数参数 | 倾向不用（拿所有权） | **首选**（借用，灵活） |

**经验法则**：函数参数能写 `&str` 就别写 `String`，能写 `&[T]` 就别写 `&Vec<T>`。

> 📖 对照：TaskFlow 里所有 `add_task(title: &str, ...)`、`search_task(keyword: &str)`
> 都遵循这条法则，所以你能传 `&String`、`"字面量"`、`&task.title` 等多种实参。

### 字符串切片按字节，小心中文！

```rust
let cn = "中文";
// let half = &cn[0..1]; // ✗ panic：从 UTF-8 中间切断
let ok = &cn[0..3]; // ✓ 一个中文字符占 3 字节
```

> 📖 对照：这正是 TaskFlow 里 `title.chars().count() > 100` 而非
> `title.len() > 100` 的原因——`.len()` 是字节数。

## 2.7 解引用强制（Deref Coercion）

`&String` 能自动变成 `&str`，`&Vec<T>` 能自动变成 `&[T]`，靠的是 `Deref` trait：

```rust
fn takes_str(s: &str) {}

let s = String::from("hi");
takes_str(&s); // &String 自动 deref 成 &str
```

`String` 实现了 `Deref<Target=str>`，编译器看到类型不匹配时会层层 deref。
你写 `&tasks` 传给 `&[Task]` 也是同理。

## 2.8 可变借用与不可变借用混用：常见坑

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];     // 不可变借用
v.push(4);             // ✗ 可变借用（push 内部要 &mut self），与 first 冲突
println!("{first}");
```

修复：把 `first` 用完再 push：

```rust
let mut v = vec![1, 2, 3];
let first = v[0];
v.push(4);             // ✓ 拷贝出来了，不再借用 v
```

## 2.9 引用作为结构体字段

结构体可以持有引用，但**必须标注生命周期**（第 3 章详解）：

```rust
struct Excerpt<'a> {
    part: &'a str, // 'a 表示：这个引用至少要活到 'a 这么久
}

fn main() {
    let s = String::from("hello world");
    let e = Excerpt { part: &s[0..5] };
    println!("{}", e.part);
}
```

## 2.10 练习

1. 下面代码错在哪？给两种修复方式（一种用 `&mut`，一种用返回值）。
   ```rust
   fn add_one(x: i32) { x += 1; }
   fn main() {
       let n = 1;
       add_one(n);
       println!("{n}");
   }
   ```

2. 写一个函数 `first_word(s: &str) -> &str`，返回第一个空白前的子串。
   提示：用 `s.find(char::is_whitespace)`。思考为什么返回 `&str` 而非 `String` 更好。

3. 下面代码为什么编译失败？修复它（不要用 `clone`）。
   ```rust
   fn main() {
       let mut s = String::from("hi");
       let r = &s;
       s.push('!');
       println!("{r}");
   }
   ```

4. 解释：为什么 TaskFlow 的 `validate_title(title: &str)` 能同时接受
   `String`、`&String`、`"字面量"` 三种实参？

## 2.11 小结

| 概念 | 一句话 |
|------|--------|
| `&T` | 不可变借用，可多个并存 |
| `&mut T` | 可变借用，唯一，不能与任何其它借用并存 |
| 借用规则 | 多读 OR 一写，引用始终有效 |
| `&[T]` / `&str` | 切片：借用的视图，不拥有数据 |
| 函数参数首选 | `&str` / `&[T]` 而非 `String` / `&Vec<T>` |

> 下一章我们直面让新手最头疼的 `'_` 标注——**生命周期**。
> 别怕，借用的规则你已经懂了，生命周期只是给编译器"打个标"说明谁活多久。

---

[← 第 1 章](./01_ownership.md) | [下一章 →](./03_lifetimes.md)

---

📧 联系作者：pebblerwon@qq.com
