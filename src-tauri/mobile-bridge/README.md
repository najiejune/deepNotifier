# 移动端推送桥接（模板）

`tauri android init` / `tauri ios init` 生成原生工程后，按本文把桥接代码放进对应位置。Rust 侧已就绪：收到 token 后前端/原生只需调用 invoke `mobilePushTokenUpdate(token)`，由 `src-tauri/src/mobile/mod.rs` 上传服务端并持久化。

## Android（FCM）

### 1. Gradle 依赖（`gen/android/app/build.gradle.kts`）

```kotlin
plugins {
    // 文件顶部已有 id("kotlin-android") 等，追加：
    id("com.google.gms.google-services")
}
dependencies {
    implementation(platform("com.google.firebase:firebase-bom:33.7.0"))
    implementation("com.google.firebase:firebase-messaging")
    implementation("androidx.core:core-ktx:1.13.1")
}
```

`gen/android/build.gradle.kts` 顶层 `plugins {}` 加 `id("com.google.gms.google-services") version "4.4.2" apply false`。

把 Firebase 控制台下载的 `google-services.json` 放到 `gen/android/app/`。

### 2. 拷贝桥接代码

- `android/PushBridgePlugin.kt` → `gen/android/app/src/main/java/com/deepnotifier/app/PushBridgePlugin.kt`（包名与 MainActivity 一致）

### 3. 注册插件（`gen/android/app/src/main/java/com/deepnotifier/app/MainActivity.kt`）

```kotlin
class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        // 在 super.onCreate 之后：
        registerPlugin(PushBridgePlugin::class.java)
        super.onCreate(savedInstanceState)
    }
}
```

### 4. Manifest（`gen/android/app/src/main/AndroidManifest.xml`）

`<application>` 内加 service 声明（Android 13+ 通知权限插件已处理 `POST_NOTIFICATIONS`）：

```xml
<service
    android:name=".PushBridgeService"
    android:exported="false">
    <intent-filter>
        <action android:name="com.google.firebase.MESSAGING_EVENT" />
    </intent-filter>
</service>
```

### 5. 前端接线

App 启动时：`listen("push-token", e => invoke('mobilePushTokenUpdate', { token: e.payload.token }))`；并 invoke `getStoredPushToken` 补报可能错过的 token。

## iOS（Firebase → APNs）

前置：Apple Developer 账号、APNs key（.p8）上传到 Firebase 控制台、Xcode 开启 Push Notifications capability。

### 1. Podfile（`gen/apple/Podfile`）

```ruby
target 'deepNotifier_iOS' do
  pod 'FirebaseMessaging'
end
```

### 2. 拷贝桥接代码

- `ios/PushBridge.swift` → 加入 Xcode 工程（与 `ios-iPhone` target 同 target）

### 3. 注册插件（`src-tauri/src/lib.rs` 的 Builder 中、或 gen/apple 的入口 Swift）

```swift
PushBridgePlugin.register()
```

并在 AppDelegate 中设置 `Messaging.messaging().delegate = PushBridge.shared`（模板文件内有注释说明）。

### 4. 前端接线

同 Android：监听 `push-token` 事件并调 `mobilePushTokenUpdate`。

## 服务端配合

- App 注册设备时 `push_token` 可能尚未就绪（FCM 异步下发）：token 到达后通过 `POST /api/v1/devices/{id}/push-token` 补报，服务端需实现（协议见 `docs/protocol-v1.md`）。
- 杀进程推送：服务端按协议 §5 发 FCM data message，字段 `v`/`type`/`event`；App 被杀时由 `PushBridgeService.onMessageReceived` 重建系统通知（模板已含）。
