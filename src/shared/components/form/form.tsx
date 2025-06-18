import type React from "react"
import { createContext, type FormEvent, useContext, useState } from "react"

interface FormContextValue<T> {
	data: T
	updateField: (field: keyof T, value: unknown) => void
	isLoading: boolean
	errors: Partial<Record<keyof T, string>>
	setError: (field: keyof T, message: string) => void
	clearError: (field: keyof T) => void
}

const FormContext = createContext<FormContextValue<
	Record<string, unknown>
> | null>(null)

export function useFormContext<
	T extends Record<string, unknown>,
>(): FormContextValue<T> {
	const context = useContext(FormContext)
	if (!context) {
		throw new Error("Form components must be used within a Form")
	}
	return context as FormContextValue<T>
}

interface FormProps<T> {
	children: React.ReactNode
	onSubmit: (data: T) => void | Promise<void>
	initialData: T
	isLoading?: boolean
	className?: string
}

export function Form<T extends Record<string, unknown>>({
	children,
	onSubmit,
	initialData,
	isLoading = false,
	className = "space-y-4",
}: FormProps<T>) {
	const [data, setData] = useState<T>(initialData)
	const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({})

	const updateField = (field: keyof T, value: unknown) => {
		setData((prev) => ({ ...prev, [field]: value }))
		if (errors[field]) {
			clearError(field)
		}
	}

	const setError = (field: keyof T, message: string) => {
		setErrors((prev) => ({ ...prev, [field]: message }))
	}

	const clearError = (field: keyof T) => {
		setErrors((prev) => {
			const newErrors = { ...prev }
			delete newErrors[field]
			return newErrors
		})
	}

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault()
		try {
			await onSubmit(data)
		} catch (error) {
			console.error("Form submission error:", error)
		}
	}

	const contextValue: FormContextValue<T> = {
		data,
		updateField,
		isLoading,
		errors,
		setError,
		clearError,
	}

	return (
		<FormContext.Provider value={contextValue}>
			<form onSubmit={handleSubmit} className={className}>
				{children}
			</form>
		</FormContext.Provider>
	)
}
