use std::path::{Path, PathBuf};

use chrono::Datelike;
use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::tag::{Tag, TagType};

use crate::{download::DashAudioStream, error::{BilidownError, Result}, models::{VideoBasicInfo, VideoPart}, utils};
use log::{debug, info};

pub async fn convert_audio_with_metadata(
    input_path: &Path,
    output_dir: &Path,
    video_info: &VideoBasicInfo,
    video_part: &VideoPart,
    audio_stream: &DashAudioStream,
) -> Result<PathBuf> {
    info!("开始转换音频并添加元数据: {:?}", input_path);

    // 确定输出格式
    let output_format = determine_output_format(audio_stream);
    let output_filename =
        generate_output_filename(&video_info.title, video_part, output_format.clone());
    let output_path = output_dir.join(output_filename);

    // 转换音频格式
    match output_format {
        AudioFormat::Mp3 => convert_to_mp3(input_path, &output_path).await?,
        AudioFormat::Flac => convert_to_flac(input_path, &output_path).await?,
    }

    // 添加元数据
    add_metadata_to_file(&output_path, video_info, video_part)?;

    info!("音频转换完成: {:?}", output_path);
    Ok(output_path)
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Flac,
}

fn determine_output_format(audio_stream: &DashAudioStream) -> AudioFormat {
    if audio_stream.mime_type.contains("aac")
        || audio_stream.mime_type.contains("mp4")
        || audio_stream.mime_type.contains("m4a")
        || audio_stream.mime_type.contains("mp3")
    {
        AudioFormat::Mp3
    } else {
        AudioFormat::Flac
    }
}

fn generate_output_filename(title: &str, video_part: &VideoPart, format: AudioFormat) -> String {
    let clean_title = utils::sanitize_filename(title);
    let extension = match format {
        AudioFormat::Mp3 => "mp3",
        AudioFormat::Flac => "flac",
    };

    format!("{}-P{}.{}", clean_title, video_part.page, extension)
}

async fn convert_to_mp3(input_path: &Path, output_path: &Path) -> Result<()> {
    info!("正在转换为MP3格式: {:?} -> {:?}", input_path, output_path);

    let input_path_str = input_path
        .to_str()
        .ok_or_else(|| BilidownError::ConversionError("输入路径无效".to_string()))?;

    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| BilidownError::ConversionError("输出路径无效".to_string()))?;

    // 使用ffmpeg命令行工具进行转换
    utils::run_ffmpeg_command(&[
        "-i",
        input_path_str,
        "-codec:a",
        "libmp3lame",
        "-q:a",
        "2",  // 高质量
        "-y", // 覆盖输出文件
        output_path_str,
    ]).await
}

async fn convert_to_flac(input_path: &Path, output_path: &Path) -> Result<()> {
    info!("正在转换为FLAC格式: {:?} -> {:?}", input_path, output_path);

    let input_path_str = input_path
        .to_str()
        .ok_or_else(|| BilidownError::ConversionError("输入路径无效".to_string()))?;

    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| BilidownError::ConversionError("输出路径无效".to_string()))?;

    // 使用ffmpeg命令行工具进行转换
    utils::run_ffmpeg_command(&[
        "-i",
        input_path_str,
        "-codec:a",
        "flac",
        "-compression_level",
        "5", // 中等压缩级别
        "-y",
        output_path_str,
    ]).await
}

fn add_metadata_to_file(
    file_path: &Path,
    video_info: &VideoBasicInfo,
    video_part: &VideoPart,
) -> Result<()> {
    info!("正在添加元数据到文件: {:?}", file_path);

    // 确保文件存在并可以访问
    if !file_path.exists() {
        return Err(BilidownError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("文件不存在: {:?}", file_path),
        )));
    }

    // 读取现有的音频文件
    let mut tagged_file = lofty::read_from_path(file_path)
        .map_err(|e| BilidownError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // 获取或创建标签
    let tag = if let Some(existing_tag) = tagged_file.tag_mut(TagType::Id3v2) {
        existing_tag
    } else {
        let tag_type = match file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "mp3" => TagType::Id3v2,
            "flac" => TagType::VorbisComments,
            _ => TagType::Id3v2, // 默认使用ID3v2
        };
        let new_tag = Tag::new(tag_type);
        tagged_file.insert_tag(new_tag);
        tagged_file.tag_mut(tag_type).unwrap()
    };

    // 设置元数据 - 按照音乐播放器的标准映射
    // 分P标题作为歌曲名，视频标题作为专辑名，UP主作为艺术家
    tag.set_title(video_part.part.clone()); // 歌名 = 分P标题
    tag.set_artist(video_info.owner.name.clone()); // 艺术家 = UP主
    tag.set_album(video_info.title.clone()); // 专辑名 = 视频标题
    tag.set_genre("Bilibili".to_string());
    tag.set_year(chrono::Local::now().year() as u32);
    tag.set_track(video_part.page as u32);

    // 添加注释信息
    if !video_info.desc.is_empty() {
        tag.set_comment(video_info.desc.clone());
    }

    info!("元数据准备完成，正在写入文件...");

    tagged_file
        .save_to_path(file_path, WriteOptions::default())
        .map_err(|e| BilidownError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    debug!("元数据添加完成");
    Ok(())
}

/// 验证转换后的音频文件
pub fn validate_converted_file(file_path: &Path) -> Result<()> {
    // 检查文件是否存在
    utils::validate_file_exists(file_path)?;
    
    // 检查文件大小
    utils::validate_file_not_empty(file_path)?;

    debug!("验证音频文件: {:?} ({} bytes)", file_path, std::fs::metadata(file_path)?.len());
    Ok(())
}
