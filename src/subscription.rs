use std::path::Path;

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::{VideoBasicInfo, user::User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub bvid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
}

impl Subscription {
    pub async fn download(&self, user: &User, output_dir: &Path, info_only: bool) -> Result<()> {
        let bvid = &self.bvid;
        info!("开始处理视频: {}", bvid);

        debug!("正在获取视频信息...");
        let video = VideoBasicInfo::new_from_bvid(&user, bvid).await?;
        info!(
            "视频信息获取成功: {} ({} - {})",
            video.title, video.owner.name, video.bvid
        );
        let safe_bvid = crate::utils::sanitize_filename(&self.bvid);
        let output_dir = output_dir.join(&safe_bvid);
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
                .download_best_quality_audios_to_file(&user, &output_dir, &self)
                .await?;
            info!("下载完成!");
            println!("下载完成!");
        }
        Ok(())
    }
}
