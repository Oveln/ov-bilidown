use qrcode::{QrCode, render::unicode};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;
use std::{io, path::PathBuf, time::Duration};
use tokio::{
    fs::{create_dir_all, read_to_string, write},
    sync::OnceCell,
    time::sleep,
};

use crate::{
    error::{BilidownError, Result},
    wbi::get_wbi_keys,
};
use log::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
struct GenData {
    url: String,
    qrcode_key: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GenResp {
    code: i32,
    message: String,
    ttl: i32,
    data: GenData,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PollData {
    url: Option<String>,
    refresh_token: Option<String>,
    timestamp: Option<i64>,
    code: Option<i32>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PollResp {
    code: i32,
    message: String,
    ttl: i32,
    data: PollData,
}

pub struct User {
    cookies: Vec<String>,
    client: Client,
    wbi_keys: OnceCell<(String, String)>,
}

impl User {
    pub async fn new() -> Result<Self> {
        let mut user = Self {
            cookies: Vec::new(),
            client: Client::new(),
            wbi_keys: OnceCell::new(),
        };
        user.login().await?;
        Ok(user)
    }

    pub async fn new_from_file(file_path: &PathBuf) -> Result<Self> {
        let contents = read_to_string(file_path).await.map_err(|e| {
            BilidownError::LoginError(format!(
                "无法读取文件 {}: {}",
                file_path.to_string_lossy(),
                e
            ))
        })?;
        let cookies: Vec<String> = contents
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        if cookies.is_empty() {
            return Err(BilidownError::LoginError(format!(
                "文件 {} 中没有有效的 cookie",
                file_path.to_string_lossy()
            )));
        }
        let ret = Self {
            cookies,
            client: Client::new(),
            wbi_keys: OnceCell::new(),
        };
        if ret.verify_login().await? {
            Ok(ret)
        } else {
            Err(BilidownError::LoginError(
                "Cookie 无效或登录已过期".to_string(),
            ))
        }
    }

    pub fn save_to_file(&self, file_name: &PathBuf) -> io::Result<()> {
        let contents = self.cookies.join("\n");
        // 保证路径
        if let Some(parent) = std::path::Path::new(file_name).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_name, contents)?;
        Ok(())
    }

    async fn verify_login(&self) -> Result<bool> {
        let url = "https://api.bilibili.com/x/web-interface/nav";
        let req = self.get(url);
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let json: serde_json::Value = resp.json().await?;
        debug!("验证登录返回数据: {}", json);
        if json["code"] == 0 && json["data"]["isLogin"] == true {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn gen_qr(&self) -> Result<GenResp> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
        let res: GenResp = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }

    async fn poll_qr(&self, key: &str) -> Result<(PollResp, Vec<String>)> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
        let req = self.client.get(url).query(&[("qrcode_key", key)]);
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

    pub async fn login(&mut self) -> Result<()> {
        info!("申请二维码...（生成 qrcode_key 与 url）");
        let gen_resp = self.gen_qr().await.map_err(|e| {
            error!("申请二维码失败: {}", e);
            e
        })?;

        if gen_resp.code != 0 {
            error!(
                "接口返回错误 code={} message={}",
                gen_resp.code, gen_resp.message
            );
            return Err(BilidownError::LoginError(gen_resp.message));
        }

        info!("请用手机扫码打开下面的 URL（或将 url 生成二维码）:");
        info!("url: {}", gen_resp.data.url);
        let code = QrCode::new(gen_resp.data.url).unwrap();
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        println!("{}", image); // 二维码图像仍使用print输出
        info!("qrcode_key: {}", gen_resp.data.qrcode_key);

        info!("开始轮询登录状态（最多 180s）...");
        let mut elapsed = 0u32;

        loop {
            if elapsed > 180 {
                warn!("二维码超时，请重试");
                return Err(BilidownError::LoginError("二维码超时".to_string()));
            }

            match self.poll_qr(&gen_resp.data.qrcode_key).await {
                Ok((pr, cookies)) => {
                    if pr.code != 0 {
                        error!("轮询接口返回非 0 code={} message={}", pr.code, pr.message);
                    }

                    let code = pr.data.code.unwrap_or(86101);
                    match code {
                        86101 => {
                            // 未扫码
                            info!("等待扫码...");
                        }
                        86090 => {
                            info!("已扫码，等待确认...");
                        }
                        86038 => {
                            warn!("二维码已失效或超时");
                            return Err(BilidownError::LoginError(
                                "二维码已失效或超时".to_string(),
                            ));
                        }
                        0 => {
                            info!("登录成功！");
                            if !cookies.is_empty() {
                                debug!("set-cookie headers:");
                                for c in cookies.iter() {
                                    debug!("  {}", c);
                                }
                                self.cookies = cookies;
                            }
                            break;
                        }
                        other => {
                            info!(
                                "轮询返回 code={} message={}",
                                other,
                                pr.data.message.unwrap_or_default()
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("轮询失败: {}", e);
                    return Err(e);
                }
            }
            sleep(Duration::from_secs(2)).await;
            elapsed += 2;
        }

        info!("登录流程结束");
        Ok(())
    }
    pub async fn get_wbi_keys(&self) -> (&str, &str) {
        let wbi_keys = self
            .wbi_keys
            .get_or_init(|| async {
                get_wbi_keys().await.unwrap_or_else(|e| {
                    error!("获取WBI密钥失败: {}", e);
                    panic!("无法获取WBI密钥，程序退出");
                })
            })
            .await;
        (&wbi_keys.0, &wbi_keys.1)
    }
    pub fn get_client(&self) -> &Client {
        &self.client
    }
    pub fn get(&self, url: &str) -> RequestBuilder {
        self.client
        .get(url)
        .header("Cookie", self.cookies.join("; "))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36")
        .header("Referer", "https://www.bilibili.com/")
    }
    pub async fn download_to_file(&self, url: &str, path: &PathBuf, file_name: &str) -> Result<()> {
        let req = self.get(url);
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(BilidownError::ApiError(format!(
                "下载失败，HTTP状态码: {}",
                resp.status()
            )));
        }
        let bytes = resp.bytes().await?;
        create_dir_all(path).await?;
        let file_path = path.join(file_name);
        write(file_path, bytes).await?;
        Ok(())
    }
}
