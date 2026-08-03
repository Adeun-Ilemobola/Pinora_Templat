import { mergeProps } from "@base-ui/react/merge-props"
import { useRender } from "@base-ui/react/use-render"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "group/badge inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent font-medium whitespace-nowrap transition-[color,background-color,border-color,box-shadow] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground [a]:hover:bg-primary/80",
        secondary:
          "bg-secondary text-secondary-foreground [a]:hover:bg-secondary/80",
        destructive:
          "bg-destructive/10 text-destructive focus-visible:ring-destructive/20 dark:bg-destructive/20 dark:focus-visible:ring-destructive/40 [a]:hover:bg-destructive/20",
        outline:
          "border-border bg-input/30 text-foreground [a]:hover:bg-muted [a]:hover:text-muted-foreground",
        ghost:
          "hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50",
        link: "text-primary underline-offset-4 hover:underline",

        // Log priority variants intentionally become warmer as urgency increases.
        low:
          "border-emerald-600/20 bg-emerald-500/10 text-emerald-700 shadow-[inset_0_0_0_1px_color-mix(in_oklab,currentColor_5%,transparent)] dark:border-emerald-400/20 dark:bg-emerald-400/10 dark:text-emerald-300",
        medium:
          "border-amber-600/20 bg-amber-500/12 text-amber-800 shadow-[inset_0_0_0_1px_color-mix(in_oklab,currentColor_5%,transparent)] dark:border-amber-400/20 dark:bg-amber-400/10 dark:text-amber-300",
        high:
          "border-orange-600/20 bg-orange-500/12 text-orange-800 shadow-[inset_0_0_0_1px_color-mix(in_oklab,currentColor_5%,transparent)] dark:border-orange-400/20 dark:bg-orange-400/10 dark:text-orange-300",
        critical:
          "border-rose-600/25 bg-rose-500/12 text-rose-700 shadow-[inset_0_0_0_1px_color-mix(in_oklab,currentColor_8%,transparent)] dark:border-rose-400/25 dark:bg-rose-400/12 dark:text-rose-300",

        // Incoming event variants are categorical rather than urgency-based.
        registration:
          "border-violet-600/20 bg-violet-500/10 text-violet-700 dark:border-violet-400/20 dark:bg-violet-400/10 dark:text-violet-300",
        "module-event":
          "border-sky-600/20 bg-sky-500/10 text-sky-700 dark:border-sky-400/20 dark:bg-sky-400/10 dark:text-sky-300",
        system:
          "border-slate-500/20 bg-slate-500/10 text-slate-700 dark:border-slate-300/20 dark:bg-slate-300/10 dark:text-slate-300",
      },
      size: {
        xs: "h-4 gap-0.5 px-1.5 text-[0.5625rem] [&>svg]:size-2.5",
        sm: "h-4.5 gap-0.5 px-1.5 text-[0.625rem] [&>svg]:size-2.5",
        default:
          "h-5 px-2 text-xs [&>svg]:size-3",
        lg: "h-6 gap-1.5 px-2.5 text-sm [&>svg]:size-3.5",
        xl: "h-7 gap-1.5 px-3 text-sm [&>svg]:size-4",
      },
      iconOnly: {
        true: "px-0",
        false: "",
      },
    },
    compoundVariants: [
      { size: "xs", iconOnly: true, className: "w-4" },
      { size: "sm", iconOnly: true, className: "w-4.5" },
      { size: "default", iconOnly: true, className: "w-5" },
      { size: "lg", iconOnly: true, className: "w-6" },
      { size: "xl", iconOnly: true, className: "w-7" },
    ],
    defaultVariants: {
      variant: "default",
      size: "default",
      iconOnly: false,
    },
  },
)

type BadgeProps = useRender.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants>

function Badge({
  className,
  variant = "default",
  size = "default",
  iconOnly = false,
  render,
  ...props
}: BadgeProps) {
  return useRender({
    defaultTagName: "span",
    props: mergeProps<"span">(
      {
        className: cn(badgeVariants({ variant, size, iconOnly }), className),
      },
      props,
    ),
    render,
    state: {
      slot: "badge",
      variant,
      size,
    },
  })
}

export { Badge, badgeVariants }
export type { BadgeProps }
