import { describe, expect, it } from "vitest"
import {
	amountForCommand,
	formatAmount,
	parseAmount,
	parseSignedAmount,
	signedAmountForCommand,
} from "./amount"

describe("金額", () => {
	it("3 桁ごとに区切る", () => {
		expect(formatAmount(1234567)).toBe("1,234,567")
		expect(formatAmount(-1200)).toBe("-1,200")
	})

	it("カンマ・円記号・全角の数字を読める", () => {
		expect(parseAmount("1,200")).toBe(1200)
		expect(parseAmount("￥１２００円")).toBe(1200)
		expect(parseAmount(" 30 000 ")).toBe(30000)
		expect(parseAmount("")).toBeNull()
	})

	it("整数でないものは NaN にし、Rust には -1 を送る", () => {
		expect(parseAmount("1.5")).toBeNaN()
		expect(parseAmount("-100")).toBeNaN()
		expect(amountForCommand("abc")).toBe(-1)
		expect(amountForCommand("")).toBe(-1)
		expect(amountForCommand("1,000")).toBe(1000)
	})

	it("丸められる桁数や無限大になる金額はコマンドに渡さない", () => {
		expect(parseAmount("9007199254740993")).toBeNaN()
		expect(amountForCommand("9".repeat(400))).toBe(-1)
	})
})

describe("期首残高の金額", () => {
	it("マイナスの残高を読み、不正な入力は有効な -1 円と区別する", () => {
		expect(parseSignedAmount("−100")).toBeNaN()
		expect(parseSignedAmount("－１，２００円")).toBe(-1200)
		expect(signedAmountForCommand("-1")).toBe(-1)
		expect(signedAmountForCommand("abc")).toBeGreaterThan(999_999_999_999)
	})
})
