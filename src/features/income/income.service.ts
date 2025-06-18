import { createLogger } from "@/lib/logger"
import { prisma } from "@/lib/prisma"
import type {
	CreateIncomeData,
	IncomeData,
	UpdateIncomeData,
} from "./income.model"
import { Income, validateCreateIncomeData } from "./income.model"

const logger = createLogger("IncomeService")

// 収入に関する全ての操作を管理するサービス
export class IncomeService {
	// 新しい収入を作成
	async createIncome(data: CreateIncomeData): Promise<Income> {
		try {
			// バリデーション
			const errors = validateCreateIncomeData(data)
			if (errors.length > 0) {
				throw new Error(`入力エラー: ${errors.join(", ")}`)
			}

			// Prismaでデータベースに保存
			const created = await prisma.income.create({
				data: {
					date: data.date,
					amount: data.amount,
					description: data.description,
					category: data.category,
				},
			})

			logger.info("Income created successfully", { id: created.id })

			// ドメインモデルに変換して返す
			return Income.fromData(created as IncomeData)
		} catch (error) {
			logger.error("Failed to create income", error)
			if (error instanceof Error) {
				throw error
			}
			throw new Error("収入の作成に失敗しました")
		}
	}

	// 全ての収入を取得
	async getAllIncomes(): Promise<Income[]> {
		try {
			const incomes = await prisma.income.findMany({
				orderBy: { date: "desc" },
			})

			return incomes.map((income) => Income.fromData(income as IncomeData))
		} catch (error) {
			logger.error("Failed to get all incomes", error)
			throw new Error("収入一覧の取得に失敗しました")
		}
	}

	// IDで収入を取得
	async getIncomeById(id: string): Promise<Income | null> {
		try {
			const income = await prisma.income.findUnique({
				where: { id },
			})

			if (!income) {
				return null
			}

			return Income.fromData(income as IncomeData)
		} catch (error) {
			logger.error("Failed to get income by id", { id, error })
			throw new Error("収入の取得に失敗しました")
		}
	}

	// 期間で収入を取得
	async getIncomesByDateRange(
		startDate: Date,
		endDate: Date
	): Promise<Income[]> {
		try {
			const incomes = await prisma.income.findMany({
				where: {
					date: {
						gte: startDate,
						lte: endDate,
					},
				},
				orderBy: { date: "desc" },
			})

			return incomes.map((income) => Income.fromData(income as IncomeData))
		} catch (error) {
			logger.error("Failed to get incomes by date range", {
				startDate,
				endDate,
				error,
			})
			throw new Error("期間別収入の取得に失敗しました")
		}
	}

	// カテゴリで収入を取得
	async getIncomesByCategory(category: string): Promise<Income[]> {
		try {
			const incomes = await prisma.income.findMany({
				where: { category },
				orderBy: { date: "desc" },
			})

			return incomes.map((income) => Income.fromData(income as IncomeData))
		} catch (error) {
			logger.error("Failed to get incomes by category", { category, error })
			throw new Error("カテゴリ別収入の取得に失敗しました")
		}
	}

	// 収入を更新
	async updateIncome(data: UpdateIncomeData): Promise<Income> {
		try {
			// 既存の収入を取得
			const existingIncome = await this.getIncomeById(data.id)
			if (!existingIncome) {
				throw new Error("収入が見つかりません")
			}

			// ドメインモデルで更新処理とバリデーション
			const updatedIncome = existingIncome.update(data)
			const errors = updatedIncome.validate()
			if (errors.length > 0) {
				throw new Error(`入力エラー: ${errors.join(", ")}`)
			}

			// データベースを更新
			const updated = await prisma.income.update({
				where: { id: data.id },
				data: updatedIncome.toData(),
			})

			logger.info("Income updated successfully", { id: data.id })
			return Income.fromData(updated as IncomeData)
		} catch (error) {
			logger.error("Failed to update income", { id: data.id, error })
			if (error instanceof Error) {
				throw error
			}
			throw new Error("収入の更新に失敗しました")
		}
	}

	// 収入を削除
	async deleteIncome(id: string): Promise<void> {
		try {
			// 存在チェック
			const existingIncome = await this.getIncomeById(id)
			if (!existingIncome) {
				throw new Error("収入が見つかりません")
			}

			await prisma.income.delete({
				where: { id },
			})

			logger.info("Income deleted successfully", { id })
		} catch (error) {
			logger.error("Failed to delete income", { id, error })
			if (error instanceof Error) {
				throw error
			}
			throw new Error("収入の削除に失敗しました")
		}
	}

	// 合計金額を取得
	async getTotalAmount(): Promise<number> {
		try {
			const result = await prisma.income.aggregate({
				_sum: {
					amount: true,
				},
			})

			return result._sum.amount || 0
		} catch (error) {
			logger.error("Failed to get total amount", error)
			throw new Error("総収入の取得に失敗しました")
		}
	}

	// カテゴリ別合計金額を取得
	async getTotalAmountByCategory(category: string): Promise<number> {
		try {
			const result = await prisma.income.aggregate({
				where: { category },
				_sum: {
					amount: true,
				},
			})

			return result._sum.amount || 0
		} catch (error) {
			logger.error("Failed to get total amount by category", {
				category,
				error,
			})
			throw new Error("カテゴリ別総収入の取得に失敗しました")
		}
	}

	// 期間別合計金額を取得
	async getTotalAmountByDateRange(
		startDate: Date,
		endDate: Date
	): Promise<number> {
		try {
			const result = await prisma.income.aggregate({
				where: {
					date: {
						gte: startDate,
						lte: endDate,
					},
				},
				_sum: {
					amount: true,
				},
			})

			return result._sum.amount || 0
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
