//! Data models for the application

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoBasicInfo {
    // 基本信息
    pub bvid: String,  // 稿件bvid
    pub aid: i64,      // 稿件avid
    pub tid: i32,      // 分区id
    pub title: String, // 视频标题
    pub desc: String,  // 视频简介
    pub duration: i32, // 稿件总时长(所有分P)

    // 封面和分区信息
    pub pic: String,   // 封面图片url
    pub tname: String, // 分区名称

    // 视频作者
    pub owner: Owner,

    // 视频状态数据
    pub stat: VideoStat,

    // 视频权限信息
    pub rights: VideoRights,

    // 分P信息
    pub pages: Option<Vec<VideoPart>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub mid: i64,     // UP主mid
    pub name: String, // UP主名称
    pub face: String, // UP主头像
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoStat {
    pub view: i32,     // 播放数
    pub danmaku: i32,  // 弹幕数
    pub reply: i32,    // 评论数
    pub favorite: i32, // 收藏数
    pub coin: i32,     // 投币数
    pub share: i32,    // 分享数
    pub like: i32,     // 获赞数
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoRights {
    pub bp: i32,         // 是否允许BP
    pub elec: i32,       // 是否支持充电
    pub download: i32,   // 是否允许下载
    pub movie: i32,      // 是否为电影
    pub pay: i32,        // 是否需要付费
    pub hd5: i32,        // 是否有高码率
    pub no_reprint: i32, // 是否禁止转载
    pub autoplay: i32,   // 是否自动播放
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoPart {
    pub cid: i64,      // 分P cid
    pub page: i32,     // 分P序号
    pub from: String,  // 视频来源
    pub part: String,  // 分P标题
    pub duration: i32, // 分P时长(秒)
    pub dimension: VideoDimension,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoDimension {
    pub width: i32,  // 视频宽度
    pub height: i32, // 视频高度
    pub rotate: i32, // 是否旋转 0:正常 1:宽高对换
}

// 用于视频API响应的结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub ttl: i32,
    pub data: Option<T>,
}
