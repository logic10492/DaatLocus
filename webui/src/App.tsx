import { useEffect, useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import { LoginPage } from "@/components/login-page";
import { LogsPage } from "@/components/logs-page";
import { SettingsPage } from "@/components/settings-page";
import { AgentPage, StatusPage } from "@/components/status-page";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { getStoredDaemonToken } from "@/lib/daemon-auth";
import {
  createSession,
  deleteSession,
  fetchSessions,
  type SessionInfo,
} from "@/lib/daemon-api";

type AppPage = "agent" | "status" | "settings" | "logs";
const SELECTED_SESSION_STORAGE_KEY = "daat-locus.webui.selected-session-id";

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(() =>
    Boolean(getStoredDaemonToken()),
  );
  const [activePage, setActivePage] = useState(getCurrentPage);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(
    readStoredSelectedSessionId,
  );
  const [sessionError, setSessionError] = useState<string | null>(null);
  const [hasLoadedSessions, setHasLoadedSessions] = useState(false);
  const [isCreatingSession, setIsCreatingSession] = useState(false);
  const [deletingSessionId, setDeletingSessionId] = useState<string | null>(
    null,
  );

  useEffect(() => {
    function updateActivePage() {
      setActivePage(getCurrentPage());
    }

    updateActivePage();
    window.addEventListener("hashchange", updateActivePage);

    return () => window.removeEventListener("hashchange", updateActivePage);
  }, []);

  useEffect(() => {
    if (!isAuthenticated) {
      return;
    }

    const controller = new AbortController();
    void refreshSessions(controller.signal);
    const interval = window.setInterval(() => {
      void refreshSessions(controller.signal);
    }, 5000);

    return () => {
      controller.abort();
      window.clearInterval(interval);
    };
  }, [isAuthenticated]);

  useEffect(() => {
    if (selectedSessionId) {
      window.localStorage.setItem(
        SELECTED_SESSION_STORAGE_KEY,
        selectedSessionId,
      );
    } else {
      window.localStorage.removeItem(SELECTED_SESSION_STORAGE_KEY);
    }
  }, [selectedSessionId]);

  async function refreshSessions(signal?: AbortSignal) {
    try {
      const nextSessions = await fetchSessions({ signal });
      setSessions(nextSessions);
      setSessionError(null);
      setHasLoadedSessions(true);
      setSelectedSessionId((current) => {
        if (current && nextSessions.some((s) => s.session_id === current)) {
          return current;
        }
        return preferredSession(nextSessions)?.session_id ?? null;
      });
    } catch (error) {
      if (signal?.aborted) {
        return;
      }
      setSessionError(error instanceof Error ? error.message : String(error));
      setHasLoadedSessions(true);
    }
  }

  async function handleCreateSession(projectDir?: string) {
    if (isCreatingSession) {
      return;
    }
    setIsCreatingSession(true);
    setSessionError(null);
    try {
      const session = await createSession({
        projectDir,
        title: projectDir ? projectLabel(projectDir) : undefined,
      });
      setSessions((current) => [...current, session]);
      setSelectedSessionId(session.session_id);
      if (activePage !== "agent") {
        window.location.hash = "#agent";
      }
    } catch (error) {
      setSessionError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsCreatingSession(false);
    }
  }

  async function handleDeleteSession(sessionId: string) {
    if (deletingSessionId) {
      return;
    }
    setDeletingSessionId(sessionId);
    setSessionError(null);
    try {
      await deleteSession({ sessionId });
      setSessions((current) => {
        const nextSessions = current.filter(
          (session) => session.session_id !== sessionId,
        );
        setSelectedSessionId((currentSelected) => {
          if (currentSelected !== sessionId) {
            return currentSelected;
          }
          return preferredSession(nextSessions)?.session_id ?? null;
        });
        return nextSessions;
      });
    } catch (error) {
      setSessionError(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      setDeletingSessionId(null);
    }
  }

  function handleSelectSession(sessionId: string) {
    setSelectedSessionId(sessionId);
    if (activePage !== "agent") {
      window.location.hash = "#agent";
    }
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      {isAuthenticated ? (
        <SidebarProvider>
          <AppSidebar
            activePage={activePage}
            sessions={sessions}
            selectedSessionId={selectedSessionId}
            sessionError={sessionError}
            isCreatingSession={isCreatingSession}
            deletingSessionId={deletingSessionId}
            onSelectSession={handleSelectSession}
            onCreateSession={handleCreateSession}
            onDeleteSession={handleDeleteSession}
          />
          <SidebarInset className="min-h-screen">
            {renderAuthenticatedPage(activePage, selectedSessionId, {
              hasLoadedSessions,
              sessionError,
            })}
          </SidebarInset>
        </SidebarProvider>
      ) : (
        <LoginPage onAuthenticated={() => setIsAuthenticated(true)} />
      )}
    </main>
  );
}

function renderAuthenticatedPage(
  activePage: AppPage,
  selectedSessionId: string | null,
  sessionState: {
    hasLoadedSessions: boolean;
    sessionError: string | null;
  },
) {
  switch (activePage) {
    case "status":
      return <StatusPage />;
    case "settings":
      return <SettingsPage />;
    case "logs":
      return <LogsPage />;
    case "agent":
    default:
      return selectedSessionId ? (
        <AgentPage sessionId={selectedSessionId} />
      ) : (
        <NoSessionPage {...sessionState} />
      );
  }
}

function NoSessionPage({
  hasLoadedSessions,
  sessionError,
}: {
  hasLoadedSessions: boolean;
  sessionError: string | null;
}) {
  return (
    <section
      aria-label="Session required"
      className="flex min-h-screen w-full items-center justify-center px-6 py-16"
    >
      <div className="flex w-full max-w-md flex-col items-center gap-4 text-center">
        <h1 className="text-2xl font-semibold tracking-normal">
          {hasLoadedSessions ? "No session selected" : "Loading sessions"}
        </h1>
        <p className="text-sm text-muted-foreground">
          {sessionError ??
            (hasLoadedSessions
              ? "Create or select a session from the sidebar."
              : "Fetching available sessions.")}
        </p>
      </div>
    </section>
  );
}

function preferredSession(sessions: SessionInfo[]) {
  return (
    sessions.find((session) => session.scope.kind === "general") ??
    sessions[0] ??
    null
  );
}

function readStoredSelectedSessionId() {
  if (typeof window === "undefined") {
    return null;
  }
  return (
    window.localStorage.getItem(SELECTED_SESSION_STORAGE_KEY)?.trim() || null
  );
}

function projectLabel(projectDir: string) {
  const parts = projectDir.split("/").filter(Boolean);
  return parts.at(-1) ?? projectDir;
}

function getCurrentPage(): AppPage {
  if (typeof window === "undefined") {
    return "agent";
  }

  if (window.location.hash === "#status") {
    return "status";
  }
  if (window.location.hash === "#logs") {
    return "logs";
  }
  if (window.location.hash === "#settings") {
    return "settings";
  }
  return "agent";
}
