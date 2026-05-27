import type {
	DataProvider,
	RaRecord,
	GetListParams,
	GetOneParams,
	GetManyParams,
	GetManyReferenceParams,
	CreateParams,
	UpdateParams,
	UpdateManyParams,
	DeleteParams,
	DeleteManyParams,
} from "react-admin"
import { IncomeService } from "@/features/income/income.service"
import type {
	CreateIncomeData,
	UpdateIncomeData,
} from "@/features/income/income.model"
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
}): RaRecord {
	return income as RaRecord
}

// react-admin DataProvider のジェネリクス制約を回避するため型アサーションを使用
// biome-ignore lint/suspicious/noExplicitAny: required for DataProvider compatibility
export const incomeDataProvider = {
	async getList(resource: string, params: GetListParams) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes.map((income) => toRecord(income))

		const sortField = params.sort?.field
		if (sortField) {
			const direction = params.sort?.order === "DESC" ? -1 : 1
			data.sort((left, right) => {
				const leftValue = left[sortField]
				const rightValue = right[sortField]
				if (leftValue instanceof Date && rightValue instanceof Date) {
					return (leftValue.getTime() - rightValue.getTime()) * direction
				}
				if (typeof leftValue === "number" && typeof rightValue === "number") {
					return (leftValue - rightValue) * direction
				}
				return String(leftValue).localeCompare(String(rightValue)) * direction
			})
		}

		const page = params.pagination?.page ?? 1
		const perPage = params.pagination?.perPage ?? 25
		const start = (page - 1) * perPage
		const paginated = data.slice(start, start + perPage)

		return { data: paginated, total: data.length }
	},
	async getOne(resource: string, params: GetOneParams) {
		ensureIncomeResource(resource)
		const income = await incomeService.getIncomeById(params.id as string)
		if (!income) throw new Error("Income not found")
		return { data: toRecord(income) }
	},
	async getMany(resource: string, params: GetManyParams) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes
			.filter((income) => params.ids.includes(income.id))
			.map((income) => toRecord(income))
		return { data }
	},
	async getManyReference(resource: string, params: GetManyReferenceParams) {
		ensureIncomeResource(resource)
		const incomes = await incomeService.getAllIncomes()
		const data = incomes.map((income) => toRecord(income))
		return { data, total: data.length }
	},
	async create(resource: string, params: CreateParams) {
		ensureIncomeResource(resource)
		const raw = params.data as Record<string, string | number | Date>
		const createData: CreateIncomeData = {
			date: new Date(raw.date as string),
			amount: Number(raw.amount),
			description: String(raw.description),
			// biome-ignore lint/suspicious/noExplicitAny: dynamic user input
			category: INCOME_CATEGORIES.includes(raw.category as any)
				? // biome-ignore lint/suspicious/noExplicitAny: dynamic user input
					(raw.category as any)
				: "売上",
		}
		const created = await incomeService.createIncome(createData)
		return { data: toRecord(created) }
	},
	async update(resource: string, params: UpdateParams) {
		ensureIncomeResource(resource)
		const raw = params.data as Record<string, string | number | Date>
		const updateData: UpdateIncomeData = {
			id: params.id as string,
			date: raw.date ? new Date(raw.date as string) : undefined,
			amount: raw.amount !== undefined ? Number(raw.amount) : undefined,
			description: raw.description ? String(raw.description) : undefined,
			// biome-ignore lint/suspicious/noExplicitAny: dynamic user input
			category: raw.category ? (raw.category as any) : undefined,
		}
		const updated = await incomeService.updateIncome(updateData)
		return { data: toRecord(updated) }
	},
	async delete(resource: string, params: DeleteParams) {
		ensureIncomeResource(resource)
		await incomeService.deleteIncome(params.id as string)
		return { data: params.previousData as RaRecord }
	},
	async deleteMany(resource: string, params: DeleteManyParams) {
		ensureIncomeResource(resource)
		await Promise.all(
			params.ids.map((id) => incomeService.deleteIncome(id as string)),
		)
		return { data: params.ids as string[] }
	},
	async updateMany(resource: string, params: UpdateManyParams) {
		ensureIncomeResource(resource)
		const raw = params.data as Record<string, string | number | Date>
		const updates = await Promise.all(
			params.ids.map(async (id) => {
				const updateData: UpdateIncomeData = {
					id: id as string,
					date: raw.date ? new Date(raw.date as string) : undefined,
					amount: raw.amount !== undefined ? Number(raw.amount) : undefined,
					description: raw.description ? String(raw.description) : undefined,
					// biome-ignore lint/suspicious/noExplicitAny: dynamic user input
					category: raw.category ? (raw.category as any) : undefined,
				}
				const updated = await incomeService.updateIncome(updateData)
				return updated.id
			}),
		)
		return { data: updates }
	},
} as unknown as DataProvider
