use qrcode::{QrCode, render::unicode};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;
use std::{error::Error as StdError, fmt, io, time::Duration};
use tokio::{
    fs::{create_dir_all, write},
    sync::OnceCell,
    time::sleep,
};

use crate::wbi::get_wbi_keys;

#[derive(Debug)]
pub enum LoginError {
    ApiError(String),
    RequestError(reqwest::Error),
    Timeout(String),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginError::ApiError(msg) => write!(f, "API错误: {}", msg),
            LoginError::RequestError(e) => write!(f, "请求错误: {}", e),
            LoginError::Timeout(msg) => write!(f, "超时: {}", msg),
        }
    }
}

impl StdError for LoginError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            LoginError::RequestError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for LoginError {
    fn from(err: reqwest::Error) -> Self {
        LoginError::RequestError(err)
    }
}

#[derive(Debug, Deserialize)]
struct GenData {
    url: String,
    qrcode_key: String,
}

#[derive(Debug, Deserialize)]
struct GenResp {
    code: i32,
    message: String,
    ttl: i32,
    data: GenData,
}

#[derive(Debug, Deserialize)]
struct PollData {
    url: Option<String>,
    refresh_token: Option<String>,
    timestamp: Option<i64>,
    code: Option<i32>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
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
    pub async fn new() -> Result<Self, LoginError> {
        let mut user = Self {
            cookies: Vec::new(),
            client: Client::new(),
            wbi_keys: OnceCell::new(),
        };
        user.login().await?;
        Ok(user)
    }

    pub fn new_from_file(file_name: &str) -> Result<Self, LoginError> {
        let contents = std::fs::read_to_string(file_name)
            .map_err(|e| LoginError::ApiError(format!("无法读取文件 {}: {}", file_name, e)))?;
        let cookies: Vec<String> = contents
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        if cookies.is_empty() {
            return Err(LoginError::ApiError(format!(
                "文件 {} 中没有有效的 cookie",
                file_name
            )));
        }
        Ok(Self {
            cookies,
            client: Client::new(),
            wbi_keys: OnceCell::new(),
        })
    }

    pub fn save_to_file(&self, file_name: &str) -> io::Result<()> {
        let contents = self.cookies.join("\n");
        // 保证路径
        if let Some(parent) = std::path::Path::new(file_name).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_name, contents)?;
        Ok(())
    }

    fn is_logged_in(&self) -> bool {
        !self.cookies.is_empty()
    }

    async fn gen_qr(&self) -> Result<GenResp, LoginError> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
        let res: GenResp = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }

    async fn poll_qr(&self, key: &str) -> Result<(PollResp, Vec<String>), LoginError> {
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

    pub async fn login(&mut self) -> Result<(), LoginError> {
        println!("申请二维码...（生成 qrcode_key 与 url）");
        let gen_resp = match self.gen_qr().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("申请二维码失败: {}", e);
                return Err(e);
            }
        };

        if gen_resp.code != 0 {
            eprintln!(
                "接口返回错误 code={} message={}",
                gen_resp.code, gen_resp.message
            );
            return Err(LoginError::ApiError(gen_resp.message));
        }

        println!("请用手机扫码打开下面的 URL（或将 url 生成二维码）:");
        println!("url: {}", gen_resp.data.url);
        let code = QrCode::new(gen_resp.data.url).unwrap();
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        println!("{}", image);
        println!("qrcode_key: {}", gen_resp.data.qrcode_key);

        println!("开始轮询登录状态（最多 180s）...");
        let mut elapsed = 0u32;

        loop {
            if elapsed > 180 {
                println!("二维码超时，请重试");
                return Err(LoginError::Timeout("二维码超时".to_string()));
            }

            match self.poll_qr(&gen_resp.data.qrcode_key).await {
                Ok((pr, cookies)) => {
                    if pr.code != 0 {
                        eprintln!("轮询接口返回非 0 code={} message={}", pr.code, pr.message);
                    }

                    let code = pr.data.code.unwrap_or(86101);
                    match code {
                        86101 => {
                            // 未扫码
                            println!("等待扫码...");
                        }
                        86090 => {
                            println!("已扫码，等待确认...");
                        }
                        86038 => {
                            println!("二维码已失效或超时");
                            return Err(LoginError::Timeout("二维码已失效或超时".to_string()));
                        }
                        0 => {
                            println!("登录成功！");
                            if !cookies.is_empty() {
                                println!("set-cookie headers:");
                                for c in cookies.iter() {
                                    println!("  {}", c);
                                }
                                self.cookies = cookies;
                            }
                            break;
                        }
                        other => {
                            println!(
                                "轮询返回 code={} message={}",
                                other,
                                pr.data.message.unwrap_or_default()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("轮询失败: {}", e);
                    return Err(e);
                }
            }
            sleep(Duration::from_secs(2)).await;
            elapsed += 2;
        }

        println!("结束");
        Ok(())
    }
    pub async fn get_wbi_keys(&self) -> (&str, &str) {
        let wbi_keys = self
            .wbi_keys
            .get_or_init(|| async { get_wbi_keys().await.unwrap() })
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
    pub async fn download_to_file(
        &self,
        url: &str,
        path: &str,
        file_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let req = self.get(url);
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(format!("下载失败，HTTP状态码: {}", resp.status()).into());
        }
        let bytes = resp.bytes().await?;
        create_dir_all(path).await?;
        let file_path = format!("{}/{}", path, file_name);
        write(file_path, bytes).await?;
        Ok(())
    }
}
