//! ## 练习 2: 泛型缓存 Cache<K, V>
//!
//! ### 学习目标
//!
//! - 掌握多类型参数的泛型：`<K, V>`（Key + Value）
//! - 初步接触生命周期：`get` 方法返回引用时需要标注生命周期
//! - 理解「借用」在方法签名中的体现
//!
//! ### 概念讲解
//!
//! #### 双泛型参数
//!
//! 泛型可以有多个参数，用逗号分隔。HashMap 就是 `<K, V>`：
//!
//! ```rust,ignore
//! struct Cache<K, V> {
//!     data: HashMap<K, V>,
//! }
//! ```
//!
//! #### 返回引用与生命周期
//!
//! 当方法返回对内部数据的引用时，编译器需要知道「这个引用活多久」。
//!
//! ```rust,ignore
//! // 这个方法签名：
//! fn get(&self, key: &K) -> Option<&V>
//!
//! // 编译器通过「生命周期省略规则」自动推断为：
//! fn get<'a>(&'a self, key: &'a K) -> Option<&'a V>
//! ```
//!
//! 规则：返回的引用生命周期与 `&self` 相同（不能超过 self 的存活时间）。
//! 大多数情况下编译器能自动推断（省略规则），但理解原理很重要。
//!
//! ### 你的任务
//!
//! 实现 `Cache<K, V>` 结构体，让所有测试通过。
//! 注意 `get` 方法返回 `Option<&V>` — 思考为什么返回引用而不是值。

use std::collections::HashMap;
use std::hash::Hash;

// ────────────── 实现区域 ──────────────

/// 一个泛型缓存结构，用 HashMap 存储 Key-Value 对。
///
/// K 和 V 可以是不同的类型：
/// - Cache<String, i32>          字符串 → 整数
/// - Cache<i64, Vec<String>>     整数 → 字符串列表
///
/// K 必须实现 Hash + Eq trait（HashMap 的要求）
pub struct Cache<K, V> {
    // 提示：内部用一个 HashMap<K, V> 存储
    data: HashMap<K, V>,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    /// 创建一个空缓存
    pub fn new() -> Self {
        // todo!("用 HashMap::new() 创建空缓存")
        Cache {
            data: HashMap::new(),
        }
    }

    /// 插入或更新一个键值对。
    /// 如果 key 已存在，旧值会被覆盖。
    ///
    /// 提示：HashMap 的 insert 方法返回 Option<V>（旧值）
    ///       本方法不需要返回旧值，直接忽略 insert 的返回值即可
    pub fn insert(&mut self, key: K, value: V) {
        // todo!("调用 self.data 的 insert 方法")
        self.data.insert(key, value);
    }

    /// 根据 key 获取值的引用，返回 Some(&V)。
    /// 如果 key 不存在，返回 None。
    ///
    /// 💡 思考：为什么返回 &V 而不是 V？
    ///    如果返回 V（值），就需要转移所有权，缓存里的数据就没了。
    ///    返回 &V 只是「借用」，缓存里的数据仍然在。
    ///
    /// 提示：HashMap 的 get 方法返回 Option<&V>
    pub fn get(&self, key: &K) -> Option<&V> {
        // todo!("调用 self.data 的 get 方法");
        self.data.get(key)
    }

    /// 删除指定 key，返回被删除的值。
    /// 如果 key 不存在，返回 None。
    ///
    /// 提示：HashMap 的 remove 方法返回 Option<V>
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // todo!("调用 self.data 的 remove 方法")
        self.data.remove(key)
    }

    /// 返回缓存中的键值对数量
    pub fn len(&self) -> usize {
        // todo!("调用 self.data 的 len 方法")
        self.data.len()
    }

    /// 判断缓存是否为空
    pub fn is_empty(&self) -> bool {
        // todo!("调用 self.data 的 is_empty 方法")
        self.data.is_empty()
    }

    /// 清空缓存中的所有数据
    pub fn clear(&mut self) {
        // todo!("调用 self.data 的 clear 方法")
        self.data.clear();
    }
}

pub struct MyCache<K, V> {
    data: HashMap<K, V>,
}
impl<K: Eq + Hash, V> MyCache<K, V> {
    pub fn new() -> Self {
        MyCache {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }
}
// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_is_empty() {
        let cache: Cache<String, i32> = Cache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = Cache::new();

        cache.insert("name".to_string(), 42);
        cache.insert("age".to_string(), 25);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"name".to_string()), Some(&42));
        assert_eq!(cache.get(&"age".to_string()), Some(&25));
        assert_eq!(cache.get(&"missing".to_string()), None);
    }

    #[test]
    fn test_get_returns_reference() {
        // 这个测试验证 get 返回的是引用，而不是值

        let mut cache = Cache::new();
        cache.insert("key1".to_string(), "hello world".to_string());

        // get 返回 &String，不转移所有权
        let value: &String = cache.get(&"key1".to_string()).unwrap();
        assert_eq!(value, "hello world");

        // 缓存里的数据仍然存在
        assert_eq!(cache.len(), 1);

        // 可以再次 get（因为只是借用，没有移走）
        let value2: &String = cache.get(&"key1".to_string()).unwrap();
        assert_eq!(value2, "hello world");
    }

    #[test]
    fn test_insert_overwrites() {
        let mut cache = Cache::new();

        cache.insert("key".to_string(), 1);
        assert_eq!(cache.get(&"key".to_string()), Some(&1));

        // 相同 key 会覆盖
        cache.insert("key".to_string(), 999);
        assert_eq!(cache.get(&"key".to_string()), Some(&999));
        assert_eq!(cache.len(), 1); // 数量还是 1，没有增加
    }

    #[test]
    fn test_remove() {
        let mut cache = Cache::new();
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);

        // remove 返回被删除的值
        let removed = cache.remove(&"a".to_string());
        assert_eq!(removed, Some(1));
        assert_eq!(cache.len(), 1);

        // 再删一次已删除的 key，返回 None
        let removed_again = cache.remove(&"a".to_string());
        assert_eq!(removed_again, None);
    }

    #[test]
    fn test_clear() {
        let mut cache = Cache::new();
        cache.insert(1, "a".to_string());
        cache.insert(2, "b".to_string());
        cache.insert(3, "c".to_string());

        assert_eq!(cache.len(), 3);

        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_different_generic_types() {
        // 同一个 Cache 泛型可以实例化为不同的具体类型

        // String → i32
        let mut cache1 = Cache::new();
        cache1.insert("one".to_string(), 1);
        assert_eq!(cache1.get(&"one".to_string()), Some(&1));

        // i32 → Vec<String>
        let mut cache2 = Cache::new();
        cache2.insert(1, vec!["a".to_string(), "b".to_string()]);
        let val = cache2.get(&1).unwrap();
        assert_eq!(val, &vec!["a".to_string(), "b".to_string()]);
    }
}
