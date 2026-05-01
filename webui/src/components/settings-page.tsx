import { useEffect, useMemo, useState, type ReactNode } from "react";
import { RefreshCwIcon, TriangleAlertIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  fetchSettingsSummary,
  type SettingsCredentialStatus,
  type SettingsCredentialSummary,
  type SettingsModelSummary,
  type SettingsProviderSummary,
  type SettingsSummary,
} from "@/lib/daemon-api";
import { cn } from "@/lib/utils";

const NUMBER_FORMATTER = new Intl.NumberFormat("en-US");

type LoadState = "idle" | "loading" | "error";
type Tone = "good" | "warn" | "neutral";

type DetailItem = {
  label: string;
  value: ReactNode;
  meta?: ReactNode;
  mono?: boolean;
  breakAll?: boolean;
};

type MetricItem = {
  label: string;
  value: ReactNode;
  meta?: ReactNode;
};

export function SettingsPage() {
  const [summary, setSummary] = useState<SettingsSummary | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [loadError, setLoadError] = useState<string | null>(null);

  const providerByName = useMemo(() => {
    const providers = new Map<string, SettingsProviderSummary>();
    for (const provider of summary?.providers ?? []) {
      providers.set(provider.name, provider);
    }
    return providers;
  }, [summary]);

  useEffect(() => {
    const controller = new AbortController();
    void loadSummary(controller.signal);

    return () => controller.abort();
  }, []);

  async function loadSummary(signal?: AbortSignal) {
    setLoadState("loading");
    setLoadError(null);

    try {
      const nextSummary = await fetchSettingsSummary({ signal });
      setSummary(nextSummary);
      setLoadState("idle");
    } catch (error) {
      if (signal?.aborted) {
        return;
      }
      setLoadState("error");
      setLoadError(error instanceof Error ? error.message : String(error));
    }
  }

  const isLoading = loadState === "loading";

  return (
    <section
      id="settings"
      aria-label="Settings"
      className="min-h-screen w-full px-6 pb-10 pt-20 md:pb-12 md:pt-24"
    >
      <div className="flex w-full flex-col gap-4">
        {loadError ? (
          <Alert variant="destructive">
            <TriangleAlertIcon className="size-4" aria-hidden="true" />
            <AlertTitle>Unable to load settings</AlertTitle>
            <AlertDescription>{loadError}</AlertDescription>
          </Alert>
        ) : null}

        {summary ? (
          <SettingsGrid
            summary={summary}
            providerByName={providerByName}
            isLoading={isLoading}
            onRefresh={() => void loadSummary()}
          />
        ) : (
          <SettingsSkeleton />
        )}
      </div>
    </section>
  );
}

function SettingsGrid({
  summary,
  providerByName,
  isLoading,
  onRefresh,
}: {
  summary: SettingsSummary;
  providerByName: Map<string, SettingsProviderSummary>;
  isLoading: boolean;
  onRefresh: () => void;
}) {
  return (
    <div className="grid w-full grid-cols-1 items-start gap-4 lg:grid-cols-2 xl:grid-cols-3">
      <div className="flex min-w-0 flex-col gap-4">
        <OverviewCard
          summary={summary}
          isLoading={isLoading}
          onRefresh={onRefresh}
        />
        <RuntimeCard summary={summary} />
      </div>

      <div className="flex min-w-0 flex-col gap-4">
        <ProvidersCard providers={summary.providers} />
        <ServicesCard summary={summary} />
      </div>

      <div className="flex min-w-0 flex-col gap-4 lg:col-span-2 xl:col-span-1">
        <ModelsCard models={summary.models} providerByName={providerByName} />
      </div>
    </div>
  );
}

