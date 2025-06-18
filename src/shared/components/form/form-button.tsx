import type React from "react"
import { useFormContext } from "./form"

interface FormSubmitButtonProps {
	children: React.ReactNode
	loadingText?: string
	className?: string
}

export function FormSubmitButton({
	children,
	loadingText = "処理中...",
	className = "w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed",
}: FormSubmitButtonProps) {
	const { isLoading } = useFormContext()

	return (
		<button type="submit" disabled={isLoading} className={className}>
			{isLoading ? loadingText : children}
		</button>
	)
}

interface FormCancelButtonProps {
	onClick: () => void
	children: React.ReactNode
	className?: string
}

export function FormCancelButton({
	onClick,
	children,
	className = "w-full bg-gray-300 text-gray-700 py-2 px-4 rounded-md hover:bg-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2",
}: FormCancelButtonProps) {
	return (
		<button type="button" onClick={onClick} className={className}>
			{children}
		</button>
	)
}

interface FormButtonGroupProps {
	children: React.ReactNode
	className?: string
}

export function FormButtonGroup({
	children,
	className = "flex space-x-4",
}: FormButtonGroupProps) {
	return <div className={className}>{children}</div>
}
