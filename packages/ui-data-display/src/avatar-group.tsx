"use client";

/**
 * AvatarGroup — Overlapping avatar stack with overflow counter.
 *
 * Displays a horizontal stack of avatars that overlap slightly.
 * When the number exceeds the limit, shows a +N counter badge.
 *
 * @module components/data-display/avatar-group
 */

import * as React from "react";
import { cn } from "@furistic/sdk-fe-ui-core/lib/utils";

// ============================================================================
// Types
// ============================================================================

export interface AvatarGroupItem {
  /** Image URL */
  src?: string;
  /** Fallback initials or label */
  fallback: string;
  /** Tooltip title */
  title?: string;
}

export interface AvatarGroupProps {
  /** Avatars to display */
  items: AvatarGroupItem[];
  /** Maximum to show before overflow */
  limit?: number;
  /** Avatar size */
  size?: "sm" | "md" | "lg";
  /** Overlap amount */
  overlap?: "sm" | "md" | "lg";
  /** Show ring around avatars */
  ring?: boolean;
  /** Called on overflow click */
  onOverflowClick?: () => void;
  /** Called on avatar click */
  onAvatarClick?: (item: AvatarGroupItem, index: number) => void;
  /** Additional CSS classes */
  className?: string;
}

// ============================================================================
// Component
// ============================================================================

export function AvatarGroup({
  items,
  limit = 4,
  size = "md",
  overlap = "md",
  ring = true,
  onOverflowClick,
  onAvatarClick,
  className,
}: AvatarGroupProps) {
  const visible = items.slice(0, limit);
  const overflow = Math.max(0, items.length - limit);

  const sizeClasses = {
    sm: "h-6 w-6 text-[10px]",
    md: "h-8 w-8 text-xs",
    lg: "h-10 w-10 text-sm",
  };

  const overlapClasses = {
    sm: "-ml-1",
    md: "-ml-2",
    lg: "-ml-3",
  };

  return (
    <div className={cn("flex items-center", className)}>
      <div className="flex items-center">
        {visible.map((item, index) => (
          <button
            key={index}
            type="button"
            onClick={() => onAvatarClick?.(item, index)}
            title={item.title ?? item.fallback}
            className={cn(
              "relative inline-flex items-center justify-center rounded-full bg-muted font-medium text-muted-foreground",
              sizeClasses[size],
              index > 0 && overlapClasses[overlap],
              ring && "ring-2 ring-background",
              onAvatarClick && "cursor-pointer hover:ring-primary/30",
            )}
          >
            {item.src ? (
              <img
                src={item.src}
                alt={item.fallback}
                className="h-full w-full rounded-full object-cover"
              />
            ) : (
              <span>{item.fallback.slice(0, 2).toUpperCase()}</span>
            )}
          </button>
        ))}

        {overflow > 0 && (
          <button
            type="button"
            onClick={onOverflowClick}
            className={cn(
              "relative inline-flex items-center justify-center rounded-full bg-muted font-medium text-muted-foreground",
              sizeClasses[size],
              overlapClasses[overlap],
              ring && "ring-2 ring-background",
              onOverflowClick && "cursor-pointer hover:bg-accent",
            )}
          >
            +{overflow}
          </button>
        )}
      </div>
    </div>
  );
}
