# 第 9 章：并发与异步编程

## 本章目标

- 理解 `Send` / `Sync` 标记 trait
- 学会用 `std::thread` 起线程、`mpsc` 通道通信
- 掌握 `Mutex` / `RwLock` 共享可变状态
- 理解 `async` / `await` 与 Future 模型
- 用 `tokio` 写一个简单的并发 HTTP 请求示例
- 解释 TaskFlow 为何是同步、单线程的

## 9.1 Rust 的并发哲学

> "Fearless concurrency"——无畏并发。

Rust 的所有权 + 借用规则**在编译期**就阻止了大部分数据竞争：

- `Rc<T>` 非 `Send`，编译期阻止跨线程共享 → 改用 `Arc`
- `&mut T` 唯一性 → 多线程要么 `Arc<Mutex<T>>`，要么通过 channel 传值

## 9.2 `Send` 与 `Sync`

两个标记 trait（marker trait，无方法）：

| trait | 含义 |
|-------|------|
| `Send` | 类型可以**安全地移动**到另一个线程 |
| `Sync` | 类型可以**安全地被多线程共享引用** `&T` |

关系：`T: Sync` ⇔ `&T: Send`。

- `i32`、`String`、`Vec<T>`（当 T: Send）：`Send + Sync`
- `Rc<T>`：既不 Send 也不 Sync（计数非原子）
- `Arc<T>`：当 T: Sync 时是 Send + Sync
- `RefCell<T>`：非 Sync（运行期借用检查非线程安全）
- `Mutex<T>`：当 T: Send 时是 Send + Sync
- `Raw pointers`：既不 Send 也不 Sync

**绝大多数类型自动推导**，只有原始指针、`Rc`、`RefCell` 等少数例外需要手动 `unsafe impl`。

## 9.3 起线程：`std::thread::spawn`

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for i in 0..3 {
            println!("子线程 {i}");
        }
    });

    for i in 0..3 {
        println!("主线程 {i}");
    }

    handle.join().unwrap(); // 等子线程结束
}
```

### `move` 闭包跨线程

```rust
let data = vec![1, 2, 3];
let handle = thread::spawn(move || { // ← move 必须
    println!("{:?}", data);
});
// 这里 data 不可用了，已被搬到子线程
handle.join().unwrap();
```

不加 `move`，闭包借 `data`，但子线程可能比 `data` 活得久——编译错误。

## 9.4 通道：`mpsc` 消息传递

"不要通过共享内存来通信，而要通过通信来共享内存。"

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    let producer = thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    let consumer = thread::spawn(move || {
        for msg in rx { // rx 是迭代器
            println!("收到 {msg}");
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

### 多生产者：`clone`

```rust
let (tx, rx) = mpsc::channel();
let tx2 = tx.clone();
thread::spawn(move || tx.send("A").unwrap());
thread::spawn(move || tx2.send("B").unwrap());
```

### 同步 vs 异步通道

- `channel()`：异步，无界缓冲
- `sync_channel(n)`：同步，缓冲 n，满了阻塞 send

## 9.5 共享可变状态：`Mutex` / `RwLock`

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..10).map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || {
            let mut num = c.lock().unwrap(); // 上锁，返回 MutexGuard
            *num += 1;
            // guard 离开作用域自动解锁
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
    println!("结果 = {}", *counter.lock().unwrap()); // 10
}
```

要点：
- `Arc<Mutex<T>>` 是多线程共享可变的标准组合
- `lock()` 返回 `Result<MutexGuard, _>`， poisoned 时返回 `Err`
- `MutexGuard` 实现 `Deref` / `DerefMut`，离开作用域自动解锁（RAII）

### `RwLock`：读写锁

读多写少用 `RwLock`：多个读可并发，写独占。

```rust
use std::sync::RwLock;
let lock = RwLock::new(5);
{
    let r1 = lock.read().unwrap();
    let r2 = lock.read().unwrap(); // 多个读 OK
    println!("{r1} {r2}");
}
{
    let mut w = lock.write().unwrap(); // 写独占
    *w += 1;
}
```

### `Mutex` vs `RwLock` 取舍

- `Mutex` 更轻量，临界区短时优先
- `RwLock` 读多写少时更并发，但开销略大
- **避免在锁里做耗时操作**（IO、长计算、嵌套锁）

## 9.6 异步编程：`async` / `await`

并发（多任务）≠ 并行（多核）。`async` 是**协作式并发**：在单线程内通过状态机
切换任务，等待 IO 时不阻塞线程。

```rust
async fn fetch(url: &str) -> String {
    // 假装发请求
    format!("来自 {url} 的数据")
}

async fn main_async() {
    let r1 = fetch("a").await;
    let r2 = fetch("b").await;
    println!("{r1} {r2}");
}
```

- `async fn` 返回 `impl Future<Output = ...>`
- `.await` 等待 Future 完成
- Future 是**惰性**的——必须有人 poll 它才会执行

## 9.7 运行时：`tokio`

标准库只提供 `Future` trait，不提供运行时。最流行的是 `tokio`：

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() {
    println!("hello async");
}
```

`#[tokio::main]` 把 `async fn main` 展开成同步 `fn main` 启动运行时。

### 并发任务：`tokio::spawn`

