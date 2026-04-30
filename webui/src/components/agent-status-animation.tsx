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

type MatrixDot = {
  columnIndex: number;
  rowIndex: number;
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
const activeDotRadius = 2.35;
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

function createRectangleDots({
  columnStart,
  rowStart,
  columnCount,
  rowCount,
}: {
  columnStart: number;
  rowStart: number;
  columnCount: number;
  rowCount: number;
}): MatrixDot[] {
  return Array.from({ length: columnCount * rowCount }, (_, index) => ({
    columnIndex: columnStart + (index % columnCount),
    rowIndex: rowStart + Math.floor(index / columnCount),
  }));
}

const eyeDots: MatrixDot[] = [
  ...createRectangleDots({
    columnStart: 6,
    rowStart: 4,
    columnCount: 3,
    rowCount: 9,
  }),
  ...createRectangleDots({
    columnStart: 13,
    rowStart: 4,
    columnCount: 3,
    rowCount: 9,
  }),
];

const mouthDots: MatrixDot[] = [
  { columnIndex: 3, rowIndex: 17 },
  { columnIndex: 18, rowIndex: 17 },
  { columnIndex: 3, rowIndex: 18 },
  { columnIndex: 4, rowIndex: 18 },
  { columnIndex: 17, rowIndex: 18 },
  { columnIndex: 18, rowIndex: 18 },
  { columnIndex: 4, rowIndex: 19 },
  { columnIndex: 5, rowIndex: 19 },
  { columnIndex: 16, rowIndex: 19 },
  { columnIndex: 17, rowIndex: 19 },
  { columnIndex: 5, rowIndex: 20 },
  { columnIndex: 6, rowIndex: 20 },
  { columnIndex: 15, rowIndex: 20 },
  { columnIndex: 16, rowIndex: 20 },
  { columnIndex: 6, rowIndex: 21 },
  { columnIndex: 7, rowIndex: 21 },
  { columnIndex: 14, rowIndex: 21 },
  { columnIndex: 15, rowIndex: 21 },
  { columnIndex: 7, rowIndex: 22 },
  { columnIndex: 8, rowIndex: 22 },
  { columnIndex: 9, rowIndex: 22 },
  { columnIndex: 12, rowIndex: 22 },
  { columnIndex: 13, rowIndex: 22 },
  { columnIndex: 14, rowIndex: 22 },
  { columnIndex: 9, rowIndex: 23 },
  { columnIndex: 10, rowIndex: 23 },
  { columnIndex: 11, rowIndex: 23 },
  { columnIndex: 12, rowIndex: 23 },
];

const activeDots: MatrixDot[] = [...eyeDots, ...mouthDots];

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
        {activeDots.map(({ columnIndex, rowIndex }) => (
          <circle
            key={`active-${rowIndex}-${columnIndex}`}
            cx={columnIndex * matrixCellSize + matrixCellSize / 2}
            cy={rowIndex * matrixCellSize + matrixCellSize / 2}
            fill="currentColor"
            r={activeDotRadius}
          />
        ))}
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

