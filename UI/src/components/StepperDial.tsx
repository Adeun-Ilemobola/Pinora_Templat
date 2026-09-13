import { useId, useRef, type PointerEvent } from "react";

export const normalizeAngle = (angle: number) => ((angle % 360) + 360) % 360;
export const angularDelta = (from: number, to: number) =>
  ((to - from + 540) % 360) - 180;

interface StepperDialProps {
  reportedAngle: number | null;
  steps: number | null;
  target: number | null;
  disabled?: boolean;
  onTargetChange: (angle: number | null) => void;
  onCommit: (angle: number) => void;
}

/** Absolute multi-turn position: ring geometry wraps, while the target retains turns. */
export function StepperDial({
  reportedAngle,
  steps,
  target,
  disabled = false,
  onTargetChange,
  onCommit,
}: StepperDialProps) {
  const hintId = useId();
  const drag = useRef<{
    id: number;
    bearing: number;
    total: number;
    previous: number | null;
  } | null>(null);
  const bearing = (event: PointerEvent<HTMLDivElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    const x = event.clientX - rect.left - rect.width / 2;
    const y = event.clientY - rect.top - rect.height / 2;
    if (Math.hypot(x, y) < rect.width * 0.22) return null;
    return normalizeAngle((Math.atan2(x, -y) * 180) / Math.PI);
  };
  const move = (event: PointerEvent<HTMLDivElement>) => {
    const active = drag.current;
    if (disabled || !active || active.id !== event.pointerId) return;
    const next = bearing(event);
    if (next === null) return;
    active.total += angularDelta(active.bearing, next);
    active.bearing = next;
    onTargetChange(Math.round(active.total * 10) / 10);
  };
  const cancel = () => {
    if (drag.current) onTargetChange(drag.current.previous);
    drag.current = null;
  };
  const point = (angle: number, radius: number) => ({
    x: 120 + Math.sin((angle * Math.PI) / 180) * radius,
    y: 120 - Math.cos((angle * Math.PI) / 180) * radius,
  });
  const shown = target ?? reportedAngle;
  return (
    <div className="space-y-3">
      <div
        role="spinbutton"
        aria-label="Stepper target angle"
        aria-describedby={hintId}
        aria-disabled={disabled}
        aria-valuenow={shown ?? undefined}
        aria-valuetext={
          shown === null
            ? "Position not reported"
            : `${shown.toFixed(1)} degrees, ${target === null ? "reported position" : "target preview"}`
        }
        tabIndex={disabled ? -1 : 0}
        className={`relative mx-auto aspect-square w-full max-w-60 touch-none select-none rounded-full outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4 focus-visible:ring-offset-card ${disabled ? "opacity-60" : "cursor-grab active:cursor-grabbing"}`}
        onPointerDown={(event) => {
          if (
            disabled ||
            !event.isPrimary ||
            event.button !== 0 ||
            drag.current
          )
            return;
          const next = bearing(event);
          if (next === null) return;
          event.preventDefault();
          event.currentTarget.focus();
          event.currentTarget.setPointerCapture(event.pointerId);
          const base = target ?? reportedAngle ?? 0;
          const total = base + angularDelta(normalizeAngle(base), next);
          drag.current = {
            id: event.pointerId,
            bearing: next,
            total,
            previous: target,
          };
          onTargetChange(Math.round(total * 10) / 10);
        }}
        onPointerMove={move}
        onPointerUp={(event) => {
          if (!drag.current || drag.current.id !== event.pointerId) return;
          move(event);
          const total = Math.round(drag.current.total * 10) / 10;
          drag.current = null;
          event.currentTarget.releasePointerCapture(event.pointerId);
          if (!disabled) onCommit(total);
        }}
        onPointerCancel={cancel}
        onLostPointerCapture={cancel}
        onKeyDown={(event) => {
          if (disabled) return;
          if (
            [
              "ArrowUp",
              "ArrowRight",
              "ArrowDown",
              "ArrowLeft",
              "Home",
              "Enter",
              "Escape",
            ].includes(event.key)
          )
            event.preventDefault();
          if (event.key === "Escape") {
            cancel();
            onTargetChange(null);
            return;
          }
          if (event.key === "Enter") {
            if (target !== null) onCommit(target);
            return;
          }
          if (event.key === "Home") {
            onTargetChange(0);
            return;
          }
          const direction = ["ArrowUp", "ArrowRight"].includes(event.key)
            ? 1
            : ["ArrowDown", "ArrowLeft"].includes(event.key)
              ? -1
              : 0;
          if (direction)
            onTargetChange(
              (target ?? reportedAngle ?? 0) +
                direction * (event.shiftKey ? 10 : 1),
            );
        }}
      >
        <svg viewBox="0 0 240 240" className="size-full" aria-hidden="true">
          <circle
            cx="120"
            cy="120"
            r="88"
            fill="none"
            stroke="currentColor"
            strokeWidth="12"
            className="text-muted"
          />
          {Array.from({ length: 36 }, (_, index) => {
            const a = point(index * 10, 101);
            const b = point(index * 10, index % 9 === 0 ? 112 : 106);
            return (
              <line
                key={index}
                x1={a.x}
                y1={a.y}
                x2={b.x}
                y2={b.y}
                stroke="currentColor"
                className={index % 9 === 0 ? "text-foreground" : "text-border"}
                strokeWidth={index % 9 === 0 ? 2 : 1}
              />
            );
          })}
          <text
            x="120"
            y="10"
            textAnchor="middle"
            fill="currentColor"
            className="fill-muted-foreground text-[9px]"
          >
            0°
          </text>
          {reportedAngle !== null && (
            <g transform={`rotate(${normalizeAngle(reportedAngle)} 120 120)`}>
              <line
                x1="120"
                y1="47"
                x2="120"
                y2="23"
                stroke="currentColor"
                strokeWidth="3"
                className="text-primary"
              />
              <circle
                cx="120"
                cy="32"
                r="6"
                fill="currentColor"
                className="text-primary"
              />
            </g>
          )}
          {target !== null && (
            <circle
              cx={point(normalizeAngle(target), 88).x}
              cy={point(normalizeAngle(target), 88).y}
              r="8"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeDasharray="3 2"
              className="text-foreground"
            />
          )}
        </svg>
        <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-1 px-14 text-center">
          <span className="text-[10px] uppercase tracking-widest text-muted-foreground">
            Reported position
          </span>
          <span className="font-heading text-3xl font-semibold tabular-nums">
            {reportedAngle === null
              ? "—"
              : `${normalizeAngle(reportedAngle).toFixed(1)}°`}
          </span>
          <span className="mt-1 max-w-full break-all font-mono text-sm tabular-nums">
            {steps === null
              ? "—"
              : steps.toLocaleString(undefined, { maximumFractionDigits: 2 })}
          </span>
          <span className="text-[10px] uppercase tracking-widest text-muted-foreground">
            Steps
          </span>
        </div>
      </div>
      <p
        id={hintId}
        className="text-center text-xs leading-relaxed text-muted-foreground"
      >
        Drag the ring; release to move. Arrow keys adjust 1° (Shift: 10°); Enter
        moves, Escape cancels. Home previews 0°.
      </p>
      <p className="text-center text-xs text-muted-foreground">
        Solid marker: reported · Dashed marker: target
      </p>
    </div>
  );
}
