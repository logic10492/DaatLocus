import { useEffect, useLayoutEffect, useRef, useState } from "react";

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

const faceViewBoxWidth = 132;
const faceViewBoxHeight = 180;

const expressionLabelByStatus: Record<AgentAnimationStatus, string> = {
  idle: "Idle smooth expression",
  thinking: "Working smooth expression",
  running: "Working smooth expression",
  tooling: "Working smooth expression",
  waiting: "Waiting smooth expression",
  error: "Error smooth expression",
};

const expressionMotionByStatus: Record<AgentAnimationStatus, string> = {
  idle: "opacity-100",
  thinking: "scale-[1.015]",
  running: "scale-[1.025]",
  tooling: "scale-[1.025]",
  waiting: "opacity-80",
  error: "scale-[1.025]",
};

const idleMouthPath =
  "M 22 106 C 32 126 46 139 66 139 C 86 139 100 126 110 106";
const workingMouthPath =
  "M 36 121 C 48 121 54 121 66 121 C 78 121 84 121 96 121";
const expressionTransitionDurationMs = 420;
const expressionTransitionDuration = `${expressionTransitionDurationMs}ms`;
const expressionTransitionKeyTimes = "0;0.52;1";
const expressionTransitionKeySplines = "0.2 0 0 1;0.2 0 0 1";
const mouthPathByVisualKind = {
  idle: idleMouthPath,
  working: workingMouthPath,
} as const;
const eyeTransitionFrames = {
  left: {
    height: "41;12;41",
    rx: "8.5;6;8.5",
    width: "17;27;17",
    x: "37;32;37",
    y: "31;45.5;31",
  },
  right: {
    height: "41;12;41",
    rx: "8.5;6;8.5",
    width: "17;27;17",
    x: "78;73;78",
    y: "31;45.5;31",
  },
} as const;
const workingEyeDuration = "2.2s";
const workingEyeKeyTimes = "0;0.12;0.38;0.5;0.62;0.88;1";
const leftWorkingEyeFrames = {
  height: "41;9;9;41;41;41;41",
  rx: "8.5;4.5;4.5;8.5;8.5;8.5;8.5",
  width: "17;26;26;17;17;17;17",
  x: "37;32.5;32.5;37;37;37;37",
  y: "31;47;47;31;31;31;31",
} as const;
const rightWorkingEyeFrames = {
  height: "41;41;41;41;9;9;41",
  rx: "8.5;8.5;8.5;8.5;4.5;4.5;8.5",
  width: "17;17;17;17;26;26;17",
  x: "78;78;78;78;73.5;73.5;78",
  y: "31;31;31;31;47;47;31",
} as const;

type ExpressionVisualKind = keyof typeof mouthPathByVisualKind;

type ExpressionTransition = {
  from: ExpressionVisualKind;
  id: number;
  to: ExpressionVisualKind;
};

function isWorkingStatus(status: AgentAnimationStatus) {
  return status === "thinking" || status === "running" || status === "tooling";
}

