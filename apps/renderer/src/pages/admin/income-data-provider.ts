import type { DataProvider } from "react-admin"
import { IncomeService } from "@/features/income/income.service"
import type { CreateIncomeData, UpdateIncomeData } from "@/features/income/income.model"
import { INCOME_CATEGORIES } from "@/features/income/income.model"

const incomeService = new IncomeService()

function ensureIncomeResource(resource: string) {
	if (resource !== "incomes") {
		throw new Error(`Unsupported resource: ${resource}`)
	}
}

function toRecord(income: {
	id: string
	date: Date
	amount: number
	description: string
	category: string
}) {
	return {
		...income,
	}
}

export const incomeDataProvider: DataProvider = {
	async getList(resource, params) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes.map((income) => toRecord(income))

		if (params.sort?.field) {
			const direction = params.sort.order === "DESC" ? -1 : 1
			data.sort((left, right) => {
				const leftValue = left[params.sort.field as keyof typeof left]
				const rightValue = right[params.sort.field as keyof typeof right]
				if (leftValue instanceof Date && rightValue instanceof Date) {
					return (leftValue.getTime() - rightValue.getTime()) * direction
				}
				if (typeof leftValue === "number" && typeof rightValue === "number") {
					return (leftValue - rightValue) * direction
				}
				return String(leftValue).localeCompare(String(rightValue)) * direction
			})
		}

		const start = (params.pagination.page - 1) * params.pagination.perPage
		const end = start + params.pagination.perPage
		const paginated = data.slice(start, end)

		return {
			data: paginated,
			total: data.length,
		}
	},
	async getOne(resource, params) {
		ensureIncomeResource(resource)
		const income = await incomeService.getIncomeById(params.id as string)
		if (!income) {
			throw new Error("Income not found")
		}
		return { data: toRecord(income) }
	},
	async getMany(resource, params) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes
			.filter((income) => params.ids.includes(income.id))
			.map((income) => toRecord(income))
		return { data }
	},
	async getManyReference(resource, params) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes.map((income) => toRecord(income))
		return { data, total: data.length }
	},
	async create(resource, params) {
		ensureIncomeResource(resource)
		const data = params.data as Record<string, string | number | Date>
		const createData: CreateIncomeData = {
			date: new Date(data.date as string),
			amount: Number(data.amount),
			description: String(data.description),
			category: INCOME_CATEGORIES.includes(data.category as any)
				? (data.category as any)
				: "売上",
		}
		const created = await incomeService.createIncome(createData)
		return { data: toRecord(created) }
	},
	async update(resource, params) {
		ensureIncomeResource(resource)
		const data = params.data as Record<string, string | number | Date>
		const updateData: UpdateIncomeData = {
			id: params.id as string,
			date: data.date ? new Date(data.date as string) : undefined,
			amount: data.amount !== undefined ? Number(data.amount) : undefined,
			description: data.description ? String(data.description) : undefined,
			category: data.category ? (data.category as any) : undefined,
		}
		const updated = await incomeService.updateIncome(updateData)
		return { data: toRecord(updated) }
	},
	async delete(resource, params) {
		ensureIncomeResource(resource)
		await incomeService.deleteIncome(params.id as string)
		return { data: params.previousData }
	},
	async deleteMany(resource, params) {
		ensureIncomeResource(resource)
		await Promise.all(params.ids.map((id) => incomeService.deleteIncome(id as string)))
		return { data: params.ids }
	},
	async updateMany(resource, params) {
		ensureIncomeResource(resource)
		const data = params.data as Record<string, string | number | Date>
		const updates = await Promise.all(
			params.ids.map(async (id) => {
				const updateData: UpdateIncomeData = {
					id: id as string,
					date: data.date ? new Date(data.date as string) : undefined,
					amount: data.amount !== undefined ? Number(data.amount) : undefined,
					description: data.description ? String(data.description) : undefined,
					category: data.category ? (data.category as any) : undefined,
				}
				const updated = await incomeService.updateIncome(updateData)
				return updated.id
			})
		)
		return { data: updates }
	},
}
