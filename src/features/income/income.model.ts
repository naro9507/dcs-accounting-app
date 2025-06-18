// 収入データの型定義（Prismaと合わせる）
export type IncomeData = {
	id: string
	date: Date
	amount: number
	description: string
	category: IncomeCategory
	createdAt: Date
	updatedAt: Date
}

// 収入カテゴリの型
export type IncomeCategory = "売上" | "給与" | "雑収入" | "その他"

// 新規収入作成時の型
export type CreateIncomeData = {
	date: Date
	amount: number
	description: string
	category: IncomeCategory
}

// 収入更新時の型
export type UpdateIncomeData = {
	id: string
	date?: Date
	amount?: number
	description?: string
	category?: IncomeCategory
}

// 収入カテゴリの定数
export const INCOME_CATEGORIES: readonly IncomeCategory[] = [
	"売上",
	"給与",
	"雑収入",
	"その他",
]

// 収入ドメインモデル - 振る舞いを持つクラス
export class Income {
	constructor(
		public readonly id: string,
		public readonly date: Date,
		public readonly amount: number,
		public readonly description: string,
		public readonly category: IncomeCategory,
		public readonly createdAt: Date,
		public readonly updatedAt: Date
	) {}

	// ファクトリーメソッド：新規作成
	static create(data: CreateIncomeData): Income {
		const now = new Date()
		return new Income(
			"", // IDはPrismaで生成
			data.date,
			data.amount,
			data.description,
			data.category,
			now,
			now
		)
	}

	// ファクトリーメソッド：データベースから復元
	static fromData(data: IncomeData): Income {
		return new Income(
			data.id,
			data.date,
			data.amount,
			data.description,
			data.category,
			data.createdAt,
			data.updatedAt
		)
	}

	// 金額をフォーマット
	formatAmount(): string {
		return new Intl.NumberFormat("ja-JP", {
			style: "currency",
			currency: "JPY",
		}).format(this.amount)
	}

	// 日付をフォーマット
	formatDate(): string {
		return this.date.toLocaleDateString("ja-JP")
	}

	// バリデーション
	validate(): string[] {
		const errors: string[] = []

		if (this.amount <= 0) {
			errors.push("金額は1円以上である必要があります")
		}

		if (!this.description || this.description.trim().length === 0) {
			errors.push("説明は必須です")
		}

		if (this.description && this.description.length > 500) {
			errors.push("説明は500文字以下である必要があります")
		}

		if (!INCOME_CATEGORIES.includes(this.category)) {
			errors.push("無効なカテゴリです")
		}

		if (this.date > new Date()) {
			errors.push("日付は未来の日付にできません")
		}

		return errors
	}

	// 有効かどうかを判定
	isValid(): boolean {
		return this.validate().length === 0
	}

	// 更新されたインスタンスを返す（イミュータブル）
	update(data: Partial<CreateIncomeData>): Income {
		return new Income(
			this.id,
			data.date ?? this.date,
			data.amount ?? this.amount,
			data.description ?? this.description,
			data.category ?? this.category,
			this.createdAt,
			new Date() // 更新日時は現在時刻
		)
	}

	// 同じ収入かどうかを判定
	equals(other: Income): boolean {
		return this.id === other.id
	}

	// プレーンオブジェクトに変換（Prisma用）
	toData(): Omit<IncomeData, "id" | "createdAt" | "updatedAt"> {
		return {
			date: this.date,
			amount: this.amount,
			description: this.description,
			category: this.category,
		}
	}
}

// バリデーション関数（単体でも使用可能）
export function validateCreateIncomeData(data: CreateIncomeData): string[] {
	const income = Income.create(data)
	return income.validate()
}
