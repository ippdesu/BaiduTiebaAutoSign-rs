# 百度贴吧自动签到 Rust 版

## 自动签到状态

[![Baidu Tieba Auto Sign](https://github.com/ippdesu/BaiduTiebaAutoSign-rs/actions/workflows/main.yml/badge.svg)](https://github.com/ippdesu/BaiduTiebaAutoSign-rs/actions/workflows/main.yml)

## 使用说明

### GitHub Actions 部署

1. Fork 本仓库，然后在仓库的右上角的 Settings 找到 Secrets 一项，新建一个密钥变量名为 `BDUSS` 的值。BDUSS 值支持同时添加多个账号的 BDUSS 值之间用 `#` 进行连接。

2. 启用自己 Fork 的仓库的 Actions 选项，第一次创建需要点 `I understand...` 按钮来确认 Fork 的仓库已开启 GitHub Actions。

3. 手动触发发送一次 commit。

### 本地运行

1. 安装 [Rust](https://www.rust-lang.org/tools/install)

2. 配置环境变量：

   **Windows PowerShell:**
   ```powershell
   $env:BDUSS="your_bduss_value"
   # 多个账号
   $env:BDUSS="bduss1#bduss2#bduss3"
   ```

   **Linux / macOS:**
   ```bash
   export BDUSS="your_bduss_value"
   # 多个账号
   export BDUSS="bduss1#bduss2#bduss3"
   ```

3. 运行程序：
   ```bash
   cargo run --release
   ```

## 功能特点

- 支持多账号同时签到（使用 `#` 分隔）
- 异步高效执行（Tokio 运行时）
- 自动避免限流（智能延迟机制）
- 移动端模拟（避免风控）
- 请求失败自动重试（3 次指数退避）

## 项目结构

```
.
├── src/
│   └── main.rs          # 主程序入口
├── .github/
│   └── workflows/
│       └── main.yml     # GitHub Actions 配置
├── Cargo.toml           # 项目依赖配置
└── README.md
```

## 依赖项

- `reqwest` — HTTP 客户端
- `tokio` — 异步运行时
- `serde` / `serde_json` — JSON 序列化
- `md5` — 签名哈希
- `rand` — 随机延迟
