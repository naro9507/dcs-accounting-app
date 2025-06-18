import { useCallback, useState } from "react"
import { createLogger } from "@/lib/logger"
import type {
	CreateIncomeData,
	Income,
	UpdateIncomeData,
} from "../income.model"
import { IncomeService } from "../income.service"

const logger = createLogger("useIncome")

// 型をエクスポート（コンポーネントが使用するため）
export type { Income, CreateIncomeData, UpdateIncomeData }

// サービスのインスタンス
const incomeService = new IncomeService()

interface UseIncomeReturn {
	incomes: Income[]
	isLoading: boolean
	error: string | null
	createIncome: (data: CreateIncomeData) => Promise<void>
	updateIncome: (data: UpdateIncomeData) => Promise<void>
	deleteIncome: (id: string) => Promise<void>
	refreshIncomes: () => Promise<void>
	getTotalAmount: () => Promise<number>
}

export function useIncome(): UseIncomeReturn {
	const [incomes, setIncomes] = useState<Income[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	const refreshIncomes = useCallback(async () => {
		try {
			setIsLoading(true)
			setError(null)
			const fetchedIncomes = await incomeService.getAllIncomes()
			setIncomes(fetchedIncomes)
		} catch (err) {
			const errorMessage =
				err instanceof Error ? err.message : "収入の取得に失敗しました"
			setError(errorMessage)
			logger.error("Failed to refresh incomes", err)
		} finally {
			setIsLoading(false)
		}
	}, [])

	const createIncome = useCallback(
		async (data: CreateIncomeData) => {
			try {
				setIsLoading(true)
				setError(null)
				await incomeService.createIncome(data)
				await refreshIncomes()
			} catch (err) {
				const errorMessage =
					err instanceof Error ? err.message : "収入の作成に失敗しました"
				setError(errorMessage)
				logger.error("Failed to create income", err)
				throw err
			} finally {
				setIsLoading(false)
			}
		},
		[refreshIncomes]
	)

	const updateIncome = useCallback(
		async (data: UpdateIncomeData) => {
			try {
				setIsLoading(true)
				setError(null)
				await incomeService.updateIncome(data)
				await refreshIncomes()
			} catch (err) {
				const errorMessage =
					err instanceof Error ? err.message : "収入の更新に失敗しました"
				setError(errorMessage)
				logger.error("Failed to update income", err)
				throw err
			} finally {
				setIsLoading(false)
			}
		},
		[refreshIncomes]
	)

	const deleteIncome = useCallback(
		async (id: string) => {
			try {
				setIsLoading(true)
				setError(null)
				await incomeService.deleteIncome(id)
				await refreshIncomes()
			} catch (err) {
				const errorMessage =
					err instanceof Error ? err.message : "収入の削除に失敗しました"
				setError(errorMessage)
				logger.error("Failed to delete income", err)
				throw err
			} finally {
				setIsLoading(false)
			}
		},
		[refreshIncomes]
	)

	const getTotalAmount = useCallback(async (): Promise<number> => {
		try {
			return await incomeService.getTotalAmount()
		} catch (err) {
			logger.error("Failed to get total amount", err)
			return 0
		}
	}, [])

	return {
		incomes,
		isLoading,
		error,
		createIncome,
		updateIncome,
		deleteIncome,
		refreshIncomes,
		getTotalAmount,
	}
}
