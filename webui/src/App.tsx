import { useEffect, useState } from "react";

import { AppNavigation } from "@/components/app-navigation";
import { LoginPage } from "@/components/login-page";
import { AgentPage, StatusPage } from "@/components/status-page";
import { getStoredDaemonToken } from "@/lib/daemon-auth";

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(() =>
    Boolean(getStoredDaemonToken()),
  );
  const [activePage, setActivePage] = useState(getCurrentPage);

  useEffect(() => {
    function updateActivePage() {
      setActivePage(getCurrentPage());
    }

    updateActivePage();
    window.addEventListener("hashchange", updateActivePage);

    return () => window.removeEventListener("hashchange", updateActivePage);
  }, []);

  return (
    <main className="min-h-screen bg-background text-foreground">
      <AppNavigation isAuthenticated={isAuthenticated} />
      {isAuthenticated ? (
        activePage === "status" ? (
          <StatusPage />
        ) : (
          <AgentPage />
        )
      ) : (
        <LoginPage onAuthenticated={() => setIsAuthenticated(true)} />
      )}
    </main>
  );
}

function getCurrentPage() {
  if (typeof window === "undefined") {
    return "agent";
  }

  return window.location.hash === "#status" ? "status" : "agent";
}
