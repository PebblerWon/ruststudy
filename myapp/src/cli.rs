use clap::{Parser, Subcommand};

use crate::models::{Priority, Status};

#[derive(Parser)]
#[command(
    version,
    name = "taskflow",
    about = "命令行任务管理工具",
    long_about = "TaskFlow —— 轻量级命令行任务管理工具\n\n\
        支持任务的增删改查、搜索、统计和导出。\n\
        数据存储在 ~/.taskflow/data.json，使用 UUID 作为任务 ID。",
    after_help = r#"使用示例:
        taskflow add "学习Rust" -p high
        taskflow list --status todo
        taskflow update <id> --status done
        taskflow delete <id> --force
        taskflow search "Rust"
        taskflow stats  
        taskflow export -o tasks.csv"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 创建新任务
    Add {
        /// 任务标题
        title: String,
        /// 任务描述
        #[arg(short, long)]
        description: Option<String>,
        /// 优先级（high/medium/low，默认 medium）
        #[arg(short, long, default_value = "medium")]
        priority: Priority,
        /// 标签（逗号分隔，如 -t rust,study）
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
        /// 截止日期（YYYY-MM-DD 格式）
        #[arg(long)]
        due: Option<String>,
    },

    /// 列出任务
    List {
        /// 按状态筛选（todo/in_progress/done）
        #[arg(short, long)]
        status: Option<Status>,
        /// 按优先级筛选（high/medium/low）
        #[arg(short, long)]
        priority: Option<Priority>,
        /// 按标签筛选（模糊匹配）
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// 更新任务
    Update {
        /// 任务 ID（支持前缀匹配，最少 1 位）
        id: String,
        /// 新标题
        #[arg(long)]
        title: Option<String>,
        /// 新状态（todo/in_progress/done）
        #[arg(long)]
        status: Option<Status>,
        /// 新优先级（high/medium/low）
        #[arg(long)]
        priority: Option<Priority>,
        /// 新标签（逗号分隔，替换原有标签）
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
    },

    /// 删除任务
    Delete {
        /// 任务 ID（支持前缀匹配）
        id: String,
        /// 跳过确认提示，直接删除
        #[arg(short, long)]
        force: bool,
    },

    /// 搜索任务
    Search {
        /// 搜索关键字（匹配标题和描述，大小写不敏感）
        keyword: String,
    },

    /// 查看统计
    Stats,

    /// 导出数据
    Export {
        /// 导出格式（csv，默认 csv）
        #[arg(long, default_value = "csv")]
        format: String,
        /// 输出文件路径（不指定则输出到终端）
        #[arg(short, long)]
        output: Option<String>,
    },
}
