import { useEffect } from "react";
import { api, onEvent } from "@/lib/tauri";
import { usePlatform } from "@/hooks/usePlatform";

/**
 * Bridges native push-bridge events (FCM/APNs token refreshes, emitted by the
 * platform plugin as `push-token`) to the backend, which uploads the token to
 * the alert server. Mobile only; a no-op elsewhere.
 */
export function usePushTokenBridge() {
  const { isMobile } = usePlatform();

  useEffect(() => {
    if (!isMobile) return;
    const unlisten = onEvent<{ token: string }>("push-token", (e) => {
      if (e.token) api.mobilePushTokenUpdate(e.token).catch(() => {});
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [isMobile]);
}
