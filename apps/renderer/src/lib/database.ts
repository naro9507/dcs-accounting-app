import Database from "@tauri-apps/plugin-sql"
import { Store } from "@tauri-apps/plugin-store"
import { FileEncryption } from "@/lib/security"
import { createLogger } from "@/lib/logger"

const logger = createLogger("database")

const DB_URL = "sqlite:accounting.db"

let _db: Database | null = null

export async function getDatabase(): Promise<Database> {
	if (!_db) {
		logger.info("Connecting to database", { url: DB_URL })
		_db = await Database.load(DB_URL)
	}
	return _db
}

export async function closeDatabase(): Promise<void> {
	if (_db) {
		logger.info("Closing database connection")
		_db = null
	}
}

export async function regenerateEncryptionKey(): Promise<string> {
	logger.info("Regenerating encryption key")
	const store = await Store.load("security-store.json")
	await store.delete("master_key")
	await store.save()
	return "Key regenerated — reload app to apply"
}

export async function insertEncryptedData(
	table: string,
	data: Record<string, unknown>,
): Promise<void> {
	const db = await getDatabase()
	const encryption = FileEncryption.getInstance()

	const encryptedData: Record<string, unknown> = {}
	for (const [key, value] of Object.entries(data)) {
		if (
			typeof value === "string" &&
			(key === "description" || key === "notes")
		) {
			const payload = await encryption.encrypt(value)
			encryptedData[key] = JSON.stringify(payload)
		} else {
			encryptedData[key] = value
		}
	}

	const columns = Object.keys(encryptedData).join(", ")
	const placeholders = Object.keys(encryptedData)
		.map((_, i) => `$${i + 1}`)
		.join(", ")
	const values = Object.values(encryptedData)

	await db.execute(
		`INSERT INTO ${table} (${columns}) VALUES (${placeholders})`,
		values,
	)
	logger.debug("Encrypted data inserted", { table })
}

export async function getEncryptedData(
	table: string,
	where?: string,
	params?: unknown[],
): Promise<Record<string, unknown>[]> {
	const db = await getDatabase()
	const encryption = FileEncryption.getInstance()

	let query = `SELECT * FROM ${table}`
	if (where) query += ` WHERE ${where}`

	const rows: Record<string, unknown>[] = await db.select(query, params ?? [])

	return Promise.all(
		rows.map(async (row) => {
			const decryptedRow = { ...row }
			for (const [key, value] of Object.entries(row)) {
				if (
					typeof value === "string" &&
					(key === "description" || key === "notes")
				) {
					try {
						const payload = JSON.parse(value)
						if (payload.encrypted && payload.iv !== undefined) {
							decryptedRow[key] = await encryption.decrypt(
								payload.encrypted,
								payload.iv,
								payload.tag ?? "",
							)
						}
					} catch {
						logger.debug("Data not encrypted, using as-is", { key })
					}
				}
			}
			return decryptedRow
		}),
	)
}

export async function verifyDatabaseIntegrity(): Promise<boolean> {
	try {
		const db = await getDatabase()
		const result: Array<{ integrity_check: string }> = await db.select(
			"PRAGMA integrity_check",
		)
		logger.info("Database integrity check result", result)
		return result[0]?.integrity_check === "ok"
	} catch (error) {
		logger.error("Database integrity check failed", error)
		return false
	}
}
