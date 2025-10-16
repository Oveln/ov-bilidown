use clap::Parser;
use config::ConfigError;
use dirs;
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub cookie_file: Option<PathBuf>,

    /// 订阅配置文件路径
    #[arg(short, long)]
    pub subscription_file: Option<PathBuf>,

    /// 是否只获取视频信息而不下载
    #[arg(long)]
    pub info_only: bool,

    /// 增加日志详细程度 (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// 安静模式，只显示错误
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub bvid: Option<String>,
    pub output_dir: PathBuf,
    pub cookie_file: PathBuf,
    pub info_only: bool,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub title: String,
    pub bvid: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Subscriptions {
    #[serde(default)]
    #[serde(rename = "sub")]
    subscriptions: Vec<Subscription>,
}

impl AppConfig {
    pub fn new(cli: Cli) -> Result<Self, ConfigError> {
        let output_dir = cli.output_dir.unwrap_or_else(|| {
            dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap())
        });

        let cookie_file = cli.cookie_file.unwrap_or_else(|| {
            let mut config_dir =
                dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
            config_dir.push("ov-bilidown");
            config_dir.push("cookies.txt");
            config_dir
        });

        let subscription_path = cli.subscription_file.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
            path.push("ov-bilidown");
            path.push("config.toml");
            path
        });
        let subscription_config = config::Config::builder()
            .add_source(config::File::from(subscription_path).required(false))
            .build()?;

        let subscriptions = subscription_config
            .try_deserialize::<Subscriptions>()
            .map_err(|err| {
                debug!("{}", err.to_string());
                ConfigError::Message(format!("配置文件解析出错!"))
            })?;

        Ok(Self {
            bvid: cli.bvid,
            output_dir,
            cookie_file,
            info_only: cli.info_only,
            subscriptions: subscriptions.subscriptions,
        })
    }
}