export function AgentStatusAnimation({
  status,
  className,
}: AgentStatusAnimationProps) {
  const prefersReducedMotion = usePrefersReducedMotion();
  const isWorking = isWorkingStatus(status);
  const visualKind = isWorking ? "working" : "idle";
  const expressionTransition = useExpressionTransition(
    visualKind,
    prefersReducedMotion,
  );
  const shouldBreathe = status === "idle" && !prefersReducedMotion;
  const shouldAnimateWorking =
    isWorking && !prefersReducedMotion && expressionTransition === null;
  const mouthPath = mouthPathByVisualKind[visualKind];

  return (
    <div
      data-animation-kind={isWorking ? "working" : status}
      data-status={status}
      className={cn(
        "relative flex aspect-[11/15] w-64 items-center justify-center overflow-hidden",
        "rounded-[2rem] border border-border/50 bg-card/70 p-5 shadow-sm",
        "transition-colors duration-500",
        "after:absolute after:inset-x-8 after:bottom-2 after:h-10 after:rounded-full after:bg-primary/10 after:blur-2xl after:content-['']",
        isWorking && "border-primary/25 bg-primary/[0.03] shadow-primary/10",
        shouldBreathe && "motion-safe:animate-pulse",
        className,
      )}
    >
      <svg
        aria-label={expressionLabelByStatus[status]}
        className={cn(
          "relative z-10 h-full w-full origin-center transition duration-500",
          !prefersReducedMotion && expressionMotionByStatus[status],
        )}
        role="img"
        viewBox={`0 0 ${faceViewBoxWidth} ${faceViewBoxHeight}`}
      >
        <g fill="black">
          <rect height="41" rx="8.5" width="17" x="37" y="31">
            {expressionTransition && (
              <>
                <animate
                  key={`left-eye-height-transition-${expressionTransition.id}`}
                  attributeName="height"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.left.height}
                />
                <animate
                  key={`left-eye-rx-transition-${expressionTransition.id}`}
                  attributeName="rx"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.left.rx}
                />
                <animate
                  key={`left-eye-width-transition-${expressionTransition.id}`}
                  attributeName="width"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.left.width}
                />
                <animate
                  key={`left-eye-x-transition-${expressionTransition.id}`}
                  attributeName="x"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.left.x}
                />
                <animate
                  key={`left-eye-y-transition-${expressionTransition.id}`}
                  attributeName="y"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.left.y}
                />
              </>
            )}
            {shouldAnimateWorking && (
              <>
                <animate
                  attributeName="height"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftWorkingEyeFrames.height}
                />
                <animate
                  attributeName="rx"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftWorkingEyeFrames.rx}
                />
                <animate
                  attributeName="width"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftWorkingEyeFrames.width}
                />
                <animate
                  attributeName="x"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftWorkingEyeFrames.x}
                />
                <animate
                  attributeName="y"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftWorkingEyeFrames.y}
                />
              </>
            )}
          </rect>
          <rect height="41" rx="8.5" width="17" x="78" y="31">
            {expressionTransition && (
              <>
                <animate
                  key={`right-eye-height-transition-${expressionTransition.id}`}
                  attributeName="height"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.right.height}
                />
                <animate
                  key={`right-eye-rx-transition-${expressionTransition.id}`}
                  attributeName="rx"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.right.rx}
                />
                <animate
                  key={`right-eye-width-transition-${expressionTransition.id}`}
                  attributeName="width"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.right.width}
                />
                <animate
                  key={`right-eye-x-transition-${expressionTransition.id}`}
                  attributeName="x"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.right.x}
                />
                <animate
                  key={`right-eye-y-transition-${expressionTransition.id}`}
                  attributeName="y"
                  begin="0s"
                  calcMode="spline"
                  dur={expressionTransitionDuration}
                  fill="freeze"
                  keySplines={expressionTransitionKeySplines}
                  keyTimes={expressionTransitionKeyTimes}
                  values={eyeTransitionFrames.right.y}
                />
              </>
            )}
            {shouldAnimateWorking && (
              <>
                <animate
                  attributeName="height"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightWorkingEyeFrames.height}
                />
                <animate
                  attributeName="rx"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightWorkingEyeFrames.rx}
                />
                <animate
                  attributeName="width"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightWorkingEyeFrames.width}
                />
                <animate
                  attributeName="x"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightWorkingEyeFrames.x}
                />
                <animate
                  attributeName="y"
                  dur={workingEyeDuration}
                  keyTimes={workingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightWorkingEyeFrames.y}
                />
              </>
            )}
          </rect>
        </g>
        <path
          d={mouthPath}
          fill="none"
          stroke="black"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="14"
        >
          {expressionTransition && (
            <animate
              key={`mouth-transition-${expressionTransition.id}`}
              attributeName="d"
              begin="0s"
              calcMode="spline"
              dur={expressionTransitionDuration}
              fill="freeze"
              from={mouthPathByVisualKind[expressionTransition.from]}
              keySplines="0.2 0 0 1"
              keyTimes="0;1"
              to={mouthPathByVisualKind[expressionTransition.to]}
            />
          )}
        </path>
      </svg>
    </div>
  );
}

function useExpressionTransition(
  visualKind: ExpressionVisualKind,
  prefersReducedMotion: boolean,
) {
  const [transition, setTransition] = useState<ExpressionTransition | null>(
    null,
  );
  const previousVisualKindRef = useRef<ExpressionVisualKind>(visualKind);
  const transitionIdRef = useRef(0);

  useLayoutEffect(() => {
    if (prefersReducedMotion) {
      previousVisualKindRef.current = visualKind;
      setTransition(null);
      return;
    }

    const previousVisualKind = previousVisualKindRef.current;

    if (previousVisualKind === visualKind) {
      return;
    }

    previousVisualKindRef.current = visualKind;

    const nextTransition = {
      from: previousVisualKind,
      id: (transitionIdRef.current += 1),
      to: visualKind,
    };

    setTransition(nextTransition);

    const timeout = window.setTimeout(() => {
      setTransition((currentTransition) =>
        currentTransition?.id === nextTransition.id ? null : currentTransition,
      );
    }, expressionTransitionDurationMs);

    return () => window.clearTimeout(timeout);
  }, [prefersReducedMotion, visualKind]);

  return transition;
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

