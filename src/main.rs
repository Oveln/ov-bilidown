mod config;
mod download;
mod error;
mod user;
mod video;
mod wbi;

use clap::Parser;
use config::{AppConfig, Cli};
use error::{Result, BilidownError};
use log::{debug, info, warn, error};
use chrono;

use user::User;
use video::VideoBasicInfo;

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
    
    let config = AppConfig::new(cli);
    debug!("配置已加载: output_dir={:?}, cookie_file={}", config.output_dir, config.cookie_file);
    
    // 验证必需参数
    let bvid = config.bvid.ok_or_else(|| {
        error!("未提供BVID");
        BilidownError::ApiError("必须提供BVID。使用 -b 或 --bvid 指定视频ID".to_string())
    })?;
    info!("开始处理视频: {}", bvid);
    
    // 从配置文件加载用户或新建用户
    let user = match User::new_from_file(&config.cookie_file) {
        Ok(u) => {
            info!("从文件加载用户信息: {}", &config.cookie_file);
            u
        },
        Err(_) => {
            warn!("未找到现有cookie，正在进行二维码登录...");
            let u = User::new().await.map_err(|e| BilidownError::LoginError(e.to_string()))?;
            u.save_to_file(&config.cookie_file)?;
            info!("登录成功，cookie已保存到: {}", &config.cookie_file);
            u
        }
    };

    debug!("正在获取视频信息...");
    let video = VideoBasicInfo::new_from_bvid(&user, &bvid).await?;
    info!("视频信息获取成功: {} ({} - {})", video.title, video.owner.name, video.bvid);
    
    if config.info_only {
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
        info!("开始下载音频到目录: {:?}", config.output_dir);
        // 下载音频
        video
            .download_best_quality_audios_to_file(&user, config.output_dir.to_string_lossy().as_ref())
            .await?;
        info!("下载完成!");
        println!("下载完成!");
    }

    Ok(())
}
