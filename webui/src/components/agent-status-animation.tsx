import { useEffect, useState } from "react";

import { cn } from "@/lib/utils";

export type AgentAnimationStatus =
  | "idle"
  | "thinking"
  | "running"
  | "tooling"
  | "waiting"
  | "error";

type AgentStatusAnimationProps = {
  status: AgentAnimationStatus;
  className?: string;
};

const expressionToneByStatus: Record<AgentAnimationStatus, string> = {
  idle: "text-foreground",
  thinking: "text-primary",
  running: "text-primary",
  tooling: "text-primary",
  waiting: "text-muted-foreground",
  error: "text-destructive",
};

const expressionLabelByStatus: Record<AgentAnimationStatus, string> = {
  idle: "Idle dot-matrix expression",
  thinking: "Thinking dot-matrix expression",
  running: "Running dot-matrix expression",
  tooling: "Tooling dot-matrix expression",
  waiting: "Waiting dot-matrix expression",
  error: "Error dot-matrix expression",
};

const matrixCellSize = 6;
const inactiveDotRadius = 1.1;
const matrixColumnCount = 22;
const matrixRowCount = 30;
const faceViewBoxWidth = matrixColumnCount * matrixCellSize;
const faceViewBoxHeight = matrixRowCount * matrixCellSize;
const inactiveDots = Array.from(
  { length: matrixColumnCount * matrixRowCount },
  (_, index) => ({
    columnIndex: index % matrixColumnCount,
    rowIndex: Math.floor(index / matrixColumnCount),
  }),
);

export function AgentStatusAnimation({
  status,
  className,
}: AgentStatusAnimationProps) {
  const prefersReducedMotion = usePrefersReducedMotion();
  const shouldBreathe = status === "idle" && !prefersReducedMotion;

  return (
    <div
      data-status={status}
      className={cn(
        "relative flex aspect-[11/15] w-64 items-center justify-center overflow-hidden",
        "rounded-[2rem] border border-border/50 bg-card/70 p-5 shadow-sm",
        "transition-colors duration-500",
        "after:absolute after:inset-x-8 after:bottom-2 after:h-10 after:rounded-full after:bg-primary/10 after:blur-2xl after:content-['']",
        expressionToneByStatus[status],
        shouldBreathe && "motion-safe:animate-pulse",
        className,
      )}
    >
      <svg
        aria-label={expressionLabelByStatus[status]}
        className="relative z-10 h-full w-full"
        role="img"
        viewBox={`0 0 ${faceViewBoxWidth} ${faceViewBoxHeight}`}
      >
        {inactiveDots.map(({ columnIndex, rowIndex }) => (
          <circle
            key={`${rowIndex}-${columnIndex}`}
            cx={columnIndex * matrixCellSize + matrixCellSize / 2}
            cy={rowIndex * matrixCellSize + matrixCellSize / 2}
            fill="currentColor"
            opacity={0.1}
            r={inactiveDotRadius}
          />
        ))}
        <g fill="currentColor">
          <rect height={54} rx={5} width={18} x={34} y={24} />
          <rect height={54} rx={5} width={18} x={80} y={24} />
        </g>
        <path
          d="M 28 108 C 39 140 93 140 104 108"
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={10}
        />
      </svg>
    </div>
  );
}

function usePrefersReducedMotion() {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handleChange = () => setPrefersReducedMotion(mediaQuery.matches);

    handleChange();
    mediaQuery.addEventListener("change", handleChange);

    return () => mediaQuery.removeEventListener("change", handleChange);
  }, []);

  return prefersReducedMotion;
}

