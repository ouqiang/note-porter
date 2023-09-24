use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use clap::Parser;
use log::{error, LevelFilter};
use simple_logger::SimpleLogger;
use note_porter::cli;
use note_porter::cli::Cli;
use note_porter::config;
use note_porter::exporter::Exporter;
use note_porter::thirdparty::wiz::Wiz;

fn main() -> anyhow::Result<()>  {
    init_logger();
    let args = Cli::parse();
    let conf = load_config(args.config_file.unwrap())?;

    match args.command {
        cli::Commands::Export{from, output_dir} => {
            match from.as_str() {
                "wiz" => {
                    let exporter = Wiz::new(&conf.wiz)?;
                    exporter.export(output_dir)?;
                }
                _ =>  error!("不支持的来源: {}", from)
            }
        },
        cli::Commands::Import{to: _} => {
            error!("导入暂不支持")
        }
    }

    Ok(())
}

// 加载配置
fn load_config<T>(config_file: T) -> anyhow::Result<config::Config>
where T: AsRef<OsStr> + AsRef<Path>
{
    // 尝试加载顺序
    // 绝对路径
    // 当前目录
    // 可执行文件目录
    let mut config_path_buf;
    if Path::new(&config_file).is_absolute() {
        config_path_buf = PathBuf::new().join(&config_file);
    } else {
        config_path_buf = env::current_dir()?.join(&config_file);
        if !config_path_buf.exists() {
            config_path_buf = get_execute_dir()?.join(&config_file);
        }
    }
    let conf = config::Config::parse(config_path_buf)?;

    Ok(conf)
}

// 获取可执行文件所在目录
fn get_execute_dir() -> anyhow::Result<PathBuf> {
    let mut path_buf = env::current_exe()?;
    if !path_buf.pop() {
        return Err(anyhow!("获取可执行文件目录错误: {:?}", path_buf.into_os_string()))
    };

    Ok(path_buf)
}

fn init_logger() {
    SimpleLogger::new().
        with_level(LevelFilter::Info).
        with_colors(true).
        init().unwrap();
}