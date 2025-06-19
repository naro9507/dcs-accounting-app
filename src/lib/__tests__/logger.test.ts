import { describe, expect, it } from "vitest"
import { logger } from "@/lib/logger"

describe("Logger", () => {
	it("should be defined", () => {
		expect(logger).toBeDefined()
	})

	it("should have info method", () => {
		expect(typeof logger.info).toBe("function")
	})
})
