mod config;
mod download;
mod error;
mod user;
mod video;
mod wbi;

use std::path::PathBuf;

use chrono;
use clap::Parser;
use config::{AppConfig, Cli};
use error::{BilidownError, Result};
use log::{debug, info, warn};

use user::User;
use video::VideoBasicInfo;

async fn donwload_single(
    bvid: &String,
    user: &User,
    output_dir: &PathBuf,
    info_only: bool,
) -> Result<()> {
    info!("开始处理视频: {}", bvid);

    debug!("正在获取视频信息...");
    let video = VideoBasicInfo::new_from_bvid(&user, &bvid).await?;
    info!(
        "视频信息获取成功: {} ({} - {})",
        video.title, video.owner.name, video.bvid
    );

    if info_only {
        // 仅显示视频信息
        info!("以信息模式运行，不下载音频");
        println!("视频信息:");
        println!("标题: {}", video.title);
        println!("UP主: {}", video.owner.name);
        println!("播放数: {}", video.stat.view);
        println!("时长: {}秒", video.duration);
        if let Some(pages) = &video.pages {
            println!("分P数: {}", pages.len());
            for page in pages {
                println!("  - P{}: {} ({}秒)", page.page, page.part, page.duration);
            }
        }
    } else {
        info!("开始下载音频到目录: {:?}", output_dir);
        // 下载音频
        video
            .download_best_quality_audios_to_file(&user, &output_dir)
            .await?;
        info!("下载完成!");
        println!("下载完成!");
    }
    Ok(())
}

async fn load_user(config: &AppConfig) -> Result<User> {
    // 从配置文件加载用户或新建用户
    let user = match User::new_from_file(&config.cookie_file).await {
        Ok(u) => {
            info!(
                "从文件加载用户信息: {}",
                &config.cookie_file.to_string_lossy()
            );
            u
        }
        Err(_) => {
            warn!("未找到现有cookie，正在进行二维码登录...");
            let u = User::new()
                .await
                .map_err(|e| BilidownError::LoginError(e.to_string()))?;
            u.save_to_file(&config.cookie_file)?;
            info!(
                "登录成功，cookie已保存到: {}",
                &config.cookie_file.to_string_lossy()
            );
            u
        }
    };
    Ok(user)
}
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

    let user = load_user(&config).await?;

    match config.bvid {
        Some(ref bvid) => {
            donwload_single(bvid, &user, &config.output_dir, config.info_only).await?
        }
        None => {
            for (index, subscription) in config.subscriptions.iter().enumerate() {
                info!("开始处理订阅: {}:{}", index, subscription.title);
                let videos = VideoBasicInfo::new_from_subscription(&user, &subscription).await?;
                info!(
                    "订阅 {} 获取到视频: {} ({} - {})",
                    index, videos.title, videos.owner.name, videos.bvid
                );
                donwload_single(
                    &subscription.bvid,
                    &user,
                    &config.output_dir,
                    config.info_only,
                )
                .await?;
                info!("订阅 {}:{} 处理完成", index, subscription.title);
            }
        }
    }
    Ok(())
}
