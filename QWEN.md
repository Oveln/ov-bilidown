# ov-bilidown 项目文档

## 项目概述

ov-bilidown是一个基于Rust的命令行应用程序，用于从Bilibili视频下载音频流。该应用使用Bilibili的API获取视频信息和音频流，然后下载指定视频的最高质量音频。

### 主要特性

- **二维码登录**：实现Bilibili的二维码认证系统
- **音频提取**：从Bilibili视频下载最高质量的音频流
- **WBI签名保护**：实现Bilibili的Web Business Interface签名以处理API请求
- **Cookie管理**：保存和加载认证Cookie以保持会话
- **多种质量支持**：支持包括标准音质、杜比全景声和Hi-Res在内的多种音质
- **命令行参数**：支持通过命令行参数指定视频ID、下载目录等
- **统一错误处理**：使用自定义错误类型提供更好的错误反馈
- **用户目录管理**：智能识别用户配置和下载目录

### 架构

项目组织成以下几个模块：

- `main.rs`：入口点，解析命令行参数并协调应用流程
- `config.rs`：管理应用配置和命令行参数解析
- `error.rs`：定义统一的错误类型和处理机制
- `user.rs`：处理用户认证、登录和HTTP客户端管理
- `video.rs`：管理视频信息获取和音频流下载
- `download.rs`：定义音质类型和流处理
- `wbi.rs`：实现Bilibili的WBI签名算法

## 构建和运行

### 前置条件

- Rust 2024版
- Cargo

### 构建命令

```bash
# 构建项目
cargo build

# 以发布模式构建
cargo build --release
```

### 运行应用

```bash
# 显示帮助信息
cargo run -- --help

# 下载指定视频的音频
cargo run -- -b BV1NfxMedEU6

# 指定下载目录
cargo run -- -b BV1NfxMedEU6 -o /path/to/downloads

# 仅获取视频信息而不下载
cargo run -- -b BV1NfxMedEU6 --info-only

# 指定音频质量
cargo run -- -b BV1NfxMedEU6 -q highest
```

应用将：
1. 解析命令行参数
2. 尝试从配置文件加载现有Cookie（默认位于用户配置目录）
3. 如果未找到Cookie，则提示二维码登录
4. 从指定的视频ID获取视频信息
5. 下载音频到指定目录（默认为用户下载目录）

### 依赖

在`Cargo.toml`中定义的关键依赖：
- `reqwest`：用于API请求的HTTP客户端
- `serde`/`serde_json`：JSON序列化/反序列化
- `qrcode`：登录二维码生成
- `md5`：WBI签名的MD5哈希
- `tokio`：异步运行时
- `clap`：命令行参数解析
- `anyhow`/`thiserror`：错误处理
- `dirs`：用户目录管理

## 开发规范

### 代码风格

- 遵循Rust惯用法和规范
- 对API调用使用async/await
- 实现适当的错误处理和自定义错误类型
- 使用描述性的变量和函数名

### 模块结构

- 保持每个模块专注于单一职责
- 使用适当的可见性（pub用于公共接口）
- 使用serde实现适当的序列化

### 错误处理

- 使用自定义错误类型`BilidownError`统一处理各种错误
- 使用`Result<T, BilidownError>`进行错误传播
- 提供描述性错误消息

## 测试

使用以下命令运行测试：
```bash
cargo test
```

项目在`wbi.rs`中包含了WBI签名实现的单元测试。

## 配置

- 认证Cookie保存到用户配置目录下的`ov-bilidown/cookies.txt`
- 下载的音频文件默认保存到用户下载目录
- 通过命令行参数可自定义视频ID、下载目录、音频质量等