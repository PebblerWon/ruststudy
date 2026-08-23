# 第 13 章：Unsafe Rust 基础

## 本章目标

- 理解 `unsafe` 的真正含义（不是"关闭检查"，是"我保证"）
- 掌握 unsafe 能做的五件事
- 知道何时该用、何时不该用
- 学会读 unsafe 块，能看懂标准库与第三方库的 unsafe
- 解释 TaskFlow 为何完全不碰 unsafe

## 13.1 为什么需要 unsafe

Rust 的安全保证来自静态规则：所有权、借用、生命周期。但有些操作**编译器无法静态验证**：

- 调用 C 函数（FFI）
- 解引用裸指针（可能是空、可能是悬垂）
- 访问可变 static
- 实现"非安全 trait"（如手动 `Send`/`Sync`）
- 访问 union 字段

这些操作不一定错，只是编译器无法证明它对。`unsafe` 把责任从编译器转移到**你**：
"我手动检查过了，保证不会违反 Rust 的不变量。"

> unsafe **不会**关闭借用检查、不会让 `&mut T` 突然能多个、不会让 GC 出现。
> 它只解锁上述五件事。

## 13.2 unsafe 能做的五件事

```rust
// 1. 解引用裸指针
let x = 5;
let r1 = &x as *const i32;     // 裸指针（*const 只读，*mut 可写）
let r2 = &mut x as *mut i32;   // ⚠ 这里只是借用，没 unsafe 也能创建
unsafe {
    println!("{}", *r1);        // 解引用必须 unsafe
}

// 2. 调用 unsafe 函数
unsafe fn dangerous() -> i32 { 1 }
unsafe { dangerous(); }

// 3. 实现/调用 unsafe trait
unsafe trait Foo { fn bar(&self); }
unsafe impl Foo for i32 { fn bar(&self) {} }

// 4. 访问可变 static
static mut COUNTER: i32 = 0;
unsafe { COUNTER += 1; }

// 5. 访问 union 字段
union MyUnion { a: i32, b: f32 }
let u = MyUnion { a: 1 };
unsafe { println!("{}", u.a); }
```

## 13.3 裸指针 `*const T` / `*mut T`

- 裸指针**不保证**非空、不保证有效、不实现自动清理
- 创建裸指针可以不用 unsafe（只是"指过去"），但**解引用**必须 unsafe
- 裸指针**没有**借用规则，可以同时有多个 `*mut` 指同一处

```rust
let mut v = vec![1, 2, 3];
let ptr = v.as_mut_ptr();        // *mut i32
unsafe {
    *ptr = 100;
    println!("{}", *ptr.add(1)); // 2，offset 用字节算
}
```

### 何时用裸指针

- FFI（与 C 互操作）
- 实现自研容器（`Vec`/`HashMap` 内部就是 unsafe）
- 性能极致优化（罕见）

## 13.4 `unsafe fn` 与 `unsafe block`

```rust
// 函数标记 unsafe：调用方必须 unsafe { f() }
unsafe fn raw(x: *const i32) -> i32 { *x }

// 函数体里用 unsafe 块
fn safe_wrapper(p: *const i32) -> i32 {
    if p.is_null() { return 0; }
    unsafe { *p }   // 局部 unsafe
}
```

**最佳实践**：尽量把 unsafe 范围缩到最小，并用安全函数包装。对外提供安全 API，
unsafe 留在内部。

## 13.5 一个安全包装的例子

```rust
fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = slice.len();
    assert!(mid <= len);
    let ptr = slice.as_mut_ptr();
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

这是标准库 `slice::split_at_mut` 的简化版。普通 Rust 写不出这个函数——
借用规则要求 `&mut` 唯一，但这里我们要两个不重叠的 `&mut [i32]`。
unsafe 让我们绕过检查，**前提是** `mid <= len`（用 `assert!` 保证）。

## 13.6 FFI 调用 C

```rust
extern "C" {
    fn abs(x: i32) -> i32;
}

fn main() {
    unsafe { println!("{}", abs(-5)); }
}
```

`extern "C"` 声明 C 函数签名，调用必须 unsafe——编译器不知道 C 函数是否安全。

### 反向：让 Rust 函数被 C 调

```rust
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
```

`#[no_mangle]` 防止 Rust 改名。

## 13.7 手动实现 `Send`/`Sync`

某些类型编译器无法自动判定，但你能保证它线程安全：

```rust
struct MyPtr(*mut i32);
unsafe impl Send for MyPtr {}  // 我保证：跨线程移动这个指针安全
unsafe impl Sync for MyPtr {}  // 我保证：跨线程共享 &MyPtr 安全
```

**警告**：错误实现 `Send`/`Sync` 会导致数据竞争，且编译器不会帮你检查。
绝大多数情况**别手动 impl**。

