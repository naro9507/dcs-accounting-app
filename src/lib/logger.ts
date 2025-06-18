import pino from "pino"
import pretty from "pino-pretty"

const isDevelopment = process.env.NODE_ENV === "development"

export const logger = pino(
	{
		level: isDevelopment ? "debug" : "info",
		timestamp: pino.stdTimeFunctions.isoTime,
		formatters: {
			level: (label) => ({ level: label.toUpperCase() }),
		},
	},
	isDevelopment
		? pretty({
				colorize: true,
				ignore: "pid,hostname",
				translateTime: "yyyy-mm-dd HH:MM:ss",
			})
		: undefined
)

export const createLogger = (name: string) => {
	return logger.child({ name })
}
