import { Activity, ClipboardList, LockKeyhole, ScrollText, Settings, ShieldCheck } from "lucide-react";

import { cn } from "@/lib/utils";

export type AuthStatus = "anonymous" | "saved" | "authenticated";

type NavigationItem = {
  label: string;
  active?: boolean;
  disabled?: boolean;
};

const navigationItems: NavigationItem[] = [
  { label: "登录", active: true },
  { label: "状态", disabled: true },
  { label: "任务", disabled: true },
  { label: "日志", disabled: true },
];

function authStatusLabel(authStatus: AuthStatus) {
  switch (authStatus) {
    case "authenticated":
      return "Token 已验证";
    case "saved":
      return "本地有 token";
    case "anonymous":
      return "未登录";
  }
}

function authStatusIcon(authStatus: AuthStatus) {
  if (authStatus === "authenticated") {
    return <ShieldCheck className="size-4" />;
  }
  return <LockKeyhole className="size-4" />;
}

export function AppNavigation({ authStatus }: { authStatus: AuthStatus }) {
  return (
    <header className="sticky top-0 z-10 border-b bg-background/85 backdrop-blur">
      <div className="mx-auto flex h-16 w-full max-w-6xl items-center justify-between gap-4 px-6">
        <div className="flex items-center gap-3">
          <div className="flex size-9 items-center justify-center rounded-lg border bg-card text-card-foreground shadow-sm">
            <Activity className="size-4" />
          </div>
          <div>
            <div className="font-semibold tracking-tight">Daat Locus</div>
            <div className="text-xs text-muted-foreground">WebUI</div>
          </div>
        </div>

        <nav className="hidden items-center rounded-full border bg-card p-1 shadow-sm md:flex" aria-label="主导航">
          {navigationItems.map((item) => (
            <button
              key={item.label}
              type="button"
              disabled={item.disabled}
              className={cn(
                "rounded-full px-4 py-2 text-sm font-medium transition-colors",
                item.active
                  ? "bg-primary text-primary-foreground shadow"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                item.disabled && "cursor-not-allowed opacity-50 hover:bg-transparent hover:text-muted-foreground",
              )}
            >
              {item.label}
            </button>
          ))}
        </nav>

        <div className="flex items-center gap-3">
          <div className="hidden items-center gap-2 rounded-full border bg-card px-3 py-2 text-sm text-muted-foreground shadow-sm sm:flex">
            {authStatusIcon(authStatus)}
            <span>{authStatusLabel(authStatus)}</span>
          </div>
          <div className="flex items-center gap-2 text-muted-foreground md:hidden" aria-hidden="true">
            <ClipboardList className="size-4" />
            <ScrollText className="size-4" />
            <Settings className="size-4" />
          </div>
        </div>
      </div>
    </header>
  );
}