function OverviewCard({
  summary,
  isLoading,
  onRefresh,
}: {
  summary: SettingsSummary;
  isLoading: boolean;
  onRefresh: () => void;
}) {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Settings</CardTitle>
        <CardAction className="flex items-center gap-2">
          <Badge
            variant={summary.telegram.has_real_credentials ? "secondary" : "outline"}
            className="rounded-full"
          >
            Telegram {summary.telegram.has_real_credentials ? "ready" : "check"}
          </Badge>
          <Button
            type="button"
            variant="outline"
            size="icon-sm"
            aria-label="Refresh settings"
            onClick={onRefresh}
            disabled={isLoading}
          >
            <RefreshCwIcon
              className={cn("size-4", isLoading && "animate-spin")}
              aria-hidden="true"
            />
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent className="grid gap-4">
        <div className="grid grid-cols-2 gap-4">
          <HeroMetric label="Main" value={summary.main_model} />
          <HeroMetric
            label="Locale"
            value={summary.locale}
            meta={summary.locale_label}
          />
        </div>

        <DetailList
          items={[
            {
              label: "Loaded",
              value: formatDateTime(summary.loaded_at_ms),
            },
            {
              label: "Models",
              value: `${summary.models.length}`,
              meta: `${summary.providers.length} providers`,
            },
            {
              label: "Config",
              value: summary.config_path,
              mono: true,
              breakAll: true,
            },
            {
              label: "Home",
              value: summary.home_path,
              mono: true,
              breakAll: true,
            },
          ]}
        />
      </CardContent>
    </Card>
  );
}

function RuntimeCard({ summary }: { summary: SettingsSummary }) {
  const portChanged = summary.daemon.configured_port !== summary.daemon.serving_port;

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Runtime</CardTitle>
        <CardAction>
          <StatusBadge
            tone={summary.sandbox.enabled ? "good" : "neutral"}
            label={summary.sandbox.enabled ? "Sandbox" : "No sandbox"}
          />
        </CardAction>
      </CardHeader>
      <CardContent>
        <DetailList
          items={[
            {
              label: "Daemon",
              value: `:${summary.daemon.serving_port}`,
              meta: portChanged
                ? `configured :${summary.daemon.configured_port}`
                : undefined,
            },
            {
              label: "Filesystem",
              value: summary.sandbox.strong_filesystem,
            },
            {
              label: "Judge",
              value: summary.judge_model,
            },
            {
              label: "Hindsight",
              value: summary.hindsight_model,
            },
          ]}
        />
      </CardContent>
    </Card>
  );
}

function ProvidersCard({ providers }: { providers: SettingsProviderSummary[] }) {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Providers</CardTitle>
        <CardAction>
          <Badge variant="outline" className="rounded-full">
            {providers.length}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        {providers.length ? (
          <div className="divide-y divide-border/60">
            {providers.map((provider) => (
              <ProviderRow key={provider.name} provider={provider} />
            ))}
          </div>
        ) : (
          <EmptyState>No providers</EmptyState>
        )}
      </CardContent>
    </Card>
  );
}

function ProviderRow({ provider }: { provider: SettingsProviderSummary }) {
  const endpoint = provider.base_url ?? provider.auth_file;

  return (
    <div className="py-3 first:pt-0 last:pb-0">
      <div className="flex min-w-0 items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="truncate font-medium">{provider.name}</div>
          {endpoint ? (
            <div className="mt-1 break-all font-mono text-xs text-muted-foreground">
              {endpoint}
            </div>
          ) : null}
        </div>
        <div className="flex shrink-0 flex-wrap justify-end gap-1.5">
          <Badge variant="outline" className="rounded-full">
            {provider.provider_type}
          </Badge>
          <CredentialBadge credential={provider.credential} />
        </div>
      </div>
    </div>
  );
}

function ModelsCard({
  models,
  providerByName,
}: {
  models: SettingsModelSummary[];
  providerByName: Map<string, SettingsProviderSummary>;
}) {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Models</CardTitle>
        <CardAction>
          <Badge variant="outline" className="rounded-full">
            {models.length}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        {models.length ? (
          <div className="divide-y divide-border/60">
            {models.map((model) => (
              <ModelRow
                key={model.name}
                model={model}
                provider={providerByName.get(model.provider) ?? null}
              />
            ))}
          </div>
        ) : (
          <EmptyState>No models</EmptyState>
        )}
      </CardContent>
    </Card>
  );
}

