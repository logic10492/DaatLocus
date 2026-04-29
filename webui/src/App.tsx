import { useState } from "react";

import { AppNavigation, type AuthStatus } from "@/components/app-navigation";
import { LoginPage } from "@/components/login-page";
import { getStoredDaemonToken } from "@/lib/daemon-auth";

export default function App() {
  const [authStatus, setAuthStatus] = useState<AuthStatus>(() =>
    getStoredDaemonToken() ? "saved" : "anonymous",
  );

  return (
    <main className="min-h-screen bg-background text-foreground">
      <AppNavigation authStatus={authStatus} />
      <LoginPage onAuthStatusChange={setAuthStatus} />
    </main>
  );
}
