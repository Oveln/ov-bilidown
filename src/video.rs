use std::path::PathBuf;

use crate::{
    api::endpoints,
    converter::{convert_audio_with_metadata, validate_converted_file},
    download::DashAudioStream,
    error::{BilidownError, Result},
    models::{Subscription, VideoBasicInfo},
    user::User,
};
use futures::future;
use log::{debug, error, info, warn};

impl VideoBasicInfo {
    pub async fn new_from_bvid(user: &User, bvid: &str) -> Result<Self> {
        endpoints::get_video_info(user, bvid).await
    }
    
    pub async fn new_from_subscription(user: &User, subscription: &Subscription) -> Result<Self> {
        Self::new_from_bvid(user, &subscription.bvid).await
    }
    
    pub async fn download_best_quality_audios_to_file(
        &self,
        user: &User,
        dir: &PathBuf,
    ) -> Result<()> {
        if let Some(pages) = &self.pages {
            info!("开始下载视频 {} 的 {} 个分P", self.bvid, pages.len());
            let tasks = pages.iter().map(|video_part| {
                let user = user;
                let dir = dir;
                async move {
                    info!("处理分P {} - {}", video_part.page, video_part.part);
                    match video_part.get_dash_audio_stream(&self.bvid, user).await {
                        Ok(audio_streams) => {
                            if let Some(best_audio) =
                                DashAudioStream::get_highest_quality(audio_streams.as_slice())
                            {
                                // 首先下载原始音频文件
                                let temp_dir = match tempfile::TempDir::new() {
                                    Ok(dir) => dir,
                                    Err(e) => {
                                        error!("创建临时目录失败: {}", e);
                                        return;
                                    }
                                };

                                let temp_file_name = format!(
                                    "{}-{}_{}.m4a",
                                    self.title.replace("/", "_"),
                                    video_part.page,
                                    best_audio.get_quality_description()
                                );
                                let temp_file_path = temp_dir.path().join(&temp_file_name);

                                info!("正在下载原始音频文件: {}", temp_file_name);
                                if let Err(e) = user
                                    .download_to_file(
                                        &best_audio.base_url,
                                        &temp_dir.path().to_path_buf(),
                                        &temp_file_name,
                                    )
                                    .await
                                {
                                    error!("下载原始音频文件失败: {}", e);
                                    return;
                                }

                                debug!("原始音频下载完成，开始转换和添加元数据");

                                // 转换格式并添加元数据
                                match convert_audio_with_metadata(
                                    &temp_file_path,
                                    dir,
                                    self,
                                    video_part,
                                    best_audio,
                                )
                                .await
                                {
                                    Ok(output_path) => {
                                        if let Err(e) = validate_converted_file(&output_path) {
                                            warn!("转换后的文件验证失败: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("音频转换失败: {}", e);
                                    }
                                }

                                // 清理临时文件
                                drop(temp_dir);
                            } else {
                                warn!("分P {} 未找到可用的音频流", video_part.page);
                            }
                        }
                        Err(e) => {
                            warn!("获取分P {} 的音频流失败: {}", video_part.page, e);
                        }
                    }
                }
            });
            future::join_all(tasks).await;
            info!("视频 {} 下载完成", self.bvid);
            Ok(())
        } else {
            error!("视频 {} 没有分P信息", self.bvid);
            Err(BilidownError::ApiError("视频没有分P信息".to_string()))
        }
    }
}
