use std::ffi::OsString;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "note-porter")]
#[command(version = "0.1.0")]
#[command(about = "笔记迁移工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// 配置文件路径
    #[arg(long,short,global = true,default_value = "./config.toml")]
    pub config_file:Option<OsString>,
}

#[derive(Debug,Subcommand)]
pub enum Commands {
    /// 笔记导入
    Import {
        /// 导入数据源, 可选值: notion
        #[arg(long, default_value = "notion")]
        to: String,
    },
    /// 笔记导出
    Export {
        /// 导出数据源, 可选值: wiz
        #[arg(long, default_value = "wiz")]
        from: String,
        /// 输出目录
        #[arg(long, required = true)]
        output_dir: String,
    }
}