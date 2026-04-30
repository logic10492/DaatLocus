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

const faceViewBoxWidth = 132;
const faceViewBoxHeight = 180;

const expressionLabelByStatus: Record<AgentAnimationStatus, string> = {
  idle: "Idle smooth expression",
  thinking: "Thinking smooth expression",
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
const thinkingMouthPath = "M 36 121 L 96 121";
const workingMouthPath =
  "M 24 115 C 36 126 49 131 66 131 C 83 131 96 126 108 115";
const workingMouthFrames = [
  workingMouthPath,
  "M 24 118 C 38 109 51 109 66 118 C 81 127 94 127 108 118",
  "M 24 113 C 37 124 50 128 66 128 C 82 128 95 124 108 113",
  workingMouthPath,
].join(";");
const thinkingEyeDuration = "2.2s";
const thinkingEyeKeyTimes = "0;0.12;0.38;0.5;0.62;0.88;1";
const leftThinkingEyeFrames = {
  height: "41;9;9;41;41;41;41",
  rx: "8.5;4.5;4.5;8.5;8.5;8.5;8.5",
  width: "17;26;26;17;17;17;17",
  x: "37;32.5;32.5;37;37;37;37",
  y: "31;47;47;31;31;31;31",
} as const;
const rightThinkingEyeFrames = {
  height: "41;41;41;41;9;9;41",
  rx: "8.5;8.5;8.5;8.5;4.5;4.5;8.5",
  width: "17;17;17;17;26;26;17",
  x: "78;78;78;78;73.5;73.5;78",
  y: "31;31;31;31;47;47;31",
} as const;

function isWorkingStatus(status: AgentAnimationStatus) {
  return status === "running" || status === "tooling";
}

export function AgentStatusAnimation({
  status,
  className,
}: AgentStatusAnimationProps) {
  const prefersReducedMotion = usePrefersReducedMotion();
  const isThinking = status === "thinking";
  const isWorking = isWorkingStatus(status);
  const shouldBreathe = status === "idle" && !prefersReducedMotion;
  const shouldAnimateThinking = isThinking && !prefersReducedMotion;
  const shouldAnimateWorking = isWorking && !prefersReducedMotion;

  return (
    <div
      data-animation-kind={isThinking ? "thinking" : isWorking ? "working" : status}
      data-status={status}
      className={cn(
        "relative flex aspect-[11/15] w-64 items-center justify-center overflow-hidden",
        "rounded-[2rem] border border-border/50 bg-card/70 p-5 shadow-sm",
        "transition-colors duration-500",
        "after:absolute after:inset-x-8 after:bottom-2 after:h-10 after:rounded-full after:bg-primary/10 after:blur-2xl after:content-['']",
        (isThinking || isWorking) &&
          "border-primary/25 bg-primary/[0.03] shadow-primary/10",
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
            {shouldAnimateThinking && (
              <>
                <animate
                  attributeName="height"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftThinkingEyeFrames.height}
                />
                <animate
                  attributeName="rx"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftThinkingEyeFrames.rx}
                />
                <animate
                  attributeName="width"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftThinkingEyeFrames.width}
                />
                <animate
                  attributeName="x"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftThinkingEyeFrames.x}
                />
                <animate
                  attributeName="y"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={leftThinkingEyeFrames.y}
                />
              </>
            )}
            {shouldAnimateWorking && (
              <>
                <animate
                  attributeName="height"
                  dur="1.4s"
                  repeatCount="indefinite"
                  values="41;46;41;36;41"
                />
                <animate
                  attributeName="y"
                  dur="1.4s"
                  repeatCount="indefinite"
                  values="31;27;31;36;31"
                />
              </>
            )}
          </rect>
          <rect height="41" rx="8.5" width="17" x="78" y="31">
            {shouldAnimateThinking && (
              <>
                <animate
                  attributeName="height"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightThinkingEyeFrames.height}
                />
                <animate
                  attributeName="rx"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightThinkingEyeFrames.rx}
                />
                <animate
                  attributeName="width"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightThinkingEyeFrames.width}
                />
                <animate
                  attributeName="x"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightThinkingEyeFrames.x}
                />
                <animate
                  attributeName="y"
                  dur={thinkingEyeDuration}
                  keyTimes={thinkingEyeKeyTimes}
                  repeatCount="indefinite"
                  values={rightThinkingEyeFrames.y}
                />
              </>
            )}
            {shouldAnimateWorking && (
              <>
                <animate
                  attributeName="height"
                  begin="-0.35s"
                  dur="1.4s"
                  repeatCount="indefinite"
                  values="41;36;41;46;41"
                />
                <animate
                  attributeName="y"
                  begin="-0.35s"
                  dur="1.4s"
                  repeatCount="indefinite"
                  values="31;36;31;27;31"
                />
              </>
            )}
          </rect>
        </g>
        <path
          d={
            isThinking
              ? thinkingMouthPath
              : isWorking
                ? workingMouthPath
                : idleMouthPath
          }
          fill="none"
          stroke="black"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="14"
        >
          {shouldAnimateWorking && (
            <animate
              attributeName="d"
              dur="1.4s"
              repeatCount="indefinite"
              values={workingMouthFrames}
            />
          )}
        </path>
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

