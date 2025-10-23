use chrono;
use clap::Parser;
use futures::future;
use log::{debug, info, warn};

use ov_bilidown::{
    VideoBasicInfo,
    config::{AppConfig, Cli},
    error::Result,
    subscription::Subscription,
    user::User,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志系统
    if cli.quiet {
        // 如果是安静模式，只显示错误
        let _ = env_logger::Builder::new()
            .filter_level(log::LevelFilter::Error)
            .try_init();
    } else {
        // 根据详细程度设置日志级别
        let level = match cli.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            2 => log::LevelFilter::Trace,
            _ => log::LevelFilter::Trace,
        };

        let _ = env_logger::Builder::new()
            .filter_level(level)
            .format(|buf, record| {
                use std::io::Write;
                writeln!(
                    buf,
                    "[{} {}] {}",
                    chrono::Local::now().format("%H:%M:%S%.3f"),
                    record.level(),
                    record.args()
                )
            })
            .try_init();
    }

    info!("应用启动");

    let config = AppConfig::new(cli)?;
    debug!(
        "配置已加载: output_dir={:?}, cookie_file={}",
        config.output_dir,
        config.cookie_file.to_string_lossy()
    );

    let user = User::ensure_user(&config).await?;

    match config.bvid {
        Some(ref bvid) => {
            let subscription = Subscription {
                bvid: bvid.clone(),
                title: Some("{title}".to_string()),
                artist: None,
                album: None,
            };
            subscription
                .download(&user, &config.output_dir, config.info_only)
                .await?
        }
        None => {
            let tasks = config
                .subscriptions
                .iter()
                .enumerate()
                .map(|(index, subscription)| {
                    let user = &user;
                    let output_dir = config.output_dir.as_path();
                    let info_only = config.info_only;
                    async move {
                        let title = subscription.title.clone().unwrap_or_default();
                        info!("开始处理订阅: {}:{}", index, title);
                        match VideoBasicInfo::new_from_subscription(user, &subscription).await {
                            Ok(videos) => {
                                info!(
                                    "订阅 {} 获取到视频: {} ({} - {})",
                                    index, videos.title, videos.owner.name, videos.bvid
                                );
                                if let Err(e) =
                                    subscription.download(&user, output_dir, info_only).await
                                {
                                    warn!("订阅 {}:{} 处理失败: {}", index, title, e);
                                } else {
                                    info!("订阅 {}:{} 处理完成", index, title);
                                }
                            }
                            Err(e) => {
                                warn!("订阅 {}:{} 获取视频失败: {}", index, title, e);
                            }
                        }
                    }
                });
            future::join_all(tasks).await;
        }
    }
    Ok(())
}