## 13.8 何时真的需要 unsafe

| 场景 | 是否必须 unsafe |
|------|-----------------|
| FFI 调 C 库 | ✓ |
| 自研容器（Vec/HashMap/链表） | ✓（部分内部） |
| 性能极致（绕过边界检查） | 可选，但有 `get_unchecked` 等安全替代 |
| 调用 OS 系统调用 | ✓ |
| 与硬件/裸机交互（嵌入式） | ✓ |
| 普通 CRUD 应用 | ✗ 完全不需要 |

> 📖 对照：TaskFlow 是普通 CRUD CLI，**零 unsafe**。99% 的应用代码也不需要。
> 看到 unsafe 别慌，但写之前先想清楚有没有安全方案。

## 13.9 unsafe 的"超级规则"

即使 unsafe 块内，你也不能违反以下"超级不变量"（违反即 UB，未定义行为）：

1. **数据竞争**：多线程并发写同一位置（无同步）
2. **悬垂指针解引用**：访问已释放内存
3. **无效值**：如 `bool` 不是 0/1、`char` 超出 Unicode 范围、`&T` 为空
4. **借用规则违反**：同时多个 `&mut T`（即使通过裸指针构造）
5. **未对齐访问**：类型要求对齐但地址不对齐

违反任一即 UB，编译器会做不可预测的优化（删除代码、重排指令），后果不可控。

## 13.10 调试与工具

- **Miri**：Rust 的 UB 检测器，能跑出大多数 unsafe 错误
  ```bash
  rustup +nightly component add miri
  cargo +nightly miri test
  ```
- **AddressSanitizer / ThreadSanitizer**：rustc 集成
  ```bash
  RUSTFLAGS="-Zsanitizer=address" cargo +nightly run
  ```
- **loom**：测试并发 unsafe 代码

> 写 unsafe 代码时**必跑 Miri**。它能在你写错时大喊一声。

## 13.11 常见陷阱

### 陷阱 1：以为 unsafe 关闭所有检查

```rust
let r: &mut i32;
unsafe { r = &mut 5; } // ✗ 仍然检查借用规则
```

unsafe 只解锁那五件事，不关借用检查。

### 陷阱 2：从 `&T` 转 `&mut T`

```rust
let x = 5;
let r = &x as *const i32 as *mut i32;
unsafe { *r = 10; } // ✗ UB！通过不可变引用改值
```

这是经典 UB。Rust 假设 `&T` 指向的数据不被改，编译器可能缓存读取结果。

### 陷阱 3：unsafe 边界设计不当

把 unsafe 暴露给外部，调用方容易踩坑。**封装成安全 API**——只在你 crate 内部 unsafe，
对外提供安全函数，并明确文档化"安全条件"。

### 陷阱 4：用裸指针绕过所有权，结果内存泄漏/双重释放

`Box::from_raw` / `Box::into_raw` 配对使用：

```rust
let b = Box::new(5);
let ptr = Box::into_raw(b);     // 释放所有权，不再自动 drop
unsafe { drop(Box::from_raw(ptr)); } // 必须手动回收
```

忘了 `from_raw` → 泄漏；多次 `from_raw` → 双重释放。

## 13.12 练习

1. 用 unsafe 实现一个 `fn first_two(slice: &[i32]) -> &[i32]`，返回前两个元素的切片。
   实际上安全版 `&slice[..2]` 就行，本题只为练习 `from_raw_parts`。

2. 用 `extern "C"` 调用 C 标准库的 `strlen`（链接 `libc` crate 或直接 `extern`），
   计算 `"hello"` 的长度。

3. 阅读 `Vec::push` 的源码（标准库），找出它用 unsafe 的部分。
   思考：为什么这一步必须 unsafe？

4. 解释：为什么 TaskFlow 项目里没有任何 unsafe 块。
   提示：所有"危险操作"都被标准库和第三方 crate（serde/clap/...）安全封装了。

## 13.13 小结

| 概念 | 一句话 |
|------|--------|
| unsafe | 把责任从编译器转给你 |
| 五件事 | 解引用裸指针/unsafe fn/unsafe trait/mut static/union |
| 安全包装 | unsafe 留内部，对外安全 API |
| 超级规则 | 数据竞争/悬垂/无效值/借用违反/未对齐 不可违反 |
| Miri | UB 检测器，写 unsafe 必跑 |
| 99% 应用 | 不需要 unsafe |

> 下一章我们把前面学的所有概念串起来，看 Rust 里常见的**设计模式与最佳实践**。

---

[← 第 12 章](./12_cargo_cfg.md) | [下一章 →](./14_design_patterns.md)

---

📧 联系作者：pebblerwon@qq.com
