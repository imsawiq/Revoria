"use client";

/**
 * SortableList — Drag-to-reorder list using @dnd-kit.
 *
 * Provides a sortable list where items can be reordered via drag
 * and drop. Fully accessible with keyboard support.
 *
 * @module components/composed/sortable-list
 */

import {
	DndContext,
	closestCenter,
	KeyboardSensor,
	PointerSensor,
	useSensor,
	useSensors,
	type DragEndEvent,
} from '@dnd-kit/core'
import {
	arrayMove,
	SortableContext,
	sortableKeyboardCoordinates,
	useSortable,
	verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical } from 'lucide-react'
import * as React from 'react'

import { cn } from '@modrinth/ui'

// ============================================================================
// Types
// ============================================================================

export interface SortableItem {
	id: string;
	content: React.ReactNode;
}

export interface SortableListProps {
	/** Items to display */
	items: SortableItem[];
	/** Called when order changes */
	onReorder: (items: SortableItem[]) => void;
	/** Show drag handle */
	showHandle?: boolean;
	/** Additional CSS classes */
	className?: string;
	/** Item wrapper CSS classes */
	itemClassName?: string;
}

// ============================================================================
// SortableItem component
// ============================================================================

function SortableListItem({
	item,
	showHandle,
	className,
}: {
	item: SortableItem;
	showHandle: boolean;
	className?: string;
}) {
	const {
		attributes,
		listeners,
		setNodeRef,
		transform,
		transition,
		isDragging,
	} = useSortable({ id: item.id })

	const style = {
		transform: CSS.Transform.toString(transform),
		transition,
	}

	return (
		<div
			ref={setNodeRef}
			style={style}
			className={cn(
				'flex items-center gap-2 rounded-lg border bg-background p-3',
				isDragging && 'opacity-50 shadow-lg z-50',
				className,
			)}
		>
			{showHandle && (
				<button
					type="button"
					className="cursor-grab active:cursor-grabbing text-muted-foreground hover:text-foreground"
					{...attributes}
					{...listeners}
				>
					<GripVertical className="h-4 w-4" />
				</button>
			)}
			<div className="flex-1 min-w-0">{item.content}</div>
		</div>
	)
}

// ============================================================================
// SortableList component
// ============================================================================

export function SortableList({
	items,
	onReorder,
	showHandle = true,
	className,
	itemClassName,
}: SortableListProps) {
	const sensors = useSensors(
		useSensor(PointerSensor),
		useSensor(KeyboardSensor, {
			coordinateGetter: sortableKeyboardCoordinates,
		}),
	)

	const handleDragEnd = React.useCallback(
		(event: DragEndEvent) => {
			const { active, over } = event
			if (over && active.id !== over.id) {
				const oldIndex = items.findIndex((item) => item.id === active.id)
				const newIndex = items.findIndex((item) => item.id === over.id)
				onReorder(arrayMove(items, oldIndex, newIndex))
			}
		},
		[items, onReorder],
	)

	return (
		<DndContext
			sensors={sensors}
			collisionDetection={closestCenter}
			onDragEnd={handleDragEnd}
		>
			<SortableContext
				items={items.map((item) => item.id)}
				strategy={verticalListSortingStrategy}
			>
				<div className={cn('flex flex-col gap-2', className)}>
					{items.map((item) => (
						<SortableListItem
							key={item.id}
							item={item}
							showHandle={showHandle}
							className={itemClassName}
						/>
					))}
				</div>
			</SortableContext>
		</DndContext>
	)
}
