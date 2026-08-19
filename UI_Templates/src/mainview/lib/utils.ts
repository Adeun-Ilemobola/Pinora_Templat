import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatSeconds(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return "0s"
  }

  const roundedSeconds = Math.ceil(seconds)

  if (roundedSeconds < 60) {
    return `${roundedSeconds}s`
  }

  return `${Math.ceil(roundedSeconds / 60)}m`
}
