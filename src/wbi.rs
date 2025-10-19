use reqwest::{RequestBuilder, Url, header::USER_AGENT};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Deserialize)]
struct WbiImg {
    img_url: String,
    sub_url: String,
}

#[derive(Deserialize)]
struct Data {
    wbi_img: WbiImg,
}

#[derive(Deserialize)]
struct ResWbi {
    data: Data,
}

// 对 imgKey 和 subKey 进行字符顺序打乱编码
fn get_mixin_key(orig: &[u8]) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| orig[i] as char)
        .collect::<String>()
}

fn get_url_encoded(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            true => Some(c.to_string()),
            false => {
                // 过滤 value 中的 "!'()*" 字符
                if "!'()*".contains(c) {
                    return None;
                }
                let encoded = c
                    .encode_utf8(&mut [0; 4])
                    .bytes()
                    .fold("".to_string(), |acc, b| acc + &format!("%{:02X}", b));
                Some(encoded)
            }
        })
        .collect::<String>()
}

// 为请求参数进行 wbi 签名
fn encode_wbi(
    params: Vec<(String, String)>,
    (img_key, sub_key): (String, String),
) -> Vec<(String, String)> {
    let cur_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    _encode_wbi(params, (img_key, sub_key), cur_time)
}

fn _encode_wbi(
    mut params: Vec<(String, String)>,
    (img_key, sub_key): (String, String),
    timestamp: u64,
) -> Vec<(String, String)> {
    let mixin_key = get_mixin_key((img_key + &sub_key).as_bytes());
    // 添加当前时间戳
    params.push(("wts".to_string(), timestamp.to_string()));
    // 重新排序
    params.sort_by(|a, b| a.0.cmp(&b.0));
    // 拼接参数
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", get_url_encoded(k), get_url_encoded(v)))
        .collect::<Vec<_>>()
        .join("&");
    // 计算签名
    let web_sign = format!("{:?}", md5::compute(query + &mixin_key));
    // 返回最终的 query
    params.push(("w_rid".to_string(), web_sign));
    params
}

pub async fn get_wbi_keys() -> Result<(String, String)> {
    let client = reqwest::Client::new();
    let ResWbi { data:Data{wbi_img} } = client
    .get("https://api.bilibili.com/x/web-interface/nav")
    .header(USER_AGENT,"Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
    .header("Referer","https://www.bilibili.com/")
     // SESSDATA=xxxxx
    .header("Cookie", "SESSDATA=xxxxx")
    .send()
    .await?
    .json::<ResWbi>()
    .await?;
    Ok((
        take_filename(wbi_img.img_url).unwrap(),
        take_filename(wbi_img.sub_url).unwrap(),
    ))
}

fn take_filename(url: String) -> Option<String> {
    url.rsplit_once('/')
        .and_then(|(_, s)| s.rsplit_once('.'))
        .map(|(s, _)| s.to_string())
}

pub trait WbiSendExt {
    fn wbi_send(
        self,
        client: &reqwest::Client,
        img_key: &str,
        sub_key: &str,
    ) -> impl std::future::Future<Output = std::result::Result<reqwest::Response, reqwest::Error>> + Send;
}

impl WbiSendExt for RequestBuilder {
    fn wbi_send(
        self,
        client: &reqwest::Client,
        img_key: &str,
        sub_key: &str,
    ) -> impl std::future::Future<Output = std::result::Result<reqwest::Response, reqwest::Error>> + Send {
        async move {
            let mut request = self.build()?;
            let url = request.url_mut();

            let params: Vec<(String, String)> = url.query_pairs().into_owned().collect();
            let signed = encode_wbi(params, (img_key.to_string(), sub_key.to_string()));

            let base = format!(
                "{}://{}{}",
                url.scheme(),
                url.host_str().unwrap_or(""),
                url.path()
            );
            let new_url = Url::parse_with_params(&base, &signed).unwrap();
            *url = new_url;

            // 使用传入的 client 发送！
            client.execute(request).await
        }
    }
}

// 取自文档描述的测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_filename() {
        assert_eq!(
            take_filename(
                "https://i0.hdslb.com/bfs/wbi/7cd084941338484aae1ad9425b84077c.png".to_string()
            ),
            Some("7cd084941338484aae1ad9425b84077c".to_string())
        );
    }

    #[test]
    fn test_get_mixin_key() {
        let concat_key =
            "7cd084941338484aae1ad9425b84077c".to_string() + "4932caff0ff746eab6f01bf08b70ac45";
        assert_eq!(
            get_mixin_key(concat_key.as_bytes()),
            "ea1db124af3c7062474693fa704f4ff8"
        );
    }

    #[test]
    fn test_encode_wbi() {
        let params = vec![
            ("foo".to_string(), "114".to_string()),
            ("bar".to_string(), "514".to_string()),
            ("zab".to_string(), "1919810".to_string()),
        ];
        assert_eq!(
            _encode_wbi(
                params,
                (
                    "7cd084941338484aae1ad9425b84077c".to_string(),
                    "4932caff0ff746eab6f01bf08b70ac45".to_string()
                ),
                1702204169
            ),
            vec![
                ("bar".to_string(), "514".to_string()),
                ("foo".to_string(), "114".to_string()),
                ("wts".to_string(), "1702204169".to_string()),
                ("zab".to_string(), "1919810".to_string()),
                (
                    "w_rid".to_string(),
                    "8f6f2b5b3d485fe1886cec6a0be8c5d4".to_string()
                ),
            ]
        )
    }
}
