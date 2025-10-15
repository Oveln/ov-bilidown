use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Bilibili视频ID (如 BV1NfxMedEU6)
    #[arg(short, long)]
    pub bvid: Option<String>,

    /// 下载目录
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,

    /// Cookie文件路径
    #[arg(short, long)]
    pub cookie_file: Option<String>,

    /// 是否只获取视频信息而不下载
    #[arg(long)]
    pub info_only: bool,
    
    /// 增加日志详细程度 (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    /// 安静模式，只显示错误
    #[arg(short = 's', long)]
    pub quiet: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub bvid: Option<String>,
    pub output_dir: PathBuf,
    pub cookie_file: String,
    pub info_only: bool,
}

impl AppConfig {
    pub fn new(cli: Cli) -> Self {
        let output_dir = cli.output_dir.unwrap_or_else(|| {
            dirs::download_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
        });
        
        let cookie_file = cli.cookie_file.unwrap_or_else(|| {
            let mut config_dir = dirs::config_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            config_dir.push("ov-bilidown");
            config_dir.push("cookies.txt");
            config_dir.to_string_lossy().to_string()
        });

        Self {
            bvid: cli.bvid,
            output_dir,
            cookie_file,
            info_only: cli.info_only,
        }
    }
}