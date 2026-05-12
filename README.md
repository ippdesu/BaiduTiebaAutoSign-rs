# 百度贴吧自动签到Rust版
## 今日签到状态
[![Baidu Tieba Auto Sign](https://github.com/ippdesu/BaiduTiebaAutoSign-rs/actions/workflows/main.yml/badge.svg)](https://github.com/ippdesu/BaiduTiebaAutoSign-rs/actions/workflows/main.yml)
## 使用说明

1. Fork 本仓库，然后点击你的仓库右上角的 Settings，找到 Secrets 这一项，添加一个库秘密变量。其中 `BDUSS` 存放你的 BDUSS。支持同时添加多个帐户，BDUSS 之间用 `#` 隔开即可。
2. 设置好环境变量后点击你的仓库上方的 `Actions` 选项，第一次打开需要点击 `I understand...` 按钮，确认在 Fork 的仓库上启用 GitHub Actions 。
3. 任意发起一次commit。
