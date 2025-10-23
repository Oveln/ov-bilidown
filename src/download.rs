use crate::{api::endpoints, error::Result, user::User};
use log::{debug, trace};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioQuality {
    Q64K = 1,
    Q132K = 2,
    Q192K = 3,
    DolbyAtmos = 4,
    HiRes = 5,
}

impl AudioQuality {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            30216 => Some(Self::Q64K),
            30232 => Some(Self::Q132K),
            30280 => Some(Self::Q192K),
            30250 => Some(Self::DolbyAtmos),
            30251 => Some(Self::HiRes),
            _ => None,
        }
    }

    pub fn quality_name(&self) -> &'static str {
        match self {
            Self::Q64K => "64K",
            Self::Q132K => "132K",
            Self::Q192K => "192K",
            Self::DolbyAtmos => "杜比全景声",
            Self::HiRes => "Hi-Res无损",
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DashAudioStream {
    pub id: u32, // 音质代码，如 30216/30232/30280 等
    pub base_url: String,
    pub backup_url: Option<Vec<String>>,
    pub bandwidth: u64,
    pub mime_type: String,
    pub codecs: String,
    pub segment_base: Option<DashSegmentBase>,
    pub codecid: u32,
}

impl DashAudioStream {
    /// 获取音频流的质量等级
    pub fn get_quality(&self) -> Option<AudioQuality> {
        AudioQuality::from_id(self.id)
    }

    /// 获取音频质量的描述
    pub fn get_quality_description(&self) -> String {
        match self.get_quality() {
            Some(quality) => format!("{} ({}kbps)", quality.quality_name(), self.bandwidth / 1024),
            None => format!("未知质量 ({}kbps)", self.bandwidth / 1024),
        }
    }

    /// 从音频流列表中获取指定质量等级的音频流
    #[allow(dead_code)]
    pub fn get_by_quality(
        streams: &[DashAudioStream],
        quality: AudioQuality,
    ) -> Option<&DashAudioStream> {
        streams
            .iter()
            .find(|stream| stream.get_quality() == Some(quality))
    }

    /// 从音频流列表中获取最高质量的音频流（基于id值判断）
    pub fn get_highest_quality(streams: &[DashAudioStream]) -> Option<&DashAudioStream> {
        streams
            .iter()
            .max_by_key(|stream| stream.get_quality().map(|q| q as u32).unwrap_or(0))
    }

    /// 从音频流列表中获取最高码率的音频流
    #[allow(dead_code)]
    pub fn get_highest_bandwidth(streams: &[DashAudioStream]) -> Option<&DashAudioStream> {
        streams.iter().max_by_key(|stream| stream.bandwidth)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DashSegmentBase {
    pub initialization: String,
    pub index_range: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DolbyInfo {
    pub r#type: u8, // 1: 普通杜比音效, 2: 全景杜比音效
    pub audio: Option<Vec<DashAudioStream>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FlacInfo {
    pub display: bool, // 是否在播放器显示切换Hi-Res无损音轨按钮
    pub audio: DashAudioStream,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DashInfo {
    pub duration: f64,
    pub min_buffer_time: Option<f64>,
    pub audio: Option<Vec<DashAudioStream>>,
    pub dolby: Option<DolbyInfo>, // 杜比视界视频独有
    pub flac: Option<FlacInfo>,   // Hi-Res无损音轨
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PlayUrlDashResp {
    pub code: i32,
    pub message: String,
    pub ttl: i32,
    pub data: PlayUrlDashData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayUrlDashData {
    pub dash: DashInfo,
}

impl crate::models::VideoPart {
    pub async fn get_dash_audio_stream(
        &self,
        bvid: &str,
        user: &User,
    ) -> Result<Vec<DashAudioStream>> {
        let dash_resp = endpoints::get_play_url_dash(user, bvid, self.cid).await?;

        let dash = dash_resp.data.dash;
        trace!("{:#?}", dash);
        let mut audio_streams = dash.audio.unwrap_or_default();
        if let Some(dolby) = dash.dolby {
            if let Some(dolby_stream) = dolby.audio {
                if !dolby_stream.is_empty() {
                    audio_streams.extend(dolby_stream);
                }
            }
        }
        if let Some(flac) = dash.flac {
            audio_streams.push(flac.audio.clone());
        }
        debug!("获取到 {} 个音频流", audio_streams.len());
        Ok(audio_streams)
    }
}
