import FirebaseMessaging
import UserNotifications
import WebKit
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

/// Push bridge for iOS (Firebase forwards APNs; token is an FCM token).
///
/// Registration (after `tauri ios init`):
///   1. Add `pod 'FirebaseMessaging'` to gen/apple/Podfile and run `pod install`.
///   2. In the app's AppDelegate (gen/apple/Sources/), after FirebaseApp.configure():
///        Messaging.messaging().delegate = PushBridge.shared
///   3. Call PushBridge.shared.emitToken whenever messaging(_:didReceiveRegistrationToken:)
///      fires (the extension below does this if you keep it in the same target).
///   4. Frontend: listen for the "push-token" JS event and call
///      invoke('mobilePushTokenUpdate', { token }), same as Android.
class PushBridge: NSObject, Plugin {
    static let shared = PushBridge()
    private(set) var plugin: (any Plugin)?
    private var storedToken: String?

    func register(plugin: any Plugin) { self.plugin = plugin }

    /// Called from messaging(_:didReceiveRegistrationToken:).
    func emitToken(_ token: String) {
        storedToken = token
        let obj = JSObject()
        obj["token"] = token
        plugin?.trigger("push-token", data: obj)
    }

    /// Entry point for foreground presentation options.
    func willPresent(
        _ notification: UNNotification,
        completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound])
    }
}

extension PushBridge: MessagingDelegate {
    func messaging(
        _ messaging: Messaging,
        didReceiveRegistrationToken fcmToken: String?
    ) {
        if let token = fcmToken {
            emitToken(token)
        }
    }
}
