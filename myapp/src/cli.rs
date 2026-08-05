use clap::{Parser, Subcommand};

use crate::models::{Priority, Status};

#[derive(Parser)]
#[command(version, name = "taskflow", about = "命令行任务管理工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    ///创建新任务
    Add {
        /// 任务标题
        title: String,
        #[arg(short, long)]
        description: Option<String>,

        #[arg(short, long, default_value = "medium")]
        priority: Priority,

        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,

        #[arg(long)]
        due: Option<String>,
    },

    /// 列出任务
    List {
        #[arg(short, long)]
        status: Option<Status>,

        #[arg(short, long)]
        priority: Option<Priority>,

        #[arg(short, long)]
        tag: Option<String>,
    },

    /// 更新任务
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<Status>,
        #[arg(long)]
        priority: Option<Priority>,
    },

    /// 删除任务
    Delete {
        id: String,

        #[arg(short, long)]
        force: bool,
    },

    /// 搜索任务
    Search { keyword: String },

    /// 查看统计
    Stats,

    /// 导出数据
    Export {
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}
