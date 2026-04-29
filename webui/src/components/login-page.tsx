import { type FormEvent, useState } from "react";
import { KeyRound, LockKeyhole, ShieldCheck, Terminal, TriangleAlert } from "lucide-react";

import { type AuthStatus } from "@/components/app-navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  clearStoredDaemonToken,
  getStoredDaemonToken,
  storeDaemonToken,
  verifyDaemonToken,
} from "@/lib/daemon-auth";
import { cn } from "@/lib/utils";

type LoginState = "idle" | "checking" | "authenticated" | "error";

const loginNotes = [
  "使用 daemon token 作为唯一登录凭据",
  "登录请求会携带 Authorization: Bearer <token>",
  "Vite dev server 已代理 daemon API，内置运行时使用同源路径",
];

export function LoginPage({
  onAuthStatusChange,
}: {
  onAuthStatusChange: (status: AuthStatus) => void;
}) {
  const [token, setToken] = useState(() => getStoredDaemonToken());
  const [loginState, setLoginState] = useState<LoginState>("idle");
  const [message, setMessage] = useState(() =>
    getStoredDaemonToken()
      ? "检测到本地已保存 token。你可以直接验证，或粘贴新的 daemon token。"
      : "请输入 daemon token 登录。",
  );

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedToken = token.trim();

    if (!trimmedToken) {
      setLoginState("error");
      setMessage("请输入 daemon token。");
      onAuthStatusChange("anonymous");
      return;
    }

    setLoginState("checking");
    setMessage("正在验证 token……");

    const result = await verifyDaemonToken(trimmedToken);
    if (result.ok) {
      storeDaemonToken(trimmedToken);
      setToken(trimmedToken);
      setLoginState("authenticated");
      setMessage("Token 已验证。后续状态、任务和日志页面会复用这个 token 调用 daemon API。");
      onAuthStatusChange("authenticated");
      return;
    }

    setLoginState("error");
    setMessage(result.message);
    onAuthStatusChange("anonymous");
  }

  function handleClearToken() {
    clearStoredDaemonToken();
    setToken("");
    setLoginState("idle");
    setMessage("本地 token 已清除，请重新输入 daemon token。");
    onAuthStatusChange("anonymous");
  }

  const isChecking = loginState === "checking";
  const isAuthenticated = loginState === "authenticated";
  const isError = loginState === "error";

  return (
    <section id="login" className="mx-auto grid min-h-[calc(100vh-4rem)] w-full max-w-6xl items-center gap-10 px-6 py-10 lg:grid-cols-[0.9fr_1.1fr]">
      <div className="space-y-7">
        <div className="inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1 text-sm text-muted-foreground shadow-sm">
          <LockKeyhole className="size-4" />
          Login first
        </div>

        <div className="space-y-5">
          <h1 className="max-w-2xl text-4xl font-semibold tracking-tight sm:text-6xl">
            使用 daemon token 登录 WebUI。
          </h1>
          <p className="max-w-xl text-lg leading-8 text-muted-foreground">
            先从登录页开始，后续导航里的状态、任务、日志等页面都基于同一套 daemon token 认证。
          </p>
        </div>

        <div className="space-y-3">
          {loginNotes.map((item) => (
            <div key={item} className="flex items-start gap-3 rounded-xl border bg-card p-3 text-card-foreground shadow-sm">
              <ShieldCheck className="mt-0.5 size-5 shrink-0 text-primary" />
              <span className="text-sm leading-6">{item}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="rounded-2xl border bg-card p-6 text-card-foreground shadow-sm sm:p-8">
        <div className="mb-8 flex items-start justify-between gap-4">
          <div>
            <p className="text-sm text-muted-foreground">Daat Locus WebUI</p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight">Token 登录</h2>
          </div>
          <div className="rounded-full bg-primary/10 p-3 text-primary">
            <KeyRound className="size-5" />
          </div>
        </div>

        <form className="space-y-5" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <label htmlFor="daemon-token" className="text-sm font-medium leading-none">
              Daemon token
            </label>
            <Input
              id="daemon-token"
              value={token}
              onChange={(event) => {
                setToken(event.target.value);
                if (loginState !== "checking") {
                  setLoginState("idle");
                  onAuthStatusChange(event.target.value.trim() ? "saved" : "anonymous");
                }
              }}
              placeholder="粘贴 daemon token"
              type="password"
              autoComplete="current-password"
              spellCheck={false}
              disabled={isChecking}
            />
            <p className="text-xs leading-5 text-muted-foreground">
              这里使用现有 daemon token，不引入用户名/密码。验证成功后 token 会保存在当前浏览器本地，供后续 API 请求复用。
            </p>
          </div>

          <div
            className={cn(
              "flex items-start gap-3 rounded-xl border p-3 text-sm leading-6",
              isAuthenticated && "border-primary/30 bg-primary/5 text-primary",
              isError && "border-destructive/30 bg-destructive/5 text-destructive",
              !isAuthenticated && !isError && "bg-muted/40 text-muted-foreground",
            )}
            aria-live="polite"
          >
            {isError ? <TriangleAlert className="mt-0.5 size-4 shrink-0" /> : <Terminal className="mt-0.5 size-4 shrink-0" />}
            <span>{message}</span>
          </div>

          <div className="flex flex-col gap-3 sm:flex-row">
            <Button className="sm:flex-1" size="lg" type="submit" disabled={isChecking}>
              {isChecking ? "验证中…" : "验证并登录"}
            </Button>
            <Button size="lg" type="button" variant="outline" onClick={handleClearToken} disabled={isChecking || !token.trim()}>
              清除 token
            </Button>
          </div>
        </form>
      </div>
    </section>
  );
}
