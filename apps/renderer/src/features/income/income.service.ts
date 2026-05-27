import { v4 as uuidv4 } from "uuid"
import { createLogger } from "@/lib/logger"
import { getDatabase } from "@/lib/database"
import type {
	CreateIncomeData,
	IncomeData,
	UpdateIncomeData,
} from "@/features/income/income.model"
import {
	Income,
	validateCreateIncomeData,
} from "@/features/income/income.model"

const logger = createLogger("IncomeService")

// tauri-plugin-sql はスネークケースで返すためキャメルケースに変換
function rowToIncomeData(row: Record<string, unknown>): IncomeData {
	return {
		id: String(row.id),
		date: new Date(String(row.date)),
		amount: Number(row.amount),
		description: String(row.description),
		category: String(row.category) as IncomeData["category"],
		createdAt: new Date(String(row.created_at)),
		updatedAt: new Date(String(row.updated_at)),
	}
}

export class IncomeService {
	async createIncome(data: CreateIncomeData): Promise<Income> {
		const errors = validateCreateIncomeData(data)
		if (errors.length > 0) {
			throw new Error(`入力エラー: ${errors.join(", ")}`)
		}

		try {
			const db = await getDatabase()
			const id = uuidv4()
			const now = new Date().toISOString()

			await db.execute(
				`INSERT INTO incomes (id, date, amount, description, category, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)`,
				[
					id,
					data.date.toISOString(),
					data.amount,
					data.description,
					data.category,
					now,
					now,
				],
			)

			logger.info("Income created successfully", { id })
			return Income.fromData({
				id,
				date: data.date,
				amount: data.amount,
				description: data.description,
				category: data.category,
				createdAt: new Date(now),
				updatedAt: new Date(now),
			})
		} catch (error) {
			logger.error("Failed to create income", error)
			if (error instanceof Error) throw error
			throw new Error("収入の作成に失敗しました")
		}
	}

	async getAllIncomes(): Promise<Income[]> {
		try {
			const db = await getDatabase()
			const rows: Record<string, unknown>[] = await db.select(
				"SELECT * FROM incomes ORDER BY date DESC",
			)
			return rows.map((r) => Income.fromData(rowToIncomeData(r)))
		} catch (error) {
			logger.error("Failed to get all incomes", error)
			throw new Error("収入一覧の取得に失敗しました")
		}
	}

	async getIncomeById(id: string): Promise<Income | null> {
		try {
			const db = await getDatabase()
			const rows: Record<string, unknown>[] = await db.select(
				"SELECT * FROM incomes WHERE id = $1",
				[id],
			)
			if (rows.length === 0) return null
			return Income.fromData(rowToIncomeData(rows[0]))
		} catch (error) {
			logger.error("Failed to get income by id", { id, error })
			throw new Error("収入の取得に失敗しました")
		}
	}

	async getIncomesByDateRange(
		startDate: Date,
		endDate: Date,
	): Promise<Income[]> {
		try {
			const db = await getDatabase()
			const rows: Record<string, unknown>[] = await db.select(
				"SELECT * FROM incomes WHERE date >= $1 AND date <= $2 ORDER BY date DESC",
				[startDate.toISOString(), endDate.toISOString()],
			)
			return rows.map((r) => Income.fromData(rowToIncomeData(r)))
		} catch (error) {
			logger.error("Failed to get incomes by date range", {
				startDate,
				endDate,
				error,
			})
			throw new Error("期間別収入の取得に失敗しました")
		}
	}

	async getIncomesByCategory(category: string): Promise<Income[]> {
		try {
			const db = await getDatabase()
			const rows: Record<string, unknown>[] = await db.select(
				"SELECT * FROM incomes WHERE category = $1 ORDER BY date DESC",
				[category],
			)
			return rows.map((r) => Income.fromData(rowToIncomeData(r)))
		} catch (error) {
			logger.error("Failed to get incomes by category", { category, error })
			throw new Error("カテゴリ別収入の取得に失敗しました")
		}
	}

	async updateIncome(data: UpdateIncomeData): Promise<Income> {
		const existing = await this.getIncomeById(data.id)
		if (!existing) throw new Error("収入が見つかりません")

		const updated = existing.update(data)
		const errors = updated.validate()
		if (errors.length > 0) {
			throw new Error(`入力エラー: ${errors.join(", ")}`)
		}

		try {
			const db = await getDatabase()
			const now = new Date().toISOString()
			await db.execute(
				`UPDATE incomes
         SET date = $1, amount = $2, description = $3, category = $4, updated_at = $5
         WHERE id = $6`,
				[
					updated.date.toISOString(),
					updated.amount,
					updated.description,
					updated.category,
					now,
					data.id,
				],
			)
			logger.info("Income updated successfully", { id: data.id })
			return updated
		} catch (error) {
			logger.error("Failed to update income", { id: data.id, error })
			if (error instanceof Error) throw error
			throw new Error("収入の更新に失敗しました")
		}
	}

	async deleteIncome(id: string): Promise<void> {
		const existing = await this.getIncomeById(id)
		if (!existing) throw new Error("収入が見つかりません")

		try {
			const db = await getDatabase()
			await db.execute("DELETE FROM incomes WHERE id = $1", [id])
			logger.info("Income deleted successfully", { id })
		} catch (error) {
			logger.error("Failed to delete income", { id, error })
			if (error instanceof Error) throw error
			throw new Error("収入の削除に失敗しました")
		}
	}

	async getTotalAmount(): Promise<number> {
		try {
			const db = await getDatabase()
			const rows: Array<{ total: number | null }> = await db.select(
				"SELECT SUM(amount) AS total FROM incomes",
			)
			return rows[0]?.total ?? 0
		} catch (error) {
			logger.error("Failed to get total amount", error)
			throw new Error("総収入の取得に失敗しました")
		}
	}

	async getTotalAmountByCategory(category: string): Promise<number> {
		try {
			const db = await getDatabase()
			const rows: Array<{ total: number | null }> = await db.select(
				"SELECT SUM(amount) AS total FROM incomes WHERE category = $1",
				[category],
			)
			return rows[0]?.total ?? 0
		} catch (error) {
			logger.error("Failed to get total amount by category", {
				category,
				error,
			})
			throw new Error("カテゴリ別総収入の取得に失敗しました")
		}
	}

	async getTotalAmountByDateRange(
		startDate: Date,
		endDate: Date,
	): Promise<number> {
		try {
			const db = await getDatabase()
			const rows: Array<{ total: number | null }> = await db.select(
				"SELECT SUM(amount) AS total FROM incomes WHERE date >= $1 AND date <= $2",
				[startDate.toISOString(), endDate.toISOString()],
			)
			return rows[0]?.total ?? 0
		} catch (error) {
			logger.error("Failed to get total amount by date range", {
				startDate,
				endDate,
				error,
			})
			throw new Error("期間別総収入の取得に失敗しました")
		}
	}
}
