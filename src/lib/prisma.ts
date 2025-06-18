import path from "node:path"
import { PrismaClient } from "@prisma/client"
import { app } from "electron"
import { createLogger } from "./logger"

const logger = createLogger("prisma")

// Electronアプリ用のデータベースパス設定
function getDatabaseUrl(): string {
	try {
		// Electronのユーザーデータディレクトリにデータベースを配置
		const userDataPath = app.getPath("userData")
		const dbPath = path.join(userDataPath, "accounting.db")
		return `file:${dbPath}`
	} catch (_error) {
		// Electronが利用できない場合（開発時など）
		logger.warn("Electron app not available, using default database path")
		return process.env.DATABASE_URL || "file:./dev.db"
	}
}

// Prismaクライアントのシングルトン
const globalForPrisma = globalThis as unknown as {
	prisma: PrismaClient | undefined
}

// データベースURLを動的に設定
const databaseUrl = getDatabaseUrl()
logger.info("Database URL:", databaseUrl)

export const prisma =
	globalForPrisma.prisma ??
	new PrismaClient({
		datasources: {
			db: {
				url: databaseUrl,
			},
		},
		log: ["info", "warn", "error"],
	})

if (process.env.NODE_ENV !== "production") {
	globalForPrisma.prisma = prisma
}

// 接続テスト
prisma
	.$connect()
	.then(() => {
		logger.info("Database connected successfully")
	})
	.catch((error) => {
		logger.error("Failed to connect to database", error)
	})

// アプリ終了時にクリーンアップ
process.on("beforeExit", async () => {
	await prisma.$disconnect()
	logger.info("Database disconnected")
})
