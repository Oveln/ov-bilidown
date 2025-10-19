//! API endpoint definitions

use serde::Deserialize;

use crate::{error::Result, models::{ApiResponse, VideoBasicInfo}, user::User, wbi::WbiSendExt};

#[derive(Debug, Deserialize)]
pub struct GenData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GenResp {
    pub code: i32,
    pub message: String,
    pub ttl: i32,
    pub data: GenData,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PollData {
    pub url: Option<String>,
    pub refresh_token: Option<String>,
    pub timestamp: Option<i64>,
    pub code: Option<i32>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PollResp {
    pub code: i32,
    pub message: String,
    pub ttl: i32,
    pub data: PollData,
}

pub async fn get_video_info(user: &User, bvid: &str) -> Result<VideoBasicInfo> {
    let url = "https://api.bilibili.com/x/web-interface/view";
    let params = [("bvid", bvid.to_string())];
    let req = user.get(url).query(&params);
    let wbi_keys = user.get_wbi_keys().await;
    let resp = req
        .wbi_send(user.get_client(), wbi_keys.0, wbi_keys.1)
        .await?;
    let api_resp: ApiResponse<VideoBasicInfo> = resp.json().await?;
    if api_resp.code != 0 {
        return Err(crate::error::BilidownError::ApiError(format!(
            "API错误: {}",
            api_resp.message
        )));
    }
    api_resp
        .data
        .ok_or_else(|| crate::error::BilidownError::ApiError("API返回数据为空".to_string()))
}

pub async fn get_play_url_dash(
    user: &User,
    bvid: &str,
    cid: i64,
) -> Result<crate::download::PlayUrlDashResp> {
    let url = "https://api.bilibili.com/x/player/playurl";
    let params = vec![
        ("bvid", bvid.to_string()),
        ("cid", cid.to_string()),
        ("fnval", "4048".to_string()), // 获取所有 DASH 流
        ("fnver", "0".to_string()),
        ("otype", "json".to_string()),
    ];
    let wbi_keys = user.get_wbi_keys().await;
    let resp = user
        .get(url)
        .query(&params)
        .wbi_send(user.get_client(), wbi_keys.0, wbi_keys.1)
        .await?;
    let dash_resp: crate::download::PlayUrlDashResp = resp.json().await?;
    if dash_resp.code != 0 {
        return Err(crate::error::BilidownError::ApiError(format!(
            "API错误: {}",
            dash_resp.message
        )));
    }
    Ok(dash_resp)
}

pub async fn generate_qr_login(user: &User) -> Result<GenResp> {
    let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
    let res: GenResp = user.get(url).send().await?.json().await?;
    Ok(res)
}

pub async fn poll_qr_login(user: &User, key: &str) -> Result<(PollResp, Vec<String>)> {
    let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
    let req = user.get(url).query(&[("qrcode_key", key)]);
    let resp = req.send().await?;
    // capture set-cookie headers before consuming the body
    let cookies: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
        .collect();
    let pr: PollResp = resp.json().await?;
    Ok((pr, cookies))
}

pub async fn verify_login(user: &User) -> Result<bool> {
    let url = "https://api.bilibili.com/x/web-interface/nav";
    let req = user.get(url);
    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(false);
    }
    let json: serde_json::Value = resp.json().await?;
    if json["code"] == 0 && json["data"]["isLogin"] == true {
        Ok(true)
    } else {
        Ok(false)
    }
}