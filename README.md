# ov-bilidown

一个基于 Rust 的命令行应用程序，用于从 Bilibili 视频下载音频流。此工具使用户能够从 Bilibili 视频中提取高质量音频并将它们保存为音频文件。

## 功能特性

- **二维码登录**: 使用扫描二维码安全地进行 Bilibili 账户认证
- **音频提取**: 从 Bilibili 视频下载最高质量的音频流
- **WBI 签名保护**: 实现 Bilibili 的 Web Business Interface 签名来处理 API 请求
- **Cookie 管理**: 保存和加载认证 Cookie 以保持会话
- **多种音质支持**: 支持包括标准音质、杜比全景声和 Hi-Res 在内的多种音质
- **命令行界面**: 支持通过命令行参数指定视频 ID、下载目录和音质
- **统一错误处理**: 使用自定义错误类型提供更好的错误反馈
- **用户目录管理**: 智能识别用户配置和下载目录
- **订阅功能**: 支持配置文件订阅多个视频进行批量下载
- **通配符替换**: 支持多种元数据通配符来自定义音频文件的标签和命名

## 依赖

- ffmpeg（用于音频格式转换）

## 安装

构建和安装项目：

```bash
# 克隆仓库（如需要）
git clone https://github.com/oveln/ov-bilidown.git
cd ov-bilidown

# 以发布模式构建
cargo build --release

# 可执行文件将在以下位置可用
./target/release/ov-bilidown
```

或者可以直接运行而不构建：

```bash
cargo run -- [参数]
```

## 使用方法

### 基本使用

```bash
# 从视频下载音频
cargo run -- -b BV1NfxMedEU6

# 指定下载目录
cargo run -- -b BV1NfxMedEU6 -o /path/to/downloads

# 获取视频信息但不下载
cargo run -- -b BV1NfxMedEU6 --info-only

# 增加日志详细程度
cargo run -- -b BV1NfxMedEU6 -v
```

### 订阅功能

ov-bilidown 支持通过 TOML 配置文件批量下载多个视频。创建一个订阅配置文件（默认为 `~/.config/ov-bilidown/sub.toml`），示例如下：

```toml
[[sub]]
title = "{title}"
bvid = "BV1H242zQEyb"
artist = "洛天依; {artist}"
album = "{title}"

[[sub]]
title = "{part_title}"
bvid = "BVXXXXXXXXX"
artist = "{artist}"
album = "{title} - 精选集"
```

要使用订阅功能，只需运行应用而不指定 `-b` 参数：

```bash
cargo run --
```

这将处理配置文件中的所有订阅项目。

### 通配符说明

ov-bilidown 支持多种元数据通配符，可自定义音频文件的标签和命名。这些通配符可在订阅配置文件的 `title`、`artist` 和 `album` 字段中使用：

- `{title}`: 视频标题
- `{part_title}`: 分P标题
- `{artist}` 或 `{uploader}`: UP主名称
- `{album}`: 视频标题（作为专辑）
- `{bv_id}`: BV号
- `{aid}`: AID
- `{duration}`: 分P时长（秒）
- `{page}`: 分P编号
- `{date}`: 当前日期（格式：YYYY-MM-DD）

例如，配置中的 `"artist": "洛天依; {artist}"` 将被替换为实际的 UP 主名称，如 "洛天依; 某UP主"。

### 命令行选项

```
USAGE:
    ov-bilidown [OPTIONS]

OPTIONS:
    -b, --bvid <BVID>                      Bilibili 视频 ID (例如, BV1NfxMedEU6)
    -o, --output-dir <OUTPUT_DIR>          下载输出目录 [默认: ~/Downloads]
    -c, --cookie-file <COOKIE_FILE>        Cookie 文件路径
    -s, --subscription-file <SUBSCRIPTION_FILE>   订阅配置文件路径
        --info-only                        显示视频信息但不下载
    -v, --verbose                          增加日志详细程度 (-v, -vv, -vvv)
    -q, --quiet                            安静模式，只显示错误
        --help                             显示帮助信息
        --version                          显示版本信息
```

### 示例

```bash
# 下载单个视频
cargo run -- -b BV1234567890

# 下载到特定文件夹
cargo run -- -b BV1234567890 -o ./music

# 使用自定义订阅文件
cargo run -- -s /path/to/my/sub.toml

# 查看视频信息但不下载
cargo run -- -b BV1234567890 --info-only

# 处理所有订阅项
cargo run --
```

## 配置

- 认证 Cookie 保存到 `~/.config/ov-bilidown/cookies.txt`
- 订阅配置文件默认为 `~/.config/ov-bilidown/sub.toml`
- 下载的音频文件默认保存到用户下载目录
- 通过命令行参数可自定义视频 ID、下载目录

## 许可证

本项目根据 [MIT] 许可 - 详见 [LICENSE](LICENSE) 文件了解详情。