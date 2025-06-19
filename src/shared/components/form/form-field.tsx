import type React from "react"
import { useFormContext } from "@/shared/components/form/form"

interface FormFieldProps {
	children: React.ReactNode
	className?: string
}

export function FormField({
	children,
	className = "form-field",
}: FormFieldProps) {
	return <div className={className}>{children}</div>
}

interface FormLabelProps {
	htmlFor: string
	children: React.ReactNode
	required?: boolean
	className?: string
}

export function FormLabel({
	htmlFor,
	children,
	required = false,
	className = "block text-sm font-medium text-gray-700 mb-1",
}: FormLabelProps) {
	return (
		<label htmlFor={htmlFor} className={className}>
			{children}
			{required && <span className="text-red-500 ml-1">*</span>}
		</label>
	)
}

interface FormInputProps<T> {
	name: keyof T
	type?: string
	placeholder?: string
	required?: boolean
	min?: string | number
	max?: string | number
	step?: string | number
	className?: string
}

export function FormInput<T extends Record<string, unknown>>({
	name,
	type = "text",
	placeholder,
	required = false,
	min,
	max,
	step,
	className = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
}: FormInputProps<T>) {
	const { data, updateField, errors } = useFormContext<T>()
	const error = errors[name]

	return (
		<>
			<input
				id={String(name)}
				name={String(name)}
				type={type}
				value={String(data[name] || "")}
				onChange={(e) => updateField(name, e.target.value)}
				placeholder={placeholder}
				required={required}
				min={min}
				max={max}
				step={step}
				className={`${className} ${error ? "border-red-500 focus:ring-red-500 focus:border-red-500" : ""}`}
			/>
			{error && <p className="mt-1 text-sm text-red-600">{error}</p>}
		</>
	)
}

interface FormTextareaProps<T> {
	name: keyof T
	placeholder?: string
	required?: boolean
	rows?: number
	maxLength?: number
	className?: string
}

export function FormTextarea<T extends Record<string, unknown>>({
	name,
	placeholder,
	required = false,
	rows = 3,
	maxLength,
	className = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
}: FormTextareaProps<T>) {
	const { data, updateField, errors } = useFormContext<T>()
	const error = errors[name]

	return (
		<>
			<textarea
				id={String(name)}
				name={String(name)}
				value={String(data[name] || "")}
				onChange={(e) => updateField(name, e.target.value)}
				placeholder={placeholder}
				required={required}
				rows={rows}
				maxLength={maxLength}
				className={`${className} ${error ? "border-red-500 focus:ring-red-500 focus:border-red-500" : ""}`}
			/>
			{error && <p className="mt-1 text-sm text-red-600">{error}</p>}
		</>
	)
}

interface FormSelectProps<T> {
	name: keyof T
	options: Array<{ value: string; label: string }>
	required?: boolean
	className?: string
}

export function FormSelect<T extends Record<string, unknown>>({
	name,
	options,
	required = false,
	className = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
}: FormSelectProps<T>) {
	const { data, updateField, errors } = useFormContext<T>()
	const error = errors[name]

	return (
		<>
			<select
				id={String(name)}
				name={String(name)}
				value={String(data[name] || "")}
				onChange={(e) => updateField(name, e.target.value)}
				required={required}
				className={`${className} ${error ? "border-red-500 focus:ring-red-500 focus:border-red-500" : ""}`}
			>
				{options.map((option) => (
					<option key={option.value} value={option.value}>
						{option.label}
					</option>
				))}
			</select>
			{error && <p className="mt-1 text-sm text-red-600">{error}</p>}
		</>
	)
}
