import { Outlet } from "react-router-dom";
import { TitleBar } from "./TitleBar";
import { Sidebar } from "./Sidebar";

export function MainLayout() {
  return (
    <div className="flex flex-col h-full bg-bg-layer">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-bg-layer">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
