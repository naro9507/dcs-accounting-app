import type React from "react"

interface DataListContainerProps {
	children: React.ReactNode
	className?: string
}

export function DataListContainer({
	children,
	className = "bg-white shadow-sm rounded-lg border border-gray-200 overflow-hidden",
}: DataListContainerProps) {
	return <div className={className}>{children}</div>
}

interface DataListHeaderProps {
	children: React.ReactNode
	className?: string
}

export function DataListHeader({
	children,
	className = "p-4 bg-gray-50 font-medium text-gray-700 border-b",
}: DataListHeaderProps) {
	return <div className={className}>{children}</div>
}

interface DataListBodyProps {
	children: React.ReactNode
	loadingComponent?: React.ReactNode
	emptyComponent?: React.ReactNode
	className?: string
}

export function DataListBody({
	children,
	loadingComponent: _loadingComponent,
	emptyComponent: _emptyComponent,
	className = "divide-y divide-gray-200",
}: DataListBodyProps) {
	return <div className={className}>{children}</div>
}

interface DataListFooterProps {
	children: React.ReactNode
	className?: string
}

export function DataListFooter({
	children,
	className = "p-4 bg-gray-50 border-t",
}: DataListFooterProps) {
	return <div className={className}>{children}</div>
}
