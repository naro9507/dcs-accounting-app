import { useTranslations } from "next-intl"
import type { Income } from "@/features/income/hooks/use-income"
import {
	DataList,
	DataListActions,
	DataListBody,
	DataListContainer,
	DataListEmptyState,
	DataListFooter,
	DataListHeader,
	DataListItem,
	DataListLoadingState,
	useDataListContext,
} from "@/shared/components/data-list"

interface IncomeListProps {
	incomes: Income[]
	onEdit?: (income: Income) => void
	onDelete?: (id: string) => void
	isLoading?: boolean
}

export function IncomeList({
	incomes,
	onEdit,
	onDelete,
	isLoading = false,
}: IncomeListProps) {
	return (
		<DataList
			items={incomes}
			onEdit={onEdit}
			onDelete={onDelete}
			isLoading={isLoading}
			emptyMessage="収入データがありません"
		>
			<DataListContainer>
				<IncomeListHeader />
				<IncomeListBody />
				<IncomeListFooter />
			</DataListContainer>
		</DataList>
	)
}

function IncomeListHeader() {
	const t = useTranslations("income")

	return (
		<DataListHeader className="grid grid-cols-6 gap-4 p-4 bg-gray-50 font-medium text-gray-700 border-b">
			<div>{t("date")}</div>
			<div>{t("amount")}</div>
			<div className="col-span-2">{t("description")}</div>
			<div>{t("category")}</div>
			<div>操作</div>
		</DataListHeader>
	)
}

function IncomeListBody() {
	const { items: incomes, isLoading } = useDataListContext<Income>()

	if (isLoading) {
		return <DataListLoadingState />
	}

	if (incomes.length === 0) {
		return <DataListEmptyState message="収入データがありません" />
	}

	return (
		<DataListBody>
			{incomes.map((income) => (
				<DataListItem
					key={income.id}
					item={income}
					getItemId={(item) => item.id}
					className="grid grid-cols-6 gap-4 p-4"
				>
					{(income) => (
						<>
							<div className="text-gray-900">{income.formatDate()}</div>
							<div className="text-gray-900 font-medium">
								{income.formatAmount()}
							</div>
							<div className="col-span-2 text-gray-700">
								{income.description}
							</div>
							<div className="text-gray-700">{income.category}</div>
							<DataListActions
								item={income}
								getItemId={(item) => item.id}
								editLabel="編集"
								deleteLabel="削除"
							/>
						</>
					)}
				</DataListItem>
			))}
		</DataListBody>
	)
}

function IncomeListFooter() {
	const { items: incomes } = useDataListContext<Income>()

	const total = incomes.reduce((sum, income) => sum + income.amount, 0)

	return (
		<DataListFooter>
			<div className="flex justify-between items-center">
				<span className="text-gray-700">合計: {incomes.length}件</span>
				<span className="text-lg font-bold text-gray-900">
					{new Intl.NumberFormat("ja-JP", {
						style: "currency",
						currency: "JPY",
					}).format(total)}
				</span>
			</div>
		</DataListFooter>
	)
}
