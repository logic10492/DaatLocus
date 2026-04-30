import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  Alert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import {
  fetchDaemonStatus,
  fetchDashboardSnapshot,
  type DaemonLifecycleState,
  type DaemonStatus,
  type DashboardSnapshot,
} from "@/lib/daemon-api";

const REFRESH_INTERVAL_MS = 5_000;

type OutputSections = Record<string, string>;

export function StatusPage({ onLogout }: { onLogout: () => void }) {
  const [daemonStatus, setDaemonStatus] = useState<DaemonStatus | null>(null);
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
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

    if (daemonResult.status === "fulfilled") {
      setDaemonStatus(daemonResult.value);
    } else {
      errors.push(formatError("Daemon status", daemonResult.reason));
    }

    if (snapshotResult.status === "fulfilled") {
      setSnapshot(snapshotResult.value);
    } else {
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

  const statusSections = useMemo(
    () => splitSections(snapshot?.status_output),
    [snapshot?.status_output],
  );
  const sleepSections = useMemo(
    () => splitSections(snapshot?.sleep_status_output),
    [snapshot?.sleep_status_output],
  );

  const recentActivity = useMemo(
    () => summarizeRecentActivity(snapshot),
    [snapshot],
  );

  const refreshLabel = lastUpdatedAt
    ? lastUpdatedAt.toLocaleTimeString()
    : "waiting";
  const healthLabel = errorMessage
    ? "degraded"
    : isRefreshing && !lastUpdatedAt
      ? "loading"
      : "live";

  return (
    <section
      id="status"
      className="mx-auto min-h-[calc(100vh-4rem)] w-full max-w-7xl px-6 py-8"
    >
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="max-w-3xl space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={errorMessage ? "destructive" : "secondary"}>
              {healthLabel}
            </Badge>
            <span className="text-xs text-muted-foreground">
              Auto refresh every {REFRESH_INTERVAL_MS / 1_000}s
            </span>
          </div>
          <h1 className="text-4xl font-semibold tracking-tight md:text-5xl">
            Status
          </h1>
          <p className="text-sm text-muted-foreground">
            A polling waterfall panel for daemon, runtime, workflow, model,
            sleep, app, and activity status.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            type="button"
            onClick={() => void refresh()}
            disabled={isRefreshing}
          >
            {isRefreshing ? "Refreshing" : "Refresh"}
          </Button>
          <Button variant="ghost" type="button" onClick={onLogout}>
            Log out
          </Button>
        </div>
      </div>

      {errorMessage ? (
        <Alert variant="destructive" className="mb-4">
          <AlertTitle>Some status cards could not refresh</AlertTitle>
          <AlertDescription>
            <pre className="whitespace-pre-wrap font-sans">{errorMessage}</pre>
          </AlertDescription>
        </Alert>
      ) : null}

      <div className="columns-1 gap-4 lg:columns-2 2xl:columns-3">
        <PanelCard
          title="Daemon"
          description={`Last refresh: ${refreshLabel}`}
          action={
            <Badge variant={lifecycleBadgeVariant(daemonStatus?.state)}>
              {daemonStatus?.state ?? "unknown"}
            </Badge>
          }
        >
          <MetricGrid
            items={[
              ["PID", daemonStatus?.pid],
              ["Port", daemonStatus?.port],
              ["Version", daemonStatus?.version],
              ["Clients", daemonStatus?.connected_clients],
              ["Started", formatTimestamp(daemonStatus?.started_at_ms)],
              ["Uptime", formatUptime(daemonStatus?.started_at_ms)],
            ]}
          />
        </PanelCard>

        <PanelCard
          title="Runtime"
          description="Current runtime and focus summary"
          action={
            <Badge variant={snapshot?.runtime_status ? "default" : "outline"}>
              {snapshot?.runtime_status ? "active" : "idle"}
            </Badge>
          }
        >
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground">
              {snapshot?.runtime_status ?? "No runtime status reported."}
            </p>
            <Separator />
            <MetricGrid
              items={[
                ["Focused app", snapshot?.focused_app ?? "none"],
                [
                  "Last cycle",
                  snapshot?.last_cycle_elapsed_ms == null
                    ? "—"
                    : `${snapshot.last_cycle_elapsed_ms}ms`,
                ],
                [
                  "Input tokens",
                  snapshot?.footer_estimated_input_tokens ?? "unknown",
                ],
              ]}
            />
            <TextBlock
              text={snapshot?.footer_context}
              empty="No footer context yet."
            />
          </div>
        </PanelCard>

        <TextPanel
          title="Overview"
          description="Runtime turn, workflow, plan, and event counts"
          text={statusSections.Overview}
        />

        <TextPanel
          title="Plan"
          description="Current actionable plan state"
          text={statusSections.Plan}
          empty="No active plan."
        />

        <TextPanel
          title="Model Usage"
          description="Token and model accounting"
          text={statusSections["Model usage"]}
        />

        <TextPanel
          title="Sleep"
          description="Background maintenance status"
          text={pickSections(sleepSections, ["Overview", "Queues"])}
        />

        <PanelCard
          title="Activity"
          description="Recent committed and live activity cells"
          action={
            <Badge variant="outline">
              {(snapshot?.activity_cells.length ?? 0) +
                (snapshot?.live_activity_cells.length ?? 0)}
            </Badge>
          }
        >
          <MetricGrid
            items={[
              ["Committed", snapshot?.activity_cells.length ?? 0],
              ["Live", snapshot?.live_activity_cells.length ?? 0],
            ]}
          />
          <Separator className="my-3" />
          <TextBlock text={recentActivity} empty="No recent activity." />
        </PanelCard>

        <TextPanel
          title="Telegram"
          description="Telegram transport and access status"
          text={snapshot?.inspect_telegram_output}
        />

        <PanelCard
          title="Pending Access"
          description="Requests waiting for Telegram ACL decisions"
          action={
            <Badge variant="outline">
              {snapshot?.pending_access_requests.length ?? 0}
            </Badge>
          }
        >
          <TextBlock
            text={summarizeList(snapshot?.pending_access_requests)}
            empty="No pending access requests."
          />
        </PanelCard>

        <PanelCard
          title="Apps"
          description="Per-app status outputs"
          action={
            <Badge variant="outline">
              {snapshot?.app_status_outputs.length ?? 0}
            </Badge>
          }
        >
          {snapshot?.app_status_outputs.length ? (
            <div className="space-y-3">
              {snapshot.app_status_outputs.map(([appId, output]) => (
                <div key={appId} className="space-y-2">
                  <div className="text-sm font-medium">{appId}</div>
                  <TextBlock text={output} />
                </div>
              ))}
            </div>
          ) : (
            <TextBlock text="" empty="No app status output." />
          )}
        </PanelCard>
      </div>
    </section>
  );
}

function PanelCard({
  title,
  description,
  action,
  children,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Card className="mb-4 inline-block w-full break-inside-avoid">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : null}
        {action ? <CardAction>{action}</CardAction> : null}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

function TextPanel({
  title,
  description,
  text,
  empty = "Waiting for the next refresh.",
}: {
  title: string;
  description: string;
  text?: string;
  empty?: string;
}) {
  return (
    <PanelCard title={title} description={description}>
      <TextBlock text={text} empty={empty} />
    </PanelCard>
  );
}

function MetricGrid({
  items,
}: {
  items: Array<[label: string, value: ReactNode]>;
}) {
  return (
    <dl className="grid grid-cols-2 gap-3">
      {items.map(([label, value]) => (
        <div key={label} className="space-y-1">
          <dt className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {label}
          </dt>
          <dd className="break-words text-sm font-medium">{value ?? "—"}</dd>
        </div>
      ))}
    </dl>
  );
}

function TextBlock({
  text,
  empty = "Waiting for the next refresh.",
}: {
  text?: string | null;
  empty?: string;
}) {
  const content = text?.trim();

  return (
    <pre className="max-h-72 overflow-auto whitespace-pre-wrap rounded-lg bg-muted/50 p-3 font-mono text-xs leading-relaxed text-muted-foreground">
      {content || empty}
    </pre>
  );
}

function splitSections(output?: string | null): OutputSections {
  if (!output?.trim()) {
    return {};
  }

  return output
    .trim()
    .split(/\n{2,}/)
    .reduce<OutputSections>((sections, section) => {
      const [titleLine, ...bodyLines] = section.split("\n");
      const title = titleLine.trim();

      if (title) {
        sections[title] = bodyLines.join("\n").trim();
      }

      return sections;
    }, {});
}

function pickSections(sections: OutputSections, titles: string[]) {
  return titles
    .map((title) => {
      const body = sections[title];
      return body ? `${title}\n${body}` : "";
    })
    .filter(Boolean)
    .join("\n\n");
}

function lifecycleBadgeVariant(state?: DaemonLifecycleState) {
  switch (state) {
    case "ready":
      return "default";
    case "failed":
      return "destructive";
    case "stopping":
      return "outline";
    case "initializing":
    default:
      return "secondary";
  }
}

function formatTimestamp(timestamp?: number) {
  if (timestamp == null) {
    return "—";
  }

  return new Date(timestamp).toLocaleString();
}

function formatUptime(startedAtMs?: number) {
  if (startedAtMs == null) {
    return "—";
  }

  const totalSeconds = Math.max(0, Math.floor((Date.now() - startedAtMs) / 1000));
  const days = Math.floor(totalSeconds / 86_400);
  const hours = Math.floor((totalSeconds % 86_400) / 3_600);
  const minutes = Math.floor((totalSeconds % 3_600) / 60);
  const seconds = totalSeconds % 60;

  if (days > 0) {
    return `${days}d ${hours}h`;
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

function summarizeRecentActivity(snapshot: DashboardSnapshot | null) {
  if (!snapshot) {
    return "";
  }

  const committed = snapshot.activity_cells
    .slice(-5)
    .map((cell) => `committed: ${activityVariant(cell)}`);
  const live = snapshot.live_activity_cells.map(
    (cell) => `live ${cell.key}: ${activityVariant(cell.cell)}`,
  );

  return [...live, ...committed].slice(-7).join("\n");
}

function activityVariant(cell: unknown) {
  if (cell && typeof cell === "object" && !Array.isArray(cell)) {
    const [variant] = Object.keys(cell as Record<string, unknown>);
    return variant ?? "Unknown";
  }

  return typeof cell;
}

function summarizeList(items?: unknown[]) {
  if (!items?.length) {
    return "";
  }

  return items.slice(0, 5).map(summarizeUnknown).join("\n\n");
}

function summarizeUnknown(value: unknown) {
  if (value == null) {
    return "—";
  }

  if (typeof value === "string") {
    return value;
  }

  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function formatError(label: string, error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  return `${label}: ${message}`;
}
