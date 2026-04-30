import { useCallback, useEffect, useRef, useState } from "react";

import {
  Alert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { fetchDaemonStatus, fetchDashboardSnapshot } from "@/lib/daemon-api";

const REFRESH_INTERVAL_MS = 5_000;

export function StatusPage() {
  const [lastUpdatedAt, setLastUpdatedAt] = useState<Date | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const isMountedRef = useRef(true);

  const refresh = useCallback(async (signal?: AbortSignal) => {
    setIsRefreshing(true);

    const [daemonResult, snapshotResult] = await Promise.allSettled([
      fetchDaemonStatus({ signal }),
      fetchDashboardSnapshot({ signal }),
    ]);

    if (signal?.aborted || !isMountedRef.current) {
      return;
    }

    const errors: string[] = [];

    if (daemonResult.status === "rejected") {
      errors.push(formatError("Daemon status", daemonResult.reason));
    }

    if (snapshotResult.status === "rejected") {
      errors.push(formatError("Dashboard snapshot", snapshotResult.reason));
    }

    setErrorMessage(errors.join("\n"));
    setLastUpdatedAt(new Date());
    setIsRefreshing(false);
  }, []);

  useEffect(() => {
    isMountedRef.current = true;

    const controller = new AbortController();
    void refresh(controller.signal);

    const intervalId = window.setInterval(() => {
      void refresh();
    }, REFRESH_INTERVAL_MS);

    return () => {
      isMountedRef.current = false;
      controller.abort();
      window.clearInterval(intervalId);
    };
  }, [refresh]);

  const refreshLabel = lastUpdatedAt ? lastUpdatedAt.toLocaleTimeString() : "—";
  const healthLabel = errorMessage
    ? "degraded"
    : isRefreshing && !lastUpdatedAt
      ? "loading"
      : "live";

  return (
    <section
      id="status"
      className="mx-auto min-h-[calc(100vh-4rem)] w-full max-w-7xl px-6 py-10"
    >
      <div className="space-y-8">
        <header className="space-y-6">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline">Status</Badge>
            <Badge variant={errorMessage ? "destructive" : "secondary"}>
              {healthLabel}
            </Badge>
          </div>

          <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_22rem] lg:items-end">
            <div className="max-w-3xl space-y-3">
              <p className="text-sm font-medium text-muted-foreground">
                Auto-updating runtime overview
              </p>
              <h1 className="text-4xl font-semibold tracking-tight md:text-6xl">
                Status
              </h1>
              <p className="text-base leading-7 text-muted-foreground">
                The Status page shell is ready for the card-by-card redesign.
                It keeps the background refresh loop, while the dashboard
                canvas stays empty until each card is intentionally added.
              </p>
            </div>

            <div className="rounded-3xl border bg-card/50 p-4 text-sm">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-xs text-muted-foreground">Refresh</p>
                  <p className="mt-1 font-medium">
                    Every {REFRESH_INTERVAL_MS / 1_000}s
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Last sync</p>
                  <p className="mt-1 font-medium">{refreshLabel}</p>
                </div>
              </div>
              <Separator className="my-4" />
              <p className="text-xs leading-5 text-muted-foreground">
                Data sources stay connected through `/status` and
                `/dashboard/snapshot`; manual controls were removed from the
                page chrome.
              </p>
            </div>
          </div>
        </header>

        {errorMessage ? (
          <Alert variant="destructive">
            <AlertTitle>Status refresh failed</AlertTitle>
            <AlertDescription>
              <pre className="whitespace-pre-wrap font-sans">
                {errorMessage}
              </pre>
            </AlertDescription>
          </Alert>
        ) : null}

        <div className="overflow-hidden rounded-4xl border bg-muted/20">
          <div className="border-b bg-background/80 px-5 py-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <h2 className="text-sm font-medium">Waterfall canvas</h2>
                <p className="mt-1 text-xs text-muted-foreground">
                  Empty by design; new cards will be placed here one at a time.
                </p>
              </div>
              <Badge variant="outline">No cards yet</Badge>
            </div>
          </div>

          <div className="grid min-h-[22rem] gap-4 p-4 md:grid-cols-2 xl:grid-cols-3">
            <div className="rounded-3xl border border-dashed bg-background/40" />
            <div className="hidden rounded-3xl border border-dashed bg-background/40 md:block" />
            <div className="hidden rounded-3xl border border-dashed bg-background/40 xl:block" />
          </div>
        </div>
      </div>
    </section>
  );
}

function formatError(label: string, reason: unknown) {
  if (reason instanceof Error) {
    return `${label}: ${reason.message}`;
  }

  return `${label}: ${String(reason)}`;
}
