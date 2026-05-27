type LogLevel = "debug" | "info" | "warn" | "error"

// Vite injects import.meta.env at build time; fallback for typecheck context
const isDev = (import.meta as { env?: { DEV?: boolean } }).env?.DEV ?? false

function makeLogger(name: string) {
	const prefix = `[${name}]`

	function log(level: LogLevel, msg: string, data?: unknown) {
		if (!isDev && level === "debug") return
		const args: unknown[] = [`${prefix} ${msg}`]
		if (data !== undefined) args.push(data)
		console[level](...args)
	}

	return {
		debug: (msg: string, data?: unknown) => log("debug", msg, data),
		info: (msg: string, data?: unknown) => log("info", msg, data),
		warn: (msg: string, data?: unknown) => log("warn", msg, data),
		error: (msg: string, data?: unknown) => log("error", msg, data),
	}
}

export const logger = makeLogger("app")
export const createLogger = (name: string) => makeLogger(name)