function ModelRow({
  model,
  provider,
}: {
  model: SettingsModelSummary;
  provider: SettingsProviderSummary | null;
}) {
  const roles = [
    model.is_main ? "main" : null,
    model.is_judge ? "judge" : null,
    model.is_hindsight ? "hindsight" : null,
  ].filter((role): role is string => Boolean(role));

  return (
    <div className="py-3 first:pt-0 last:pb-0">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="truncate font-medium">{model.name}</h3>
            {roles.map((role) => (
              <Badge key={role} variant="outline" className="rounded-full">
                {role}
              </Badge>
            ))}
          </div>
          <div className="mt-1 break-all font-mono text-xs text-muted-foreground">
            {model.model_id}
          </div>
        </div>
        <div className="flex shrink-0 flex-wrap gap-1.5 sm:justify-end">
          <Badge variant="secondary" className="rounded-full">
            {provider?.provider_type ?? model.provider}
          </Badge>
          {model.thinking_budget ? (
            <Badge variant="outline" className="rounded-full">
              {model.thinking_budget}
            </Badge>
          ) : null}
        </div>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 text-xs sm:grid-cols-4">
        <MicroMetric
          label="Context"
          value={formatNumber(model.context_window_tokens)}
          meta={`${model.effective_context_window_percent}%`}
        />
        <MicroMetric
          label="Compact"
          value={formatNumber(model.auto_compact_token_limit)}
          meta={formatNumber(model.effective_context_window_tokens)}
        />
        <MicroMetric
          label="Output"
          value={formatNumber(model.max_completion_tokens)}
          meta={`${formatNumber(model.tool_output_max_tokens)} tool`}
        />
        <MicroMetric
          label="Timeout"
          value={`${model.request_timeout_secs}s`}
          meta={`${model.stream_idle_timeout_secs}s idle`}
        />
      </div>
    </div>
  );
}

