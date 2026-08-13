# deepNotifier

[English](README.md) | [中文](README.zh-CN.md)

面向开发者的通知守护程序。基于 Rust + Tauri 2 构建。

## 功能概览

| 模块 | 说明 |
|------|------|
| **Webhook** | 接收 GitHub / GitLab / Bitbucket / 自定义 webhook 事件 |
| **Poll** | 定时轮询 HTTP 端点，解析 JSON 通知 |
| **CLI Hook** | 向 CLI AI 辅助工具（Claude Code、Codex、Kimi Code 等）注入 hook，接收任务完成、通知、审批超时提醒 |
| **通知** | 声音、应用内弹窗（Bitbucket 风格）、滚动字幕 |
| **勿扰模式** | 定时 DND 计划，按周重复，静音通知 |
| **番茄钟** | 番茄工作法 + 待办任务，支持远程拉取/推送 |
| **滚动字幕** | 多显示器支持，弹幕式多轨道（最多 3 条，可配置），多通知排队轮播、Critical 抢占，5 种预设主题，自定义颜色/字体/图标 |

## 环境要求

- **Node.js** >= 18
- **Rust** 最新稳定版
- **Windows**: Visual Studio Build Tools（C++ 桌面开发）
- **macOS**: Xcode Command Line Tools
- **Linux**: `libwebkit2gtk`、`libgtk-3-dev`

## 快速开始

```bash
npm install
npm run tauri dev
```

## 使用指南

### Webhook

本地 HTTP 服务器接收平台 webhook 事件，自动转换为通知。

**默认端口**: `3927`

| 平台 | 端点 | 验证方式 |
|------|------|----------|
| GitHub | `POST /webhook/github` | HMAC-SHA256 (`X-Hub-Signature-256`) |
| GitLab | `POST /webhook/gitlab` | Token (`X-Gitlab-Token`) |
| Bitbucket | `POST /webhook/bitbucket` | HMAC-SHA256 (`X-Hub-Signature-256`) |
| 自定义 | `POST /webhook/custom` | 无 |

**自定义 Webhook** 发送任意 JSON，通过点分隔路径提取字段：

```bash
curl -X POST http://localhost:3927/webhook/custom \
  -H "Content-Type: application/json" \
  -d '{"title": "Deploy finished", "body": "Build #42 passed", "severity": "Info"}'
```

可配置 JSON 路径（如 `repository.full_name`）和严重性等级（Info / Warning / Critical）。

### Poll 轮询

定时 GET/POST 指定端点，解析响应中的 JSON 通知。支持自定义请求头、请求体、轮询间隔和超时时间。

### 通知方式

| 方式 | 说明 |
|------|------|
| **声音** | 内置 ping / chime 音效，支持导入自定义音频 |
| **弹窗通知** | 应用内 Bitbucket 风格弹窗卡片，显示在工作区角落（不遮挡任务栏），半透明（复用滚动字幕透明度），纯 SVG 等级图标，多条排队显示 |
| **滚动字幕** | 屏幕顶部/底部滚动条，弹幕式多轨道（1~3 条可配，默认 2 条），多通知排队轮播、Critical 抢占，多显示器同步，5 种预设主题 |

**弹窗时长可按告警等级分别配置**（设置 → 通知）：

| 级别 | 时长 |
|------|------|
| Info | `toast_info_secs` 秒（默认 10） |
| Warning | `toast_warning_secs` 秒（默认 10） |
| Critical | `toast_critical_secs` 秒（默认 0） |

任意一项设为 `0` 时，该等级的弹窗**不自动关闭，只能手动关闭**。

### 勿扰模式 (DND)

- 手动开关 + 定时计划
- 按周重复，每日多时段
- 开启后声音、滚动字幕、弹窗统一静音，通知仍会正常接收并存入历史记录

### 番茄钟

1. 在"待办任务"页添加任务和截止日期
2. 为每个任务配置独立的工作时长、休息时长和轮次
3. 开始专注计时，完成后自动记录番茄数

### 滚动字幕

5 种预设主题：

| 主题 | 风格 |
|------|------|
| **Poster** | 深色底 + 红色字，Impact 字体 |
| **Anime** | 紫色底 + 金色字，趣味字体 |
| **Business** | 深蓝底 + 白字，衬线字体 |
| **Kawaii** | 粉色底 + 品红字，手写字体 |
| **Transparent** | 透明底 + 白字，极简 |

可自定义：位置（顶部/底部）、轨道数、速度、高度、字号、字体、前置/后置图标、背景色、文字色、背景透明度、显示时长。

## 构建打包

```bash
# TypeScript 类型检查 + Vite 构建
npm run build

# 生成当前平台安装包
npm run tauri build

# 仅编译二进制
npm run tauri build -- --bundles none

# 指定打包格式
npm run tauri build -- --bundles msi
```

产物路径：`src-tauri/target/release/bundle/`

| 平台 | 产物 |
|------|------|
| Windows | `.msi` / `.nsis.exe` |
| macOS | `.dmg` / `.app` |
| Linux | `.deb` / `.rpm` / `.AppImage` |

## 技术栈

- **前端**: React 19 + TypeScript + Vite 6 + Tailwind CSS 4
- **后端**: Rust + Tauri 2 + axum + tokio
- **插件**: tauri-plugin-notification, tauri-plugin-shell, tauri-plugin-dialog
