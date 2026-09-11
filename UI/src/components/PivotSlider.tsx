import { useId } from "react";
import * as SliderPrimitive from "@radix-ui/react-slider";
import { cn } from "cn";
import { Label } from "@/components/ui/label";

export interface PivotSliderProps {
  value: number;
  onValueChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  pivot?: number;
  disabled?: boolean;
  className?: string;
  label?: string;
  showValue?: boolean;
  formatValue?: (value: number) => string;
}

/** Controlled, single-value slider with a neutral point independent of the range midpoint. */
export function PivotSlider({
  value,
  onValueChange,
  min = -100,
  max = 100,
  step = 1,
  pivot = 0,
  disabled = false,
  className,
  label = "Value",
  showValue = true,
  formatValue = String,
}: PivotSliderProps) {
  const id = useId();
  const span = max - min;
  if (![min, max, step, pivot, value, span].every(Number.isFinite) || span <= 0 || step <= 0) {
    throw new RangeError("PivotSlider requires finite numbers, max > min, and step > 0.");
  }

  const clamp = (number: number) => Math.min(max, Math.max(min, number));
  const currentValue = clamp(value);
  // Both positions use the supplied bounds; the pivot need not be zero or centered.
  const pivotPercent = ((clamp(pivot) - min) / span) * 100;
  const valuePercent = ((currentValue - min) / span) * 100;
  const start = Math.min(pivotPercent, valuePercent);
  const width = Math.abs(valuePercent - pivotPercent);
  const formattedValue = formatValue(currentValue);

  return (
    <div className={cn("space-y-2 text-foreground", disabled && "opacity-50", className)}>
      <div className="flex items-center justify-between gap-4">
        <Label id={`${id}-label`}>{label}</Label>
        {showValue && <span className="text-sm tabular-nums text-muted-foreground">{formattedValue}</span>}
      </div>
      <SliderPrimitive.Root
        data-slot="pivot-slider"
        className="relative flex h-8 w-full touch-none select-none items-center data-disabled:cursor-not-allowed"
        dir="ltr"
        min={min}
        max={max}
        step={step}
        value={[currentValue]}
        onValueChange={([next]) => onValueChange(next)}
        disabled={disabled}
      >
        {/* Half-thumb margins align the track endpoints with Radix's inset thumb centers. */}
        <SliderPrimitive.Track data-slot="pivot-slider-track" className="relative mx-3 h-2 grow rounded-full bg-input/90">
          {/* Radix Range starts at min; this segment instead spans pivot to value. */}
          <span
            data-slot="pivot-slider-range"
            aria-hidden="true"
            className="pointer-events-none absolute h-full rounded-full bg-primary"
            style={{ left: `${start}%`, width: `${width}%` }}
          />
          <span
            data-slot="pivot-slider-marker"
            aria-hidden="true"
            className="pointer-events-none absolute top-1/2 h-6 w-px -translate-x-1/2 -translate-y-1/2 bg-muted-foreground/70"
            style={{ left: `${pivotPercent}%` }}
          />
        </SliderPrimitive.Track>
        <SliderPrimitive.Thumb
          aria-labelledby={`${id}-label`}
          aria-valuetext={formattedValue}
          className="block h-4 w-6 rounded-full bg-card shadow-md ring-1 ring-foreground/20 transition-[color,box-shadow] hover:ring-4 hover:ring-ring/30 focus-visible:outline-hidden focus-visible:ring-4 focus-visible:ring-ring/30 data-disabled:pointer-events-none"
        />
      </SliderPrimitive.Root>
    </div>
  );
}
