import type React from "react"
import { useDataListContext } from "./data-list"

interface DataListItemProps<T> {
	item: T
	children: (item: T) => React.ReactNode
	getItemId: (item: T) => string
	className?: string
	hoverClassName?: string
}

export function DataListItem<T>({
	item,
	children,
	getItemId: _getItemId,
	className = "p-4",
	hoverClassName = "hover:bg-gray-50",
}: DataListItemProps<T>) {
	return (
		<div className={`${className} ${hoverClassName}`}>{children(item)}</div>
	)
}

interface DataListActionsProps<T> {
	item: T
	getItemId: (item: T) => string
	editLabel?: string
	deleteLabel?: string
	className?: string
	editClassName?: string
	deleteClassName?: string
}

export function DataListActions<T>({
	item,
	getItemId,
	editLabel = "編集",
	deleteLabel = "削除",
	className = "flex space-x-2",
	editClassName = "text-blue-600 hover:text-blue-800 text-sm",
	deleteClassName = "text-red-600 hover:text-red-800 text-sm",
}: DataListActionsProps<T>) {
	const { onEdit, onDelete } = useDataListContext<T>()

	return (
		<div className={className}>
			{onEdit && (
				<button
					type="button"
					onClick={() => onEdit(item)}
					className={editClassName}
				>
					{editLabel}
				</button>
			)}
			{onDelete && (
				<button
					type="button"
					onClick={() => onDelete(getItemId(item))}
					className={deleteClassName}
				>
					{deleteLabel}
				</button>
			)}
		</div>
	)
}

interface DataListEmptyStateProps {
	message?: string
	className?: string
}

export function DataListEmptyState({
	message = "データがありません",
	className = "p-8 text-center text-gray-500",
}: DataListEmptyStateProps) {
	return <div className={className}>{message}</div>
}

interface DataListLoadingStateProps {
	message?: string
	className?: string
}

export function DataListLoadingState({
	message = "読み込み中...",
	className = "p-8 text-center text-gray-500",
}: DataListLoadingStateProps) {
	return <div className={className}>{message}</div>
}
