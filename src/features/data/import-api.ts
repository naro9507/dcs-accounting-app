import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { callCommand } from "@/lib/tauri"

export type ImportKind = "journalCsv" | "excelBook"

/** Rust 側 `commands::import::ChosenFile` と同じ形。 */
export type ChosenFile = {
	name: string
	kind: ImportKind
	suggestedYear: number | null
}

/** Rust 側 `import::IssueCode` と同じ文字列。 */
export type IssueCode =
	| "missingColumns"
	| "duplicateColumns"
	| "missingTransactionNo"
	| "inconsistentDate"
	| "monthLocked"
	| "invalidDate"
	| "outsideYear"
	| "invalidAmount"
	| "invalidSide"
	| "unknownAccount"
	| "unmappedAccount"
	| "inactiveMaster"
	| "unknownPartner"
	| "unknownSubAccount"
	| "invalidTaxCategory"
	| "unbalanced"
	| "sameAccount"
	| "missingSide"
	| "tooLong"

export type Issue = {
	sheet: string | null
	row: number
	code: IssueCode
	value?: string
}

export type BookAccount = {
	number: number
	nameJa: string
	nameEn: string
	suggestedAccountId: number | null
	debit: number
	credit: number
}

export type AccountMapping = { number: number; accountId: number }

export type ImportPreview = {
	fiscalYear: number | null
	createsFiscalYear: boolean
	ready: number
	duplicates: number
	issues: Issue[]
	issueCount: number
	totalDebit: number
	totalCredit: number
	accountTotals: { accountId: number; debit: number; credit: number }[]
	bookAccounts: BookAccount[]
	newPartners: { code: string; name: string }[]
}

export type ImportResult = {
	fiscalYear: number
	imported: number
	duplicates: number
}

/** 取り込むファイルを選ぶ。選ぶ画面は Rust 側で開き、場所は画面に渡さない。やめたら null。 */
export function useChooseImportFile() {
	return useMutation({
		mutationFn: (kind: ImportKind) =>
			callCommand<ChosenFile | null>("choose_import_file", { kind }),
	})
}

/** 取り込む前の確認。何も書き込まない。 */
export function useImportPreview(
	file: ChosenFile | null,
	fiscalYear: number | null,
	mapping: AccountMapping[]
) {
	return useQuery({
		queryKey: ["import-preview", file, fiscalYear, mapping],
		queryFn: () =>
			callCommand<ImportPreview>("preview_import", { fiscalYear, mapping }),
		enabled:
			file !== null && (file.kind === "journalCsv" || fiscalYear !== null),
		retry: false,
		gcTime: 0,
	})
}

/** 取り込む。取り込んだら、帳簿・集計・マスタ・年度を読み直す。 */
export function useRunImport() {
	const queryClient = useQueryClient()
	return useMutation({
		mutationFn: ({
			fiscalYear,
			mapping,
		}: {
			fiscalYear: number | null
			mapping: AccountMapping[]
		}) => callCommand<ImportResult>("run_import", { fiscalYear, mapping }),
		onSuccess: () => queryClient.invalidateQueries(),
	})
}
