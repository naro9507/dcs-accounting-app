import { useTranslations } from "next-intl"
import {
	Form,
	FormButtonGroup,
	FormCancelButton,
	FormField,
	FormInput,
	FormLabel,
	FormSelect,
	FormSubmitButton,
	FormTextarea,
} from "@/shared/components/form"

interface IncomeFormData extends Record<string, string> {
	date: string
	amount: string
	description: string
	category: string
}

interface IncomeFormProps {
	onSubmit: (data: IncomeFormData) => void
	onCancel: () => void
	initialData?: Partial<IncomeFormData>
	isLoading?: boolean
}

export function IncomeForm({
	onSubmit,
	onCancel,
	initialData = {},
	isLoading = false,
}: IncomeFormProps) {
	const t = useTranslations("income")

	const defaultData: IncomeFormData = {
		date: new Date().toISOString().split("T")[0],
		amount: "",
		description: "",
		category: "売上",
		...initialData,
	}

	const categoryOptions = [
		{ value: "売上", label: "売上" },
		{ value: "給与", label: "給与" },
		{ value: "雑収入", label: "雑収入" },
		{ value: "その他", label: "その他" },
	]

	return (
		<Form initialData={defaultData} onSubmit={onSubmit} isLoading={isLoading}>
			<FormField>
				<FormLabel htmlFor="date" required>
					{t("date")}
				</FormLabel>
				<FormInput<IncomeFormData> name="date" type="date" required />
			</FormField>

			<FormField>
				<FormLabel htmlFor="amount" required>
					{t("amount")}
				</FormLabel>
				<FormInput<IncomeFormData>
					name="amount"
					type="number"
					placeholder="0"
					min="0"
					step="1"
					required
				/>
			</FormField>

			<FormField>
				<FormLabel htmlFor="description" required>
					{t("description")}
				</FormLabel>
				<FormTextarea<IncomeFormData>
					name="description"
					maxLength={500}
					required
				/>
			</FormField>

			<FormField>
				<FormLabel htmlFor="category" required>
					{t("category")}
				</FormLabel>
				<FormSelect<IncomeFormData>
					name="category"
					options={categoryOptions}
					required
				/>
			</FormField>

			<FormButtonGroup>
				<FormSubmitButton>{t("add")}</FormSubmitButton>
				<FormCancelButton onClick={onCancel}>キャンセル</FormCancelButton>
			</FormButtonGroup>
		</Form>
	)
}
