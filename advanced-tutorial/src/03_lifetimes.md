# 第 3 章：生命周期

## 本章目标

- 理解生命周期（lifetime）是"借用有效的作用域"的编译期标注
- 学会读 `&'a T` 标注，能写出 `fn foo<'a>(x: &'a T) -> &'a U`
- 掌握三条生命周期消除规则
- 理解 `'static` 的真正含义
- 解释 TaskFlow 里为何几乎没有出现生命周期标注

## 3.1 为什么需要生命周期

第 2 章我们说：**引用不能悬垂**。但有些函数返回的引用来自参数，编译器怎么知道
返回的引用能活多久？看一个经典例子：

```rust
// 这个函数返回哪个参数的引用？编译器看不出来。
// ✗ 编译错误：missing lifetime specifier
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

`x` 和 `y` 各自有自己的有效区间，返回值到底是 `x` 还是 `y` 在运行期才决定。
编译器做静态检查时无法预测，于是要求你**显式声明返回引用与哪个参数的生命周期绑定**。

## 3.2 生命周期标注语法

生命周期写法是个**泛型参数**，以 `'` 开头：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` 不是改变值的存活时长，而是**告诉编译器：返回的引用至少和 `x`、`y` 中
较短的那个一样长**。这样调用方就不会让返回值比 `x`/`y` 活得更久。

```rust
fn main() {
    let s1 = String::from("long string");
    let r;
    {
        let s2 = String::from("hi");
        r = longest(s1.as_str(), s2.as_str());
        println!("{r}"); // ✓ s2 在这个作用域内还活着
    }
    // println!("{r}"); // ✗ r 借了 s2，s2 已 drop
}
```

## 3.3 生命周期是"约束"，不是"延长"

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { /*...*/ }
```

`'a` 的语义是"三者一致"——它把 `x`、`y`、返回值的寿命**绑成同一档**。
你只能取最小公倍数，不能凭空让短命的活得长。

## 3.4 结构体里的生命周期

结构体持有引用时必须标注：

```rust
struct Excerpt<'a> {
    content: &'a str,
}

impl<'a> Excerpt<'a> {
    // 方法签名里的生命周期通常可被消除（见 3.6）
    fn first_line(&self) -> &'a str {
        self.content.split('\n').next().unwrap_or("")
    }
}

fn main() {
    let novel = String::from("第一章\n第二章");
    let ex = Excerpt { content: &novel };
    println!("{}", ex.first_line());
    // novel 不能比 ex 先死
}
```

> 📖 对照：TaskFlow 的 `Task` 结构体所有字段都是 `String`/`Vec`/`Option` 等
> **拥有型**字段，所以 `Task` 没有任何生命周期参数——这是惯用法：
> **能拥有就别借用**，否则结构体的生命周期会传染到所有使用者。

## 3.5 生命周期消除（Elision）规则

你写过的代码里大多没出现 `'a`，因为编译器在三种情况下自动补：

**规则 1**：每个引用参数各自获得一个独立生命周期。
```rust
fn foo(x: &str)            → fn foo<'a>(x: &'a str)
fn foo(x: &str, y: &str)   → fn foo<'a, 'b>(x: &'a str, y: &'b str)
```

**规则 2**：若只有一个输入生命周期参数，它赋给所有输出引用。
```rust
fn foo(x: &str) -> &str    → fn foo<'a>(x: &'a str) -> &'a str
```

**规则 3**：若有 `&self` / `&mut self`，它的生命周期赋给所有输出引用。
```rust
fn foo(&self, x: &str) -> &str → 返回值生命周期绑定 self
```

满足这三条就不必手写。`longest` 触发不了任何规则（两个输入引用、有输出引用、
没 self），所以必须手写。

> 📖 对照：TaskFlow 的 `TaskService::get_task_by_id(&self, id: &str) -> Result<Task>`
> 返回的是 `Task`（拥有型，不是引用），所以不需要绑定生命周期。

## 3.6 `'static` 生命周期

`'static` 表示"整个程序运行期"。所有字符串字面量都是 `&'static str`：

```rust
let s: &'static str = "我活在二进制只读段，进程不退我不灭";
```

