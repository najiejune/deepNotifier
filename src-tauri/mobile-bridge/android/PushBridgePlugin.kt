package com.deepnotifier.app

import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.webkit.WebView
import androidx.core.app.NotificationCompat
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONObject

private const val PREFS = "push_bridge"
private const val KEY_TOKEN = "fcm_token"
private const val CHANNEL_ID = "alerts"
private const val EXTRA_EVENT_JSON = "event_json"

/** Token holder shared between the FCM service and the plugin instance. */
object PushBridge {
    var plugin: PushBridgePlugin? = null
    var onToken: ((String) -> Unit)? = null
}

@TauriPlugin
class PushBridgePlugin(private val activity: Activity) : Plugin(activity) {
    private val prefs: SharedPreferences =
        activity.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    override fun load(webView: WebView) {
        PushBridge.plugin = this
        // Report a token that arrived while the WebView wasn't up yet.
        prefs.getString(KEY_TOKEN, null)?.let { token ->
            PushBridge.onToken?.invoke(token)
        }
        PushBridge.onToken = { token -> emitToken(token) }
    }

    @Command
    fun getStoredPushToken(invoke: Invoke) {
        val ret = JSObject()
        ret.put("token", prefs.getString(KEY_TOKEN, null))
        invoke.resolve(ret)
    }

    private fun emitToken(token: String) {
        val obj = JSObject()
        obj.put("token", token)
        trigger("push-token", obj)
    }
}

class PushBridgeService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        super.onNewToken(token)
        getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit().putString(KEY_TOKEN, token).apply()
        PushBridge.onToken?.invoke(token)
    }

    /**
     * FCM data message (protocol v1 §5): data["event"] is the serialized
     * NotificationEvent JSON. Build a system notification from it so alerts
     * arrive even when the app was killed.
     */
    override fun onMessageReceived(message: RemoteMessage) {
        super.onMessageReceived(message)
        val data = message.data
        if (data["type"] != "notification") return
        val eventJson = data["event"] ?: return

        val title = data["title"] ?: "deepNotifier"
        val body = data["body"] ?: ""
        val event = runCatching { JSONObject(eventJson) }.getOrNull()
        val severity = event?.optString("severity") ?: "Info"

        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, "Alerts", NotificationManager.IMPORTANCE_HIGH)
        )

        val launch = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            putExtra(EXTRA_EVENT_JSON, eventJson)
        }
        val pending = PendingIntent.getActivity(
            this, 0, launch,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification) // 或用 android.R.drawable.ic_dialog_info
            .setContentTitle(title)
            .setContentText(body)
            .setPriority(if (severity == "Critical") NotificationCompat.PRIORITY_HIGH else NotificationCompat.PRIORITY_DEFAULT)
            .setAutoCancel(true)
            .setContentIntent(pending)
        // Critical alerts bypass DND on the device.
        if (severity == "Critical") builder.setCategory(NotificationCompat.CATEGORY_ALARM)

        manager.notify(event?.optString("id")?.hashCode() ?: System.currentTimeMillis().toInt(), builder.build())
    }
}
