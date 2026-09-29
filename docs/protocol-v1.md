# deepNotifier 客户端-服务端协议 v1

本协议是 deepNotifier 移动端客户端与告警服务端之间的对接契约。服务端由独立程序实现（本仓库只含客户端与一个 demo 参考实现 `crates/dn-core/examples/demo_server.rs`）。类型定义见 `crates/dn-core/src/protocol.rs` 与 `models.rs`。

约定：

- 所有 JSON 消息含 `"v": 1` 字段，为协议版本号。
- 所有时间均为 RFC 3339 格式（带时区偏移）。
- 开发环境允许纯文本 `ws://`，生产环境**必须**使用 `wss://`（TLS）。

## 1. 鉴权

| 凭据 | 用途 | 传递方式 |
|------|------|----------|
| `api_token` | 用户级凭据，由服务端管理 | REST 请求头 `Authorization: Bearer <api_token>` |
| `ws_token` | 设备级凭据，由设备注册接口签发 | WebSocket URL query `?token=<ws_token>` |

客户端首次使用流程：用户在 App 设置页填写 `server_url` + `api_token` → App 调 `POST /api/v1/devices` 注册 → 保存返回的 `device_id` + `ws_token` → 建立 WS 连接。

## 2. REST 接口

### POST /api/v1/devices — 注册设备

请求：

```json
{
  "platform": "android",
  "push_token": "fcm-or-apns-token，可省略",
  "name": "Pixel 8"
}
```

`platform` 取值：`android` | `ios`。

响应 200：

```json
{ "device_id": "dev_x7f3a9", "ws_token": "wst_4b8c..." }
```

### DELETE /api/v1/devices/{device_id} — 注销设备

响应 204。App 内「注销/断开」按钮调用。

### POST /api/v1/devices/{device_id}/push-token — 更新推送 token

请求：`{ "push_token": "..." }`，响应 204。

FCM/APNs 的 device token 会被系统轮换，App 每次拿到新 token 都必须调用此接口。`push_token` 为 FCM token（Android 与 iOS 统一走 Firebase，服务端只需对接 FCM）。

### GET /api/v1/events?since=<rfc3339> — 断线补拉

返回自 `since` 之后的事件，**最多 200 条**，按时间升序：

```json
{ "events": [ /* NotificationEvent 数组 */ ] }
```

### GET /healthz — 健康检查

响应 200，body 任意。

## 3. WebSocket（前台通道，客户端主动发起）

- 地址：`wss://<server>/api/v1/ws?token=<ws_token>`
- 连接建立后，**服务端先**发送 `hello`，客户端回以 `hello`。

### 服务端 → 客户端

```json
{ "v": 1, "type": "hello", "server_time": "2025-01-01T12:00:00+08:00", "missed_events": 0 }
{ "v": 1, "type": "notification", "event": { /* NotificationEvent */ } }
{ "v": 1, "type": "ping", "ts": 1700000000 }
```

- `hello.missed_events`：本次断线期间积压的事件数。大于 0 时客户端应调用 `GET /api/v1/events?since=<本地最后一条事件时间>` 补拉（然后按 `event.id` 去重合并）。
- `notification.event.raw_payload` 恒为 `null`（体积控制），需要原始载荷时走补拉接口。

### 客户端 → 服务端

```json
{ "v": 1, "type": "hello", "platform": "android", "app_version": "0.2.0" }
{ "v": 1, "type": "pong", "ts": 1700000000 }
{ "v": 1, "type": "ack", "ids": ["<event-id>"] }
```

- **心跳**：服务端每 25 秒发一次 `ping`，客户端回 `pong`（`ts` 原样返回）。连续 2 次未收到 `pong`，服务端断开连接。
- `ack`：客户端成功落库/展示后回执，服务端可据此做送达统计（可选实现）。

## 4. 事件载荷 NotificationEvent

与桌面端内部模型完全一致（`crates/dn-core/src/models.rs`）：

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "source": "GitHub",
  "event_type": "pull_request.opened",
  "title": "PR #42 opened",
  "body": "feat: add mobile support",
  "severity": "Info",
  "timestamp": "2025-01-01T12:00:00+08:00",
  "url": "https://github.com/org/repo/pull/42",
  "raw_payload": null,
  "pid": null
}
```

| 字段 | 说明 |
|------|------|
| `id` | UUID v4，全局唯一，**去重依据** |
| `source` | `GitHub` / `GitLab` / `Bitbucket` / `Custom` / `Poll` / `Timer` / `Pomodoro` / `System` / `Hook`（PascalCase；`Poll`/`Hook` 带字符串参数） |
| `event_type` | 平台事件类型，如 `pull_request.opened`、`push`、自定义字符串 |
| `severity` | `Info` / `Warning` / `Critical` |
| `timestamp` | 事件产生时间 |
| `url` | 可空，点击通知时的跳转地址 |
| `raw_payload` | 可空，WS 通道恒为 null |
| `pid` | 可空，桌面端专用（进程焦点），服务端推送时给 null 或省略 |

## 5. Push 推送（后台通道）

App 被杀/后台时靠系统推送。服务端职责：

1. 设备**有活跃 WS 连接** → 只走 WS，不发 push。
2. 设备**离线** → 调 FCM 发送（iOS 由 Firebase 转发为 APNs）：

**Android（FCM data message）**：

```json
{
  "message": {
    "token": "<fcm token>",
    "data": {
      "v": "1",
      "type": "notification",
      "event": "{\"id\":\"...\",\"title\":\"...\"}"
    }
  }
}
```

`event` 为整个 NotificationEvent 的 **JSON 字符串**（FCM data 值只接受字符串）。App 原生层解析后构建系统通知。

**iOS（Firebase 转发 APNs）**：alert 用 `title`/`body`，自定义 payload 同上结构。

去重：push 的 `event.id` 与 WS 可能重复到达（竞态），App 侧以 `event.id` 去重兜底。

## 6. 参考实现

`crates/dn-core/examples/demo_server.rs`：单文件 axum 实现，含 `/api/v1/ws`（广播 + ping）、`POST /api/v1/devices`（固定凭据）、`POST /push`（模拟告警入口）。**不实现**设备持久化、FCM、补拉，仅供联调：

```bash
cargo run -p dn-core --example demo_server
curl -X POST localhost:3930/push \
  -H "Content-Type: application/json" \
  -d '{"title":"Deploy finished","body":"Build #42 passed","severity":"Info"}'
```