function ServicesCard({ summary }: { summary: SettingsSummary }) {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Services</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="divide-y divide-border/60">
          <ServiceRow
            title="Judge"
            action={
              <StatusBadge
                tone={summary.judge.enabled ? "good" : "neutral"}
                label={summary.judge.enabled ? "Enabled" : "Disabled"}
              />
            }
            items={[
              {
                label: "Model",
                value: summary.judge.effective_model,
                meta: summary.judge.model ? "custom" : undefined,
              },
              {
                label: "Candidates",
                value: formatNumber(summary.judge.max_pairwise_candidates),
              },
              {
                label: "Cases",
                value: formatNumber(summary.judge.max_pairwise_cases),
              },
            ]}
          />

          <ServiceRow
            title="Hindsight"
            action={
              <Badge variant="outline" className="rounded-full">
                :{summary.hindsight.port}
              </Badge>
            }
            items={[
              {
                label: "Profile",
                value: summary.hindsight.profile,
                meta: `${summary.hindsight.namespace}/${summary.hindsight.bank_id}`,
              },
              {
                label: "Model",
                value: summary.hindsight.effective_model,
                meta: summary.hindsight.model ? "custom" : undefined,
              },
              {
                label: "Timeout",
                value: `${summary.hindsight.request_timeout_secs}s`,
              },
            ]}
          />

          <ServiceRow
            title="Telegram"
            action={
              <div className="flex flex-wrap justify-end gap-1.5">
                <StatusBadge
                  tone={summary.telegram.enabled ? "good" : "neutral"}
                  label={summary.telegram.enabled ? "Enabled" : "Disabled"}
                />
                <CredentialBadge credential={summary.telegram.credential} />
              </div>
            }
            items={[
              {
                label: "Poll",
                value: `${summary.telegram.poll_timeout_secs}s`,
              },
              {
                label: "Credential",
                value: credentialStatusLabel(summary.telegram.credential.status),
                meta: summary.telegram.has_real_credentials ? "ready" : "check",
              },
            ]}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function ServiceRow({
  title,
  action,
  items,
}: {
  title: string;
  action: ReactNode;
  items: MetricItem[];
}) {
  return (
    <div className="py-3 first:pt-0 last:pb-0">
      <div className="flex items-start justify-between gap-3">
        <h3 className="font-medium">{title}</h3>
        {action}
      </div>
      <div className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
        {items.map((item) => (
          <MicroMetric
            key={item.label}
            label={item.label}
            value={item.value}
            meta={item.meta}
          />
        ))}
      </div>
    </div>
  );
}

function HeroMetric({
  label,
  value,
  meta,
}: {
  label: string;
  value: ReactNode;
  meta?: ReactNode;
}) {
  return (
    <div className="min-w-0">
      <div className="text-xs uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="mt-1 truncate text-2xl font-semibold tracking-tight">
        {value}
      </div>
      {meta ? (
        <div className="mt-1 truncate text-xs text-muted-foreground">{meta}</div>
      ) : null}
    </div>
  );
}

function DetailList({ items }: { items: DetailItem[] }) {
  return (
    <div className="divide-y divide-border/60">
      {items.map((item) => (
        <DetailRow key={item.label} item={item} />
      ))}
    </div>
  );
}

function DetailRow({ item }: { item: DetailItem }) {
  return (
    <div className="grid gap-1 py-2.5 first:pt-0 last:pb-0 sm:grid-cols-[6rem_1fr] sm:gap-3">
      <div className="text-xs uppercase tracking-wide text-muted-foreground">
        {item.label}
      </div>
      <div className="min-w-0">
        <div
          className={cn(
            "font-medium",
            item.mono && "font-mono text-xs",
            item.breakAll ? "break-all" : "truncate",
          )}
        >
          {item.value}
        </div>
        {item.meta ? (
          <div className="mt-0.5 truncate text-xs text-muted-foreground">
            {item.meta}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function MicroMetric({
  label,
  value,
  meta,
}: {
  label: string;
  value: ReactNode;
  meta?: ReactNode;
}) {
  return (
    <div className="min-w-0">
      <div className="text-[0.68rem] uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="mt-0.5 truncate font-medium">{value}</div>
      {meta ? (
        <div className="mt-0.5 truncate text-muted-foreground">{meta}</div>
      ) : null}
    </div>
  );
}

function CredentialBadge({
  credential,
}: {
  credential: SettingsCredentialSummary;
}) {
  const tone = credentialTone(credential.status);

  return (
    <StatusBadge
      tone={tone}
      label={credentialStatusLabel(credential.status)}
      title={credential.source ? `Source: ${credential.source}` : undefined}
    />
  );
}

function StatusBadge({
  tone,
  label,
  title,
}: {
  tone: Tone;
  label: string;
  title?: string;
}) {
  return (
    <Badge
      variant={tone === "good" ? "secondary" : "outline"}
      className={cn(
        "rounded-full",
        tone === "warn" && "border-destructive/40 text-destructive",
      )}
      title={title}
    >
      <span
        aria-hidden="true"
        className={cn(
          "size-1.5 rounded-full",
          tone === "good" && "bg-emerald-500",
          tone === "warn" && "bg-destructive",
          tone === "neutral" && "bg-muted-foreground/45",
        )}
      />
      {label}
    </Badge>
  );
}

function EmptyState({ children }: { children: ReactNode }) {
  return <div className="py-3 text-sm text-muted-foreground">{children}</div>;
}

function SettingsSkeleton() {
  return (
    <div className="grid w-full grid-cols-1 items-start gap-4 lg:grid-cols-2 xl:grid-cols-3">
      {Array.from({ length: 5 }).map((_, index) => (
        <Card key={index} className={cn(index === 4 && "lg:col-span-2 xl:col-span-1")}>
          <CardContent className="grid gap-3">
            <div className="h-5 w-28 animate-pulse rounded bg-muted" />
            <div className="h-4 w-2/3 animate-pulse rounded bg-muted" />
            <div className="h-4 w-1/2 animate-pulse rounded bg-muted" />
            <div className="h-4 w-5/6 animate-pulse rounded bg-muted" />
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function credentialTone(status: SettingsCredentialStatus): Tone {
  switch (status) {
    case "configured":
    case "env_configured":
    case "oauth_file":
      return "good";
    case "env_missing":
    case "missing":
    case "placeholder":
      return "warn";
  }
}

function credentialStatusLabel(status: SettingsCredentialStatus) {
  switch (status) {
    case "configured":
      return "Configured";
    case "env_configured":
      return "Env ready";
    case "env_missing":
      return "Env missing";
    case "missing":
      return "Missing";
    case "placeholder":
      return "Placeholder";
    case "oauth_file":
      return "OAuth file";
  }
}

function formatNumber(value: number) {
  return NUMBER_FORMATTER.format(value);
}

function formatDateTime(timestampMs: number) {
  return new Date(timestampMs).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
  });
}
