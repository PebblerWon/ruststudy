//! ## 练习 13: 并发文件词频统计器（综合实战）
//!
//! ### 学习目标
//! - 用 `tokio::fs` 异步读取目录下多个文件
//! - 并发统计每个文件的单词频率
//! - 用 `Arc<Mutex<HashMap>>` 合并结果
//!
//! ### 背景
//!
//! 这是一个结合了 async、并发和共享状态的综合练习。
//! 我们将模拟一个场景：同时处理多个日志文件，统计其中出现频率最高的单词。
//!
//! ### 你的任务
//!
//! 1. 实现 `count_file_words` 函数，异步读取文件并返回单词计数 HashMap。
//! 2. 实现 `merge_counts` 函数，将多个 HashMap 合并为一个。
//! 3. 编写测试验证统计逻辑的正确性。

// ────────────── 实现区域 ──────────────

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;

/// 统计单个文件中的单词频率
pub async fn count_file_words2(path_arr: Vec<String>) -> std::io::Result<HashMap<String, usize>> {
    let res = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    for p in path_arr {
        let res = Arc::clone(&res);
        handles.push(tokio::spawn(async move {
            let content = fs::read_to_string(p).await.unwrap();

            let mut counts = res.lock().unwrap();
            for word in content.split_whitespace() {
                let word = word.to_lowercase();
                *counts.entry(word).or_insert(0) += 1;
            }
        }));
    }
    for handle in handles {
        // await 返回 Result：Err 表示任务 panic，不能丢弃否则错误会静默吞掉
        handle.await.expect("统计任务 panic");
    }
    // into_inner() 返回 LockResult（防锁中毒），所以这里有两个 unwrap：
    // 外层解 Arc::try_unwrap（此时克隆均已 drop，不会失败）
    Ok(Arc::try_unwrap(res).unwrap().into_inner().unwrap())
}

/// 统计单个文件中的单词频率
pub async fn count_file_words(path: &str) -> std::io::Result<HashMap<String, usize>> {
    let content = fs::read_to_string(path).await?;

    let mut counts = HashMap::new();
    for word in content.split_whitespace() {
        let word = word.to_lowercase();
        *counts.entry(word).or_insert(0) += 1;
    }

    Ok(counts)
}

/// 合并多个单词计数结果
pub fn merge_counts(all_counts: Vec<HashMap<String, usize>>) -> HashMap<String, usize> {
    let mut merged = HashMap::new();
    for counts in all_counts {
        for (word, count) in counts {
            *merged.entry(word).or_insert(0) += count;
        }
    }
    merged
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_count_words_in_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"hello world hello rust").await.unwrap();

        let file_path2 = dir.path().join("test2.txt");
        let mut file2 = File::create(&file_path2).await.unwrap();
        file2.write_all(b"hello tom hello jerry").await.unwrap();
        drop(file);
        drop(file2);

        let counts = count_file_words(file_path.to_str().unwrap()).await.unwrap();
        let counts2 = count_file_words(file_path2.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(counts.get("hello"), Some(&2));
        assert_eq!(counts.get("world"), Some(&1));
        assert_eq!(counts.get("rust"), Some(&1));
        let all = merge_counts(vec![counts, counts2]);
        assert_eq!(all.get("hello"), Some(&4));
    }

    #[test]
    fn test_merge_counts_logic() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), 1);

        let mut map2 = HashMap::new();
        map2.insert("a".to_string(), 2);
        map2.insert("b".to_string(), 1);

        let merged = merge_counts(vec![map1, map2]);

        assert_eq!(merged.get("a"), Some(&3));
        assert_eq!(merged.get("b"), Some(&1));
    }
}
