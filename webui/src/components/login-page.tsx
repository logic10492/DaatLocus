import { type FormEvent, useState } from "react";
import { CheckCircle2, KeyRound, Loader2, LockKeyhole, TriangleAlert } from "lucide-react";

import { type AuthStatus } from "@/components/app-navigation";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  clearStoredDaemonToken,
  getStoredDaemonToken,
  storeDaemonToken,
  verifyDaemonToken,
} from "@/lib/daemon-auth";
import { cn } from "@/lib/utils";

type LoginState = "idle" | "checking" | "authenticated" | "error";

function statusTitle(loginState: LoginState) {
  switch (loginState) {
    case "checking":
      return "正在验证";
    case "authenticated":
      return "Token 已验证";
    case "error":
      return "验证失败";
    case "idle":
      return "等待 token";
  }
}

function StatusIcon({ loginState }: { loginState: LoginState }) {
  switch (loginState) {
    case "checking":
      return <Loader2 className="size-4 animate-spin" />;
    case "authenticated":
      return <CheckCircle2 className="size-4" />;
    case "error":
      return <TriangleAlert className="size-4" />;
    case "idle":
      return <KeyRound className="size-4" />;
  }
}

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
      setMessage("Token 已验证。后续页面会复用这个 token。");
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
    <section id="login" className="mx-auto flex min-h-[calc(100vh-4rem)] w-full max-w-xl items-center px-6 py-10">
      <Card className="w-full">
        <form onSubmit={handleSubmit}>
          <CardHeader className="space-y-5">
            <div className="flex items-center justify-between gap-3">
              <Badge variant="secondary" className="gap-1.5">
                <LockKeyhole className="size-3.5" />
                需要登录
              </Badge>
              <Badge variant={isAuthenticated ? "default" : "outline"} className="gap-1.5">
                <StatusIcon loginState={loginState} />
                {statusTitle(loginState)}
              </Badge>
            </div>

            <div className="space-y-2">
              <CardTitle className="text-2xl">Token 登录</CardTitle>
              <CardDescription>
                使用 daemon token 进入 Daat Locus。不会要求用户名或密码。
              </CardDescription>
            </div>
          </CardHeader>

          <CardContent className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="daemon-token">Daemon token</Label>
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
            </div>

            <Alert
              variant={isError ? "destructive" : "default"}
              className={cn(isAuthenticated && "border-primary/30 text-primary")}
            >
              <StatusIcon loginState={loginState} />
              <AlertTitle>{statusTitle(loginState)}</AlertTitle>
              <AlertDescription>{message}</AlertDescription>
            </Alert>

            <Separator />
          </CardContent>

          <CardFooter className="flex flex-col gap-3 sm:flex-row">
            <Button className="w-full" type="submit" disabled={isChecking}>
              {isChecking ? "验证中…" : "验证并登录"}
            </Button>
            <Button
              className="w-full"
              type="button"
              variant="outline"
              onClick={handleClearToken}
              disabled={isChecking || !token.trim()}
            >
              清除 token
            </Button>
          </CardFooter>
        </form>
      </Card>
    </section>
  );
}
