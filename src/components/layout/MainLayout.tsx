import { Outlet } from "react-router-dom";
import { TitleBar } from "./TitleBar";
import { Sidebar } from "./Sidebar";
import { usePlatform } from "@/hooks/usePlatform";

export function MainLayout() {
  const { isMobile } = usePlatform();

  return (
    <div className="flex flex-col h-full bg-bg-layer">
      {!isMobile && <TitleBar />}
      <div className="flex flex-1 overflow-hidden max-sm:flex-col max-sm:min-h-0">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-bg-layer max-sm:min-h-0">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
