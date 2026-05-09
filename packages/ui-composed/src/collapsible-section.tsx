"use client";

/**
 * CollapsibleSection — Animated expand/collapse section.
 *
 * A section with a header that can be expanded or collapsed.
 * Includes smooth animation, optional badge, and icon rotation.
 *
 * @module components/composed/collapsible-section
 */

import * as React from "react";
import { ChevronDown } from "lucide-react";
import { cn } from "@modrinth/ui";

// ============================================================================
// Types
// ============================================================================

export interface CollapsibleSectionProps {
  /** Section title */
  title: React.ReactNode;
  /** Section content */
  children: React.ReactNode;
  /** Whether section is expanded by default */
  defaultOpen?: boolean;
  /** Controlled open state */
  open?: boolean;
  /** Called when open state changes */
  onOpenChange?: (open: boolean) => void;
  /** Optional badge/count displayed next to title */
  badge?: React.ReactNode;
  /** Additional CSS classes for the container */
  className?: string;
  /** Additional CSS classes for the header */
  headerClassName?: string;
  /** Additional CSS classes for the content */
  contentClassName?: string;
}

// ============================================================================
// Component
// ============================================================================

export function CollapsibleSection({
  title,
  children,
  defaultOpen = false,
  open: controlledOpen,
  onOpenChange,
  badge,
  className,
  headerClassName,
  contentClassName,
}: CollapsibleSectionProps) {
  const [internalOpen, setInternalOpen] = React.useState(defaultOpen);
  const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen;

  const toggle = React.useCallback(() => {
    const next = !isOpen;
    if (controlledOpen === undefined) {
      setInternalOpen(next);
    }
    onOpenChange?.(next);
  }, [isOpen, controlledOpen, onOpenChange]);

  const contentRef = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState<number | undefined>(isOpen ? undefined : 0);

  React.useEffect(() => {
    if (contentRef.current) {
      setHeight(isOpen ? contentRef.current.scrollHeight : 0);
    }
  }, [isOpen]);

  return (
    <div className={cn("rounded-lg border bg-background", className)}>
      <button
        type="button"
        onClick={toggle}
        className={cn(
          "flex w-full items-center justify-between gap-2 px-4 py-3 text-left font-medium hover:bg-muted/50 transition-colors",
          headerClassName,
        )}
      >
        <span className="flex items-center gap-2">
          {title}
          {badge && (
            <span className="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-xs font-medium">
              {badge}
            </span>
          )}
        </span>
        <ChevronDown
          className={cn(
            "h-4 w-4 text-muted-foreground shrink-0 transition-transform duration-200",
            isOpen && "rotate-180",
          )}
        />
      </button>

      <div
        className="overflow-hidden transition-[height] duration-200 ease-in-out"
        style={{ height }}
      >
        <div
          ref={contentRef}
          className={cn("px-4 pb-4", contentClassName)}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
