#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Keep hidden webview pages (e.g. the marquee strip windows, which stay
    // hidden 99% of the time) from being throttled/put to sleep by WebView2.
    // Otherwise their JS stops processing Tauri events: the marquee window
    // shows up with stale text or no text at all.
    // Must be set before any WebView2 loader initializes, i.e. at the very
    // start of main.
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-background-timer-throttling --disable-backgrounding-occluded-windows --disable-renderer-backgrounding",
    );
    deep_notifier_lib::run()
}
