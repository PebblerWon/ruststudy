# 第 7 章：闭包与迭代器深入

## 本章目标

- 理解闭包捕获变量的三种方式（`Fn` / `FnMut` / `FnOnce`）
- 学会 `move` 闭包与移动语义的交互
- 掌握 `Iterator` trait 的全貌与自定义迭代器
- 熟练组合迭代器链：`map` / `filter` / `fold` / `flat_map` / `zip` / `take` ...
- 解释 TaskFlow 里 `tasks.into_iter().filter(...).collect()` 的每一步

## 7.1 闭包：能捕获环境的"匿名函数"

```rust
let add = |a, b| a + b;          // 类型推断
let add: fn(i32, i32) -> i32 = |a, b| a + b; // 显式类型

let x = 10;
let add_x = |a| a + x;           // 捕获了 x
println!("{}", add_x(5));        // 15
```

闭包语法：`|参数| 表达式` 或 `|参数| { 语句; 表达式 }`。
闭包和函数最大区别：**能捕获定义处环境中的变量**。

> 📖 对照：TaskFlow 里 `tasks.into_iter().filter(|t| t.status == s).collect()`
> 中 `|t| t.status == s` 就是个闭包，捕获了外层 `s`。

## 7.2 三种捕获方式：`Fn` / `FnMut` / `FnOnce`

闭包捕获环境变量的方式由编译器按"最少权限"自动选择，对应三个 trait：

| trait | 捕获方式 | 能调用次数 |
|-------|---------|-----------|
| `FnOnce` | 取得所有权（move 出变量） | 只能调一次 |
| `FnMut` | 可变借用 `&mut` | 多次，可改捕获变量 |
| `Fn` | 不可变借用 `&` | 多次，只读 |

**层级关系**：实现 `Fn` 的也实现 `FnMut`，实现 `FnMut` 的也实现 `FnOnce`。

```rust
let mut v = vec![1, 2, 3];

// FnMut：可变借用 v
let mut push = || v.push(4);
push();
println!("{:?}", v); // [1,2,3,4]

// FnOnce：拿走 v
let consume = move || { println!("{:?}", v); };
consume();
// println!("{:?}", v); // ✗ v 已被 move 走
```

### 何时选哪个

- 函数参数要存闭包（如回调）：优先 `Fn`，要改环境用 `FnMut`，要消费环境用 `FnOnce`。
- 标准库 `Iterator::filter` 接 `FnMut(&Self::Item) -> bool`——因为可能多次调用。

## 7.3 `move` 关键字

`move` 强制闭包按值取得捕获变量的所有权，常用于把闭包传给线程或返回闭包：

```rust
fn make_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || { count += 1; count }
}
```

不加 `move`，`count` 会被借用，但 `make_counter` 一返回 `count` 就死了——闭包悬垂。
`move` 把 `count` 搬进闭包，闭包自己拥有它。

> 📖 对照：第 9 章会看到 `thread::spawn(move || { ... })` 把数据搬到新线程。

## 7.4 返回闭包

闭包大小不固定（每个闭包是独立匿名类型），不能直接返回 `impl Fn` 之外的形式：

```rust
// ✓ 返回 impl Fn
fn adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// ✓ 用 Box<dyn Fn> 返回多态
fn choose(b: bool) -> Box<dyn Fn(i32) -> i32> {
    if b { Box::new(move |x| x + 1) }
    else { Box::new(move |x| x - 1) }
}
```

## 7.5 `Iterator` trait 全貌

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;

    // 几十个默认方法，全部基于 next
    fn map<B, F: FnMut(Self::Item) -> B>(self, f: F) -> Map<Self, F> { ... }
    fn filter<P: FnMut(&Self::Item) -> bool>(self, p: P) -> Filter<Self, P> { ... }
    fn collect<B: FromIterator<Self::Item>>(self) -> B { ... }
    // ...
}
```

关键点：
1. `next` 是唯一必须实现的，其它都是默认方法。
2. `map` / `filter` 等**返回新的迭代器类型**（`Map<Self, F>`），零成本组合。
3. `collect` 通过 `FromIterator` 把迭代器"收集"成任意集合。

### 消费适配器 vs 迭代器适配器

- **迭代器适配器**（`map`/`filter`/`take`/...）：懒求值，返回新迭代器，不做事。
- **消费适配器**（`collect`/`sum`/`count`/`for_each`/`fold`/...）：触发实际遍历。

```rust
let v = vec![1, 2, 3, 4, 5];
let s: i32 = v.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x).sum();
//                       ^^^迭代器适配器（懒）          ^^^消费适配器（触发）
// 等价于：4² + 16 = 20
println!("{s}");
```

## 7.6 自定义迭代器

```rust
struct Fib { a: u64, b: u64 }

impl Fib {
    fn new() -> Self { Fib { a: 0, b: 1 } }
}

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let c = self.a;
        self.a = self.b;
        self.b = c + self.b;
        Some(c)
    }
}

