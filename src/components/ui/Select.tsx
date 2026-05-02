import { cn } from "@/lib/cn";
import { SelectHTMLAttributes } from "react";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  options: { value: string; label: string }[];
}

export function Select({ options, className, ...props }: SelectProps) {
  return (
    <select
      className={cn(
        "h-8 rounded-sm bg-white border border-border-subtle px-3 text-sm text-text",
        "focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20",
        "transition-colors duration-150 appearance-none cursor-pointer",
        className,
      )}
      {...props}
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}
