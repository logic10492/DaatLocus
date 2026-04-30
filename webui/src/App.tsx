import { useState } from "react";

import { AppNavigation } from "@/components/app-navigation";
import { LoginPage } from "@/components/login-page";
import { StatusPage } from "@/components/status-page";
import { clearStoredDaemonToken, getStoredDaemonToken } from "@/lib/daemon-auth";

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(() =>
    Boolean(getStoredDaemonToken()),
  );

  function handleLogout() {
    clearStoredDaemonToken();
    setIsAuthenticated(false);
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <AppNavigation isAuthenticated={isAuthenticated} />
      {isAuthenticated ? (
        <StatusPage onLogout={handleLogout} />
      ) : (
        <LoginPage onAuthenticated={() => setIsAuthenticated(true)} />
      )}
    </main>
  );
}
