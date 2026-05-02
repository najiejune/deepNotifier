import { cn } from "@/lib/cn";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  size?: "sm" | "md";
}

export function Toggle({
  checked,
  onChange,
  disabled,
  size = "md",
}: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative inline-flex shrink-0 cursor-pointer rounded-full transition-all duration-200",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/30",
        "disabled:opacity-40 disabled:cursor-not-allowed",
        size === "sm" ? "h-4 w-8" : "h-5 w-9",
        checked ? "bg-accent" : "bg-border-strong",
      )}
    >
      <span
        className={cn(
          "pointer-events-none block rounded-full bg-white shadow-sm transition-all duration-200",
          size === "sm" ? "h-3 w-3" : "h-4 w-4",
          checked
            ? size === "sm"
              ? "translate-x-[17px] mt-[2px] ml-[2px]"
              : "translate-x-[18px] mt-[2px] ml-[2px]"
            : "translate-x-[2px] mt-[2px]",
        )}
      />
    </button>
  );
}
