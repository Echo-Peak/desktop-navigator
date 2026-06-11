import { NavLink, Outlet } from "react-router-dom";
import {
  Boxes,
  Cable,
  ListChecks,
  ShieldCheck,
  Sparkles,
  Sliders,
  Home as HomeIcon
} from "lucide-react";
import { cn } from "@/lib/cn";
import { SessionProvider } from "@/state/session";
import { ControlBar } from "./ControlBar";
import { SessionOverlay } from "./SessionOverlay";
import { Onboarding } from "./Onboarding";
import { UpdatePrompt } from "./UpdatePrompt";
import { Toast } from "./Toast";

const NAV_ITEMS = [
  { to: "/", label: "Home", icon: HomeIcon, end: true },
  { to: "/browser-config", label: "Browser Config", icon: Sliders },
  { to: "/automations", label: "Automations", icon: Boxes },
  { to: "/integrations", label: "Integrations", icon: Cable },
  { to: "/tasks", label: "Tasks", icon: ListChecks },
  { to: "/llm-integrations", label: "LLM Integrations", icon: Sparkles },
  { to: "/captcha-resolvers", label: "Captcha Resolvers", icon: ShieldCheck }
];

export function AppLayout() {
  return (
    <SessionProvider>
      <div className="grid h-screen grid-cols-[240px_1fr]">
        <aside className="flex flex-col gap-1 border-r border-border bg-panel p-3">
          <div className="px-3 pb-4 pt-2 text-sm font-semibold">
            Desktop Navigator
          </div>
          <nav className="flex flex-col gap-0.5">
            {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => (
              <NavLink
                key={to}
                to={to}
                end={end}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-muted transition-colors hover:bg-panel-2 hover:text-fg",
                    isActive && "bg-accent/15 text-fg"
                  )
                }
              >
                <Icon size={16} />
                {label}
              </NavLink>
            ))}
          </nav>
        </aside>

        <div className="flex min-w-0 flex-col">
          <ControlBar />
          <main className="min-h-0 flex-1 overflow-y-auto p-8">
            <Outlet />
          </main>
        </div>
      </div>

      <SessionOverlay />
      <Onboarding />
      <UpdatePrompt />
      <Toast />
    </SessionProvider>
  );
}
