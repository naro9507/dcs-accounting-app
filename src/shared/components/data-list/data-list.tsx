import type React from "react"
import { createContext, useContext } from "react"

interface DataListContextValue<T> {
	items: T[]
	onEdit?: (item: T) => void
	onDelete?: (id: string) => void
	isLoading: boolean
	emptyMessage?: string
}

const DataListContext = createContext<DataListContextValue<unknown> | null>(
	null
)

export function useDataListContext<T>(): DataListContextValue<T> {
	const context = useContext(DataListContext)
	if (!context) {
		throw new Error("DataList components must be used within a DataList")
	}
	return context as DataListContextValue<T>
}

interface DataListProps<T> {
	children: React.ReactNode
	items: T[]
	onEdit?: (item: T) => void
	onDelete?: (id: string) => void
	isLoading?: boolean
	emptyMessage?: string
	className?: string
}

export function DataList<T>({
	children,
	items,
	onEdit,
	onDelete,
	isLoading = false,
	emptyMessage = "データがありません",
	className = "data-list",
}: DataListProps<T>) {
	const contextValue: DataListContextValue<T> = {
		items,
		onEdit,
		onDelete,
		isLoading,
		emptyMessage,
	}

	return (
		<DataListContext.Provider
			value={contextValue as DataListContextValue<unknown>}
		>
			<div className={className}>{children}</div>
		</DataListContext.Provider>
	)
}
