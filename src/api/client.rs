//! API client for making requests to Bilibili API

use reqwest::{Client, RequestBuilder};

pub struct ApiClient {
    pub client: Client,
    pub cookies: Vec<String>,
}

impl ApiClient {
    pub fn new(cookies: Vec<String>) -> Self {
        Self {
            client: Client::new(),
            cookies,
        }
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.client
            .get(url)
            .header("Cookie", self.cookies.join("; "))
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
            )
            .header("Referer", "https://www.bilibili.com/")
    }
}
