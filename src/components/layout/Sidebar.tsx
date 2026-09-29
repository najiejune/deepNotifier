import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  ClipboardList,
  History,
  Settings,
  ChevronRight,
} from "lucide-react";
import { cn } from "@/lib/cn";
import { useI18n } from "@/i18n";

export function Sidebar() {
  const t = useI18n();

  const navItems = [
    { to: "/", icon: LayoutDashboard, label: t.nav.dashboard },
    { to: "/pomodoro", icon: ClipboardList, label: t.nav.pomodoro },
    { to: "/history", icon: History, label: t.nav.history },
    { to: "/settings", icon: Settings, label: t.nav.settings },
  ];

  return (
    <aside className="w-40 shrink-0 bg-white border-r border-border-subtle flex flex-col max-sm:w-full max-sm:flex-row max-sm:items-stretch max-sm:border-r-0 max-sm:border-b">
      <nav className="flex-1 py-3 px-2 space-y-0.5 max-sm:flex max-sm:items-center max-sm:justify-around max-sm:gap-1 max-sm:py-1.5 max-sm:space-y-0">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            title={label}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2.5 px-3 py-2 rounded-sm text-sm font-medium transition-all duration-150",
                "max-sm:justify-center max-sm:px-2",
                isActive
                  ? "bg-accent-dim text-accent"
                  : "text-text-secondary hover:text-text hover:bg-bg-layer",
              )
            }
          >
            {({ isActive }) => (
              <>
                <Icon size={16} />
                <span className="flex-1 max-sm:hidden">{label}</span>
                {isActive && <ChevronRight size={12} className="max-sm:hidden" />}
              </>
            )}
          </NavLink>
        ))}
      </nav>

      <div className="px-3 py-3 border-t border-border-subtle max-sm:hidden">
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <span className="w-1.5 h-1.5 rounded-full bg-cyan" />
          {t.app.systemOnline}
        </div>
      </div>
    </aside>
  );
}
