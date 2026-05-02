import { cn } from "@/lib/cn";
import { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
}

export function Button({
  variant = "secondary",
  size = "md",
  className,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-2 font-medium transition-all duration-150",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/30",
        "disabled:opacity-40 disabled:pointer-events-none",
        size === "sm" && "h-7 px-3 text-xs rounded-sm",
        size === "md" && "h-8 px-4 text-sm rounded-sm",
        size === "lg" && "h-10 px-5 text-sm rounded-md",
        variant === "primary" &&
          "bg-accent text-white hover:bg-[#0065ff] active:scale-[0.98] shadow-sm",
        variant === "secondary" &&
          "bg-white text-text-secondary hover:text-text border border-border-subtle hover:border-border-default hover:bg-bg-layer shadow-sm",
        variant === "ghost" &&
          "text-text-secondary hover:text-text hover:bg-bg-overlay",
        variant === "danger" &&
          "bg-red/10 text-red hover:bg-red/15 border border-red/15",
        className,
      )}
      {...props}
    />
  );
}
