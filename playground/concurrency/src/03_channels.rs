//! ## 练习 9: 用 Channel 做生产者-消费者
//!
//! ### 学习目标
//! - 学会用 `mpsc::channel` 创建消息通道
//! - 理解 `tx.send()` 和 `rx.recv()`
//! - 实现多线程单词频率统计器
//!
//! ### 背景
//!
//! Rust 的通道（Channel）实现了“通过通信来共享内存”的理念。
//! `mpsc` 代表 Multi-producer, Single-consumer（多生产者，单消费者）。
//!
//! ```rust,ignore
//! let (tx, rx) = mpsc::channel();
//! tx.send("Hello").unwrap(); // 发送数据
//! let msg = rx.recv().unwrap(); // 接收数据（阻塞）
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `count_words_in_thread` 函数，在一个新线程中统计给定文本的单词数。
//! 2. 将结果通过通道发送回主线程。
//! 3. 在主线程中汇总所有线程的结果。

// ────────────── 实现区域 ──────────────

use std::sync::mpsc;
use std::thread;

/// 统计一段文本中的单词数量（以空格分隔）
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// 在多个线程中并行统计多段文本的单词总数
pub fn parallel_word_count(texts: Vec<String>) -> usize {
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for text in texts {
        let tx_clone = tx.clone();
        
        let handle = thread::spawn(move || {
            let count = count_words(&text);
            // 提示：通过通道发送结果
            todo!("发送统计结果")
        });
        
        handles.push(handle);
    }

    // 关闭主线程的发送端，这样当所有子线程结束后，rx 才会知道没有更多消息了
    drop(tx);

    let mut total = 0;
    // 提示：从通道接收所有结果并累加
    todo!("接收并汇总结果")
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words_basic() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("rust is awesome"), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_parallel_word_count_correctness() {
        let texts = vec![
            "hello world".to_string(),
            "rust concurrency".to_string(),
            "message passing".to_string(),
        ];
        
        assert_eq!(parallel_word_count(texts), 6);
    }

    #[test]
    fn test_channel_communication() {
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        assert_eq!(rx.recv().unwrap(), 42);
    }
}
