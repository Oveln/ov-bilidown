use thiserror::Error;

#[derive(Error, Debug)]
pub enum BilidownError {
    #[error("请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("序列化错误: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("API错误: {0}")]
    ApiError(String),

    #[error("登录错误: {0}")]
    LoginError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("配置错误: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("转换错误: {0}")]
    ConversionError(String),
}

impl From<&str> for BilidownError {
    fn from(s: &str) -> Self {
        BilidownError::ApiError(s.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BilidownError>;