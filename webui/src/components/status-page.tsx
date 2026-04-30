import { useEffect, useMemo, useState } from "react";

import {
  AgentStatusAnimation,
  type AgentAnimationStatus,
} from "@/components/agent-status-animation";
import {
  Bar,
  BarChart,
  CartesianGrid,
  XAxis,
  YAxis,
} from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import {
  subscribeDashboardSnapshots,
  type DashboardSnapshot,
  type TokenUsageInfo,
} from "@/lib/daemon-api";

const DASHBOARD_STREAM_RECONNECT_MS = 1500;
const SUMMARY_TYPE_INTERVAL_MS = 28;
const TOKEN_USAGE_CHART_CONFIG = {
  total: {
    label: "Tokens",
    color: "var(--chart-2)",
  },
} satisfies ChartConfig;

type AgentStatusView = {
  animationStatus: AgentAnimationStatus;
  label: string;
};

export function StatusPage() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<Error | null>(null);

  useEffect(() => {
    let isActive = true;
    let reconnectTimeout: number | undefined;
    let subscription: ReturnType<typeof subscribeDashboardSnapshots> | null = null;

    function connect() {
      try {
        subscription = subscribeDashboardSnapshots({
          onSnapshot: (nextSnapshot) => {
            if (!isActive) {
              return;
            }

            setSnapshot(nextSnapshot);
            setLoadError(null);
            setIsLoading(false);
          },
          onError: (error) => {
            if (!isActive) {
              return;
            }

            setLoadError(error);
            setIsLoading(false);
          },
          onClose: (event) => {
            if (!isActive) {
              return;
            }

            subscription = null;
            if (event.code !== 1000) {
              setLoadError(
                new Error(
                  `Dashboard stream closed unexpectedly (${event.code || "unknown"}).`,
                ),
              );
              setIsLoading(false);
              reconnectTimeout = window.setTimeout(
                connect,
                DASHBOARD_STREAM_RECONNECT_MS,
              );
            }
          },
        });
      } catch (error) {
        if (!isActive) {
          return;
        }

        setLoadError(error instanceof Error ? error : new Error(String(error)));
        setIsLoading(false);
        reconnectTimeout = window.setTimeout(connect, DASHBOARD_STREAM_RECONNECT_MS);
      }
    }

    connect();

    return () => {
      isActive = false;
      if (reconnectTimeout !== undefined) {
        window.clearTimeout(reconnectTimeout);
      }
      subscription?.close();
    };
  }, []);

  const agentStatus = deriveAgentStatus({
    hasLoadError: Boolean(loadError),
    isLoading,
    snapshot,
  });
  const summaryText = derivePlanSummaryText(snapshot);
  const { isTyping, text: typedSummaryText } = useTypewriterText(summaryText);

  return (
    <section
      id="status"
      className="h-[calc(100vh-4rem)] w-full snap-y snap-mandatory overflow-y-auto overscroll-contain scroll-smooth"
    >
      <div className="flex min-h-full snap-start items-center justify-center px-6 py-10">
        <div className="flex flex-col items-center justify-center gap-5 text-center">
          <AgentStatusAnimation
            status={agentStatus.animationStatus}
            className="w-64 md:w-80"
          />
          <p
            aria-live="polite"
            className="min-h-6 max-w-[min(32rem,calc(100vw-3rem))] text-balance text-sm font-medium leading-6 text-muted-foreground md:text-base"
          >
            {typedSummaryText ? (
              <>
                <span>{typedSummaryText}</span>
                {isTyping ? (
                  <span
                    aria-hidden="true"
                    className="ml-0.5 inline-block h-4 w-px translate-y-0.5 bg-muted-foreground/70 motion-reduce:hidden"
                  />
                ) : null}
              </>
            ) : null}
          </p>
          <span
            aria-live="polite"
            className="sr-only"
          >
            {agentStatus.label}
          </span>
        </div>
      </div>
      <div
        className="flex min-h-full w-full snap-start items-start justify-center px-6 py-10 md:py-12"
      >
        <div className="grid w-full max-w-6xl gap-4">
          <DailyTokenUsageCard snapshot={snapshot} />
        </div>
      </div>
    </section>
  );
}

function DailyTokenUsageCard({
  snapshot,
}: {
  snapshot: DashboardSnapshot | null;
}) {
  const chartData = useMemo(() => dailyTokenUsageChartData(snapshot), [snapshot]);
  const totalTokens = chartData.reduce((sum, item) => sum + item.total, 0);
  const hasData = chartData.some((item) => item.total > 0);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Daily token usage</CardTitle>
        <CardDescription>
          最近 14 天 main 与 judge 模型合计 token 消耗
        </CardDescription>
      </CardHeader>
      <CardContent>
        {hasData ? (
          <div className="grid gap-4">
            <div className="flex items-end justify-between gap-4">
              <div>
                <div className="text-3xl font-semibold tracking-tight tabular-nums">
                  {formatCompactNumber(totalTokens)}
                </div>
                <div className="text-sm text-muted-foreground">
                  displayed tokens
                </div>
              </div>
            </div>
            <ChartContainer
              config={TOKEN_USAGE_CHART_CONFIG}
              className="h-72 w-full"
            >
              <BarChart
                accessibilityLayer
                data={chartData}
                margin={{ top: 8, right: 12, left: 0, bottom: 0 }}
              >
                <CartesianGrid vertical={false} />
                <XAxis
                  dataKey="label"
                  tickLine={false}
                  axisLine={false}
                  tickMargin={8}
                />
                <YAxis
                  width={44}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={formatCompactNumber}
                />
                <ChartTooltip
                  cursor={false}
                  content={
                    <ChartTooltipContent
                      valueFormatter={(value) =>
                        typeof value === "number"
                          ? formatCompactNumber(value)
                          : value
                      }
                    />
                  }
                />
                <Bar
                  dataKey="total"
                  fill="var(--color-total)"
                  radius={[8, 8, 0, 0]}
                />
              </BarChart>
            </ChartContainer>
          </div>
        ) : (
          <div className="flex min-h-72 items-center justify-center rounded-lg border border-dashed text-sm text-muted-foreground">
            暂无 token usage 数据
          </div>
        )}
      </CardContent>
    </Card>
  );
}

type DailyTokenUsageChartDatum = {
  date: string;
  label: string;
  total: number;
};

function dailyTokenUsageChartData(
  snapshot: DashboardSnapshot | null,
): DailyTokenUsageChartDatum[] {
  const usageByDate = new Map<string, number>();

  for (const info of [
    snapshot?.token_usage?.main,
    snapshot?.token_usage?.judge,
  ]) {
    mergeDailyTokenUsage(usageByDate, info);
  }

  return Array.from(usageByDate.entries())
    .sort(([leftDate], [rightDate]) => leftDate.localeCompare(rightDate))
    .slice(-14)
    .map(([date, total]) => ({
      date,
      label: formatDateLabel(date),
      total,
    }));
}

function mergeDailyTokenUsage(
  usageByDate: Map<string, number>,
  info: TokenUsageInfo | null | undefined,
) {
  for (const day of info?.daily_token_usage ?? []) {
    usageByDate.set(
      day.date,
      (usageByDate.get(day.date) ?? 0) + Math.max(0, day.usage.total_tokens),
    );
  }
}

function formatDateLabel(date: string) {
  const [, month, day] = date.match(/^(\d{4})-(\d{2})-(\d{2})$/) ?? [];

  if (!month || !day) {
    return date;
  }

  return `${month}/${day}`;
}

function formatCompactNumber(value: number) {
  return new Intl.NumberFormat("en", {
    compactDisplay: "short",
    maximumFractionDigits: value >= 1000 ? 1 : 0,
    notation: "compact",
  }).format(value);
}

function useTypewriterText(text: string) {
  const characters = useMemo(() => Array.from(text), [text]);
  const [visibleCharacters, setVisibleCharacters] = useState(0);

  useEffect(() => {
    setVisibleCharacters(0);

    if (characters.length === 0) {
      return;
    }

    let nextLength = 0;
    const intervalId = window.setInterval(() => {
      nextLength += 1;
      setVisibleCharacters(nextLength);

      if (nextLength >= characters.length) {
        window.clearInterval(intervalId);
      }
    }, SUMMARY_TYPE_INTERVAL_MS);

    return () => window.clearInterval(intervalId);
  }, [characters]);

  return {
    isTyping: visibleCharacters < characters.length,
    text: characters.slice(0, visibleCharacters).join(""),
  };
}

function derivePlanSummaryText(snapshot: DashboardSnapshot | null) {
  const planStep = snapshot?.current_plan_step;

  if (!planStep?.step.trim()) {
    return "";
  }

  const prefix = planStep.status === "pending" ? "下一步" : "正在";

  return `${prefix}：${planStep.step.trim()}`;
}

function deriveAgentStatus({
  hasLoadError,
  isLoading,
  snapshot,
}: {
  hasLoadError: boolean;
  isLoading: boolean;
  snapshot: DashboardSnapshot | null;
}): AgentStatusView {
  if (isLoading && !snapshot) {
    return { animationStatus: "waiting", label: "加载中" };
  }

  if (hasLoadError && !snapshot) {
    return { animationStatus: "waiting", label: "状态不可用" };
  }

  if (!snapshot?.runtime_status) {
    return { animationStatus: "idle", label: "空闲" };
  }

  const runtimeStatus = snapshot.runtime_status.toLowerCase();
  const dashboardText = [snapshot.runtime_status, snapshot.status_output]
    .join(" ")
    .toLowerCase();

  if (/\b(error|failed|failure|panic)\b/.test(dashboardText)) {
    return { animationStatus: "error", label: "异常" };
  }

  if (/\b(waiting|backlog|pending|sleep)\b/.test(runtimeStatus)) {
    return { animationStatus: "waiting", label: "等待中" };
  }

  if (
    snapshot.focused_app &&
    /\b(action|app|browser|terminal|tool)\b/.test(dashboardText)
  ) {
    return { animationStatus: "tooling", label: "调用工具" };
  }

  if (/\b(compacting|context|model|reason|thinking|working)\b/.test(dashboardText)) {
    return { animationStatus: "thinking", label: "思考中" };
  }

  return { animationStatus: "running", label: "执行中" };
}