```rust
use tokio;

#[tokio::main]
async fn main() {
    let h1 = tokio::spawn(async {
        // 模拟 IO
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        1
    });
    let h2 = tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        2
    });
    let (a, b) = tokio::join!(h1, h2); // 并发等待
    println!("{a} {b}");
}
```

两个任务**并发**执行，总耗时约 100ms 而非 200ms。

### 异步 HTTP 请求（`reqwest`）

```toml
reqwest = { version = "0.12", features = ["json"] }
```

```rust
use reqwest;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/get").await?;
    let text = resp.text().await?;
    println!("{text}");
    Ok(())
}
```

## 9.8 `Send + 'static` 的约束

`tokio::spawn` 要求 future 是 `Send + 'static`：
- `Send`：可能被调度到其它线程（默认 multi-thread runtime）
- `'static`：不能借用局部变量（任务可能比函数活得久）

```rust
#[tokio::main]
async fn main() {
    let data = String::from("hi");
    // tokio::spawn(async { println!("{data}"); }); // ✗ 借用了 data
    tokio::spawn(async move { println!("{data}"); }); // ✓ move
}
```

> 用 `tokio::task::spawn_local` 可在 `LocalSet` 里跑非 Send future（如 GUI）。

## 9.9 同步 vs 异步：什么时候用哪个

| 场景 | 选择 |
|------|------|
| CPU 密集（并行计算） | `std::thread` 或 `rayon` |
| 大量 IO（网络、磁盘） | `async` + `tokio` |
| 简单后台任务 | `std::thread` |
| Web 服务器、代理、爬虫 | `async` |
| CLI 小工具 | 同步即可 |

> 📖 对照：TaskFlow 是**同步单线程** CLI。它做的是本地 JSON 读写，
> 没有 IO 阻塞压力，引入 `async` 反而徒增复杂度。一般经验：
> **没有大量并发 IO 就别上 async**。

## 9.10 `select!`：等最先完成的任务

```rust
use tokio::{select, time::sleep};
use std::time::Duration;

#[tokio::main]
async fn main() {
    select! {
        _ = sleep(Duration::from_millis(50)) => println!("50ms 先到"),
        _ = sleep(Duration::from_millis(100)) => println!("100ms 先到"),
    }
}
```

类似 Go 的 `select`。

## 9.11 常见陷阱

### 陷阱 1：死锁

```rust
// 两个锁，相反顺序获取 → 死锁
let lock1 = Mutex::new(0);
let lock2 = Mutex::new(0);
// 线程 A：lock1 → lock2
// 线程 B：lock2 → lock1
```

**经验**：全局规定加锁顺序；尽量不嵌套锁。

### 陷阱 2：`.await` 持有锁

```rust
let data = mutex.lock().unwrap();
something_async(data).await; // ✗ 持有锁跨 await，可能阻塞其它任务甚至死锁
```

修复：先把数据取出，drop guard，再 await：

```rust
let value = {
    let data = mutex.lock().unwrap();
    data.clone()
};
something_async(value).await;
```

或用 `tokio::sync::Mutex`（专为 async 设计）。

### 陷阱 3：忘记 `Arc`

```rust
let m = Mutex::new(0);
let h1 = thread::spawn(move || *m.lock().unwrap()); // m 被 move 走
let h2 = thread::spawn(move || *m.lock().unwrap()); // ✗ m 已被 move
```

要 `Arc::new(Mutex::new(0))`。

### 陷阱 4：以为 `async` 自动并行

```rust
// 串行：每个 await 都阻塞
let a = fetch("a").await;
let b = fetch("b").await;
```

要并发用 `tokio::join!` 或先 `spawn`：

```rust
let (a, b) = tokio::join!(fetch("a"), fetch("b"));
```

## 9.12 练习

1. 用 `std::thread` + `Arc<Mutex<Vec<i32>>>` 启动 4 个线程，每个 push 100 个数，
   主线程 join 后打印总数。

2. 用 `mpsc::channel` 实现一个生产者-消费者：1 个生产者发 1..=10，2 个消费者
   各自打印收到的值。

3. 用 `tokio` + `reqwest` 并发请求 3 个 URL，用 `tokio::join!` 等所有结果，
   打印每个响应的长度。

4. 解释：为什么 `Rc<T>` 不能跨线程？改成 `Arc<T>` 后为什么安全？

## 9.13 小结

| 概念 | 一句话 |
|------|--------|
| `Send` / `Sync` | 标记 trait，编译期保证线程安全 |
| `thread::spawn` | 起线程，闭包需 `move` + `Send + 'static` |
| `mpsc` | 多生产者单消费者通道 |
| `Mutex<T>` / `RwLock<T>` | 互斥 / 读写锁 |
| `Arc<Mutex<T>>` | 多线程共享可变标准组合 |
| `async/await` | 协作式并发，零成本状态机 |
| `tokio` | 最流行的 async 运行时 |
| 别在 `.await` 上持有同步锁 | 会阻塞 runtime |

> 下一章我们回到语言层面：**模式匹配与运算符重载**——
> 让你的 enum/struct 用起来像内置类型一样自然。

---

[← 第 8 章](./08_smart_pointers.md) | [下一章 →](./10_pattern_matching.md)

---

📧 联系作者：pebblerwon@qq.com