**警惕滥用 `'static`**。新手遇到生命周期报错，常常给所有地方加 `'static` 让它编译通过——
这等于在说"我要一个永远不死的引用"，通常意味着设计有问题（要么该用 `String`，
要么该重新理清所有权）。

`'static` 作为约束（"T 在 'static 内有效"）也用来表示"任意生命周期"，
比如 `Box<dyn Trait + 'static>`、`tokio::spawn` 要求 future 是 `'static`。

## 3.7 一个完整例子：解析配置

```rust
struct Config<'a> {
    name: &'a str,
    value: &'a str,
}

fn parse<'a>(line: &'a str) -> Config<'a> {
    let mut parts = line.splitn(2, '=');
    let name = parts.next().unwrap_or("").trim();
    let value = parts.next().unwrap_or("").trim();
    Config { name, value }
}

fn main() {
    let line = String::from("host = 127.0.0.1");
    let cfg = parse(&line);
    println!("{} = {}", cfg.name, cfg.value); // host = 127.0.0.1
    // line 不能比 cfg 先死
}
```

注意 `parse` 完全靠规则 2 自动消除——但其实它满足"单一输入引用"，所以下面这版
**等价且更简洁**，编译器会自己补 `'a`：

```rust
fn parse(line: &str) -> Config<'_> { /*...*/ }
```

`'_` 是"省略占位符"，表示"这里有个生命周期，我懒得命名"。

## 3.8 多生命周期：更精确的约束

当函数有多个引用且返回值只跟其中部分相关时，分开标注能放宽约束：

```rust
// 返回值只与 x 相关，与 y 无关
fn first_word<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
    x.split_whitespace().next().unwrap_or("")
}
```

这样调用方知道：返回值能活到 `x` 死，跟 `y` 没关系，用起来更宽松。

## 3.9 常见陷阱

### 陷阱 1：返回局部变量的引用

```rust
// ✗ 编译错误：返回的引用指向局部 s
fn make() -> &str {
    let s = String::from("local");
    &s[..]
}
```

修复：返回 `String`（让所有权交还），或返回 `&'static str`（如果是字面量）。

### 陷阱 2：在结构体里塞了 `&str` 却想长期持有

```rust
struct Cache<'a> { items: Vec<&'a str> }

fn main() {
    let mut cache = Cache { items: vec![] };
    {
        let s = String::from("temp");
        // cache.items.push(&s); // ✗ s 活得不够久
    }
}
```

通常的修复是改用 `String`，把数据"_owned"进 `Cache`。

### 陷阱 3：以为 `'static` 能"救活"局部变量

```rust
// ✗ 'static 不能凭空延长运行期生命
fn bad() -> &'static str {
    let s = String::from("local");
    &s[..]
}
```

`'static` 是**约束**，不是"转换"——`s` 本身就活不到 `'static`，标注它只会让
编译器更确信地拒绝你。

## 3.10 练习

1. 给下面函数补全生命周期标注，并解释为什么需要两个不同的 `'a` / `'b`：
   ```rust
   fn split_at(s: &str, mid: usize) -> (&str, &str) {
       (&s[..mid], &s[mid..])
   }
   ```

2. 下面结构体定义有问题吗？为什么？
   ```rust
   struct Parser { text: &str }
   ```

3. 解释为什么 TaskFlow 项目里**几乎看不到 `'a`**。
   提示：看 `Task`、`TaskService`、`Store` 的字段类型。

4. 写一个 `fn longest_in<'a>(xs: &'a [&'a str]) -> Option<&'a str>`，
   返回切片里最长的字符串。思考：为什么只需要一个 `'a`？

## 3.11 小结

| 概念 | 一句话 |
|------|--------|
| 生命周期 | 借用有效的作用域，编译期标注 |
| `&'a T` | 这个引用至少活到 `'a` |
| 消除规则 | 单输入→输出；`&self`→输出；否则手写 |
| `'static` | 整个程序运行期；别滥用 |
| 惯用法 | 能 own 就别借，结构体少持有引用 |

> 下一章我们暂时告别"内存规则"，去看 Rust 的字符串家族和标准集合——
> 这些是写"真实程序"的砖块。

---

[← 第 2 章](./02_borrowing_slices.md) | [下一章 →](./04_strings_collections.md)

---

📧 联系作者：pebblerwon@qq.com
