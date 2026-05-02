import { Routes, Route } from "react-router-dom";
import { ConfigProvider, useConfig } from "@/hooks/useConfig";
import { I18nProvider } from "@/i18n/I18nProvider";
import { MainLayout } from "@/components/layout/MainLayout";
import { DashboardPage } from "@/pages/DashboardPage";
import { PomodoroPage } from "@/pages/PomodoroPage";
import { HistoryPage } from "@/pages/HistoryPage";
import { SettingsPage } from "@/pages/SettingsPage";

function AppInner() {
  const { config, loading } = useConfig();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full bg-bg-base">
        <span className="text-sm text-text-muted">Loading...</span>
      </div>
    );
  }

  return (
    <I18nProvider language={config.general.language}>
      <Routes>
        <Route element={<MainLayout />}>
          <Route index element={<DashboardPage />} />
          <Route path="pomodoro" element={<PomodoroPage />} />
          <Route path="history" element={<HistoryPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </I18nProvider>
  );
}

export default function App() {
  return (
    <ConfigProvider>
      <AppInner />
    </ConfigProvider>
  );
}
