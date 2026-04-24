import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

interface ShimmerProps {
  className?: string;
  width?: string | number;
  height?: string | number;
  children?: ReactNode;
}

export function Shimmer({ className, width, height, children }: ShimmerProps) {
  if (children) {
    return (
      <span
        className={cn(
          "relative inline-block overflow-hidden rounded",
          "after:absolute after:inset-0 after:animate-pulse after:rounded after:bg-aurora-surface-raised/60",
          className,
        )}
        aria-hidden="true"
      >
        <span className="invisible">{children}</span>
      </span>
    );
  }

  return (
    <span
      className={cn(
        "inline-block animate-pulse rounded bg-aurora-surface-raised",
        className,
      )}
      style={{ width, height: height ?? "1em", display: "inline-block" }}
    />
  );
}
