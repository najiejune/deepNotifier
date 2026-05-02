import { cn } from "@/lib/cn";
import { InputHTMLAttributes, forwardRef } from "react";

export const Input = forwardRef<
  HTMLInputElement,
  InputHTMLAttributes<HTMLInputElement>
>(({ className, type = "text", ...props }, ref) => {
  return (
    <input
      ref={ref}
      type={type}
      className={cn(
        "h-8 w-full rounded-sm bg-white border border-border-subtle px-3 text-sm text-text",
        "placeholder:text-text-muted",
        "focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20",
        "transition-colors duration-150",
        "file:border-0 file:bg-transparent file:text-sm file:font-medium",
        className,
      )}
      {...props}
    />
  );
});
Input.displayName = "Input";
