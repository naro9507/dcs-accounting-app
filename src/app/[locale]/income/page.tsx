"use client"

import { useTranslations } from "next-intl"
import { useEffect } from "react"
import { useIncome } from "@/features/income/hooks/use-income"
import { IncomeForm } from "@/app/[locale]/income/components/income-form"
import { IncomeList } from "@/app/[locale]/income/components/income-list"

export default function IncomePage() {
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
				amount: parseInt(data.amount),
				description: data.description,
				category:
					data.category as import("@/features/income/income.model").IncomeCategory,
			})
		} catch (error) {
			console.error("Failed to create income:", error)
		}
	}

	const handleCancel = () => {
		console.log("Income form cancelled")
	}

	const handleDelete = async (id: string) => {
		if (confirm("この収入を削除しますか？")) {
			try {
				await deleteIncome(id)
			} catch (error) {
				console.error("Failed to delete income:", error)
			}
		}
	}

	return (
		<div className="container mx-auto p-6">
			<h1 className="text-2xl font-bold mb-6">{t("title")}</h1>

			{error && (
				<div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-md">
					<p className="text-red-700">{error}</p>
				</div>
			)}

			<div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
				{/* 収入追加フォーム */}
				<div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
					<h2 className="text-lg font-semibold mb-4">{t("add")}</h2>

					<IncomeForm
						onSubmit={handleSubmit}
						onCancel={handleCancel}
						isLoading={isLoading}
					/>
				</div>

				{/* 収入一覧 */}
				<div>
					<h2 className="text-lg font-semibold mb-4">収入一覧</h2>

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
