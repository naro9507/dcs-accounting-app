import { useEffect } from "react"
import { useTranslations } from "@/i18n/use-translations"
import { useIncome } from "@/features/income/hooks/use-income"
import { IncomeForm } from "@/pages/income/components/IncomeForm"
import { IncomeList } from "@/pages/income/components/IncomeList"

export function IncomePage() {
	const t = useTranslations("income")
	const {
		incomes,
		isLoading,
		error,
		createIncome,
		deleteIncome,
		refreshIncomes,
	} = useIncome()

	useEffect(() => {
		refreshIncomes()
	}, [refreshIncomes])

	const handleSubmit = async (data: {
		date: string
		amount: string
		description: string
		category: string
	}) => {
		try {
			await createIncome({
				date: new Date(data.date),
				amount: Number.parseInt(data.amount, 10),
				description: data.description,
				category:
					data.category as import("@/features/income/income.model").IncomeCategory,
			})
		} catch (submitError) {
			console.error("Failed to create income:", submitError)
		}
	}

	const handleCancel = () => {
		console.log("Income form cancelled")
	}

	const handleDelete = async (id: string) => {
		if (confirm("この収入を削除しますか？")) {
			try {
				await deleteIncome(id)
			} catch (deleteError) {
				console.error("Failed to delete income:", deleteError)
			}
		}
	}

	return (
		<div className="container mx-auto p-6">
			<h1 className="mb-6 text-2xl font-bold">{t("title")}</h1>

			{error && (
				<div className="mb-6 rounded-md border border-red-200 bg-red-50 p-4">
					<p className="text-red-700">{error}</p>
				</div>
			)}

			<div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
				<div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
					<h2 className="mb-4 text-lg font-semibold">{t("add")}</h2>

					<IncomeForm
						onSubmit={handleSubmit}
						onCancel={handleCancel}
						isLoading={isLoading}
					/>
				</div>

				<div>
					<h2 className="mb-4 text-lg font-semibold">収入一覧</h2>

					<IncomeList
						incomes={incomes}
						onDelete={handleDelete}
						isLoading={isLoading}
					/>
				</div>
			</div>
		</div>
	)
}