fn main() {
    let sum: u64 = Fib::new().take(10).sum(); // 前 10 项之和
    println!("{sum}"); // 88
}
```

> 这是 Rust 惯用法：只要实现 `next`，`map`/`filter`/`take`/...全部免费可用。

## 7.7 常用迭代器方法速查

| 方法 | 作用 | 备注 |
|------|------|------|
| `map(f)` | 每个元素变 `f(x)` | 改元素 |
| `filter(p)` | 留下 `p(&x) == true` | 不改元素 |
| `filter_map(f)` | `map` + 过滤 `None` | 等价 `flat_map` 单值 |
| `flat_map(f)` | `f` 返回迭代器，全部"拍平" | 嵌套变扁平 |
| `flatten()` | 把 `Iter<Iter<T>>` 拍成 `Iter<T>` | `flat_map(\|x\| x)` |
| `take(n)` | 取前 n 个 | 提前结束 |
| `skip(n)` | 跳过前 n 个 | |
| `take_while(p)` / `skip_while(p)` | 条件取/跳 | |
| `zip(other)` | 配对成 `(a, b)` | 短板效应 |
| `enumerate()` | 配上索引 `(0, x), (1, x)` | 极常用 |
| `fold(init, f)` | 累加器 | 左折叠 |
| `reduce(f)` | 无初值的 fold | 第一个元素当 init |
| `scan(init, f)` | 有状态的 map | |
| `chain(other)` | 串接两个迭代器 | |
| `peekable()` | 可"偷看"下一个不消费 | 解析器常用 |
| `any(p)` / `all(p)` | 是否存在/全部满足 | 短路 |
| `find(p)` / `position(p)` | 找第一个满足的 | 短路 |
| `max()` / `min()` / `sum()` / `product()` | 聚合 | |
| `collect()` | 收集成集合 | |
| `for_each(f)` | 副作用遍历 | 不算纯函数式 |

> 📖 对照：TaskFlow `service.search_task` 用 `into_iter().filter(...).collect()`，
> `list_tasks` 用 `into_iter().filter(...).filter(...).collect()`——本可链式但分开也清晰。

## 7.8 实战：用迭代器重写命令式循环

命令式：

```rust
let mut evens_sq = vec![];
for &x in &nums {
    if x % 2 == 0 {
        evens_sq.push(x * x);
    }
}
```

函数式：

```rust
let evens_sq: Vec<_> = nums.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x).collect();
```

通常更短、更易读，但**别为了函数式而函数式**——复杂逻辑用 `for` 循环加 `if` 反而清楚。

### 性能

迭代器链和手写 `for` 循环性能**几乎一致**（编译器能融合多层循环）。不必担心"函数式更慢"。

## 7.9 `into_iter` vs `iter` vs `iter_mut`

```rust
let v = vec![1, 2, 3];

for x in v.iter()     { /* x: &i32 */ }      // 借用
for x in v.iter_mut() { /* x: &mut i32 */ }  // 可变借用
for x in v.into_iter(){ /* x: i32 */ }       // 消费 v，拿所有权
// 之后 v 不可用
```

> 📖 对照：TaskFlow `list_tasks` 用 `into_iter()`，因为后续要 `filter(|t| t.status == s)`
> 时移动 `Task` 比较 `Status`（`Status` 是 `Copy`）。如果只想读不改，用 `iter()` 借用更省。

## 7.10 闭包常见陷阱

### 陷阱 1：捕获了循环变量

```rust
let mut fs = vec![];
for i in 0..3 {
    fs.push(move || println!("{i}")); // move 拷贝 i（i32: Copy）
}
for f in fs { f(); } // 0 1 2 ✓
```

`i32: Copy`，每次 `move` 都是复制，没问题。但若是非 Copy 类型要注意所有权。

### 陷阱 2：闭包捕获可变借用，外部还想用

```rust
let mut v = vec![1, 2, 3];
let mut push = || v.push(4);
// v.push(0); // ✗ push 已经 &mut v 了
push();
```

### 陷阱 3：迭代器消费后还想用

```rust
let v = vec![1, 2, 3];
let sum: i32 = v.into_iter().sum();
// println!("{:?}", v); // ✗ v 被 into_iter 消费
```

## 7.11 练习

1. 用迭代器一行写出：给定 `vec!["apple", "banana", "cherry"]`，
   返回所有以 'a' 或 'b' 开头的水果长度之和。提示：`starts_with`、`map`、`sum`。

2. 实现 `struct Range { cur: i32, end: i32 }` 的 `Iterator`，产出 `[cur, end)`。
   并测试 `Range { cur: 1, end: 5 }.filter(|x| x % 2 == 0).collect::<Vec<_>>()`。

3. 把下面命令式代码改写成迭代器链：
   ```rust
   let mut total = 0;
   for (i, &x) in nums.iter().enumerate() {
       if i % 2 == 0 { total += x; }
   }
   ```

4. 写一个 `fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C`
   返回 `g ∘ f`（先 f 后 g）。提示：`move` 闭包。

## 7.12 小结

| 概念 | 一句话 |
|------|--------|
| 闭包 | 捕获环境的匿名函数 |
| `Fn`/`FnMut`/`FnOnce` | 只读/可改/消费 三种捕获 |
| `move` | 强制按值取得捕获变量 |
| `Iterator::next` | 唯一必须实现的方法 |
| 适配器链 | `map`/`filter` 懒组合，`collect`/`sum` 触发 |
| `into_iter`/`iter`/`iter_mut` | 消费/借用/可变借用 |

> 下一章我们直面 Rust 的"内存模型"——智能指针 `Box`/`Rc`/`Arc`/`RefCell`，
> 以及内部可变性。它们是构建递归类型、共享数据、运行期可变性的关键。

---

[← 第 6 章](./06_traits_advanced.md) | [下一章 →](./08_smart_pointers.md)

---

📧 联系作者：pebblerwon@qq.com
