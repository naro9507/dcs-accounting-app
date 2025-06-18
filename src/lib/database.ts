import crypto from "node:crypto"
import path from "node:path"
import Database from "better-sqlite3"
import { app } from "electron"
import { createLogger } from "./logger"
import { FileEncryption } from "./security"

const logger = createLogger("database")

let dbInstance: Database.Database | null = null

export function getDatabase(): Database.Database {
	if (!dbInstance) {
		const dbPath = path.join(app.getPath("userData"), "accounting.db")

		logger.info("Initializing database with application-level encryption", {
			path: dbPath,
		})

		dbInstance = new Database(dbPath, {
			verbose: (message?: unknown) => logger.debug(String(message)),
		})

		initializeTables()
	}

	return dbInstance
}

function initializeTables() {
	if (!dbInstance) {
		throw new Error("Database instance is not initialized")
	}
	const db = dbInstance

	logger.info("Initializing database tables")

	// 収入テーブル
	db.exec(`
    CREATE TABLE IF NOT EXISTS income (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      date TEXT NOT NULL,
      amount INTEGER NOT NULL,
      description TEXT NOT NULL,
      category TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `)

	// 支出テーブル
	db.exec(`
    CREATE TABLE IF NOT EXISTS expense (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      date TEXT NOT NULL,
      amount INTEGER NOT NULL,
      description TEXT NOT NULL,
      category TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `)

	// 設定テーブル
	db.exec(`
    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )
  `)

	logger.info("Database tables initialized")
}

export function closeDatabase() {
	if (dbInstance) {
		logger.info("Closing database connection")
		dbInstance.close()
		dbInstance = null
	}
}

// 暗号化キーの再生成（セキュリティ用途）
export function regenerateEncryptionKey(): string {
	logger.info("Regenerating encryption key")
	const keyPath = path.join(app.getPath("userData"), ".db_key")
	const fs = require("node:fs")

	const newKey = crypto.randomBytes(32).toString("hex")
	try {
		fs.writeFileSync(keyPath, newKey, { mode: 0o600 })
		logger.info("Encryption key regenerated successfully")
	} catch (error) {
		logger.error("Failed to save new encryption key", error)
	}

	return newKey
}

// 暗号化されたデータの挿入
export function insertEncryptedData(
	table: string,
	data: Record<string, unknown>
): void {
	const db = getDatabase()
	const encryption = FileEncryption.getInstance()

	// 機密データを暗号化
	const encryptedData: Record<string, unknown> = {}
	for (const [key, value] of Object.entries(data)) {
		if (
			typeof value === "string" &&
			(key === "description" || key === "notes")
		) {
			const encrypted = encryption.encrypt(value)
			encryptedData[key] = JSON.stringify(encrypted)
		} else {
			encryptedData[key] = value
		}
	}

	const columns = Object.keys(encryptedData).join(", ")
	const placeholders = Object.keys(encryptedData)
		.map(() => "?")
		.join(", ")
	const values = Object.values(encryptedData)

	const stmt = db.prepare(
		`INSERT INTO ${table} (${columns}) VALUES (${placeholders})`
	)
	stmt.run(...values)

	logger.debug("Encrypted data inserted", { table })
}

// 暗号化されたデータの取得
export function getEncryptedData(
	table: string,
	where?: string,
	params?: unknown[]
): Record<string, unknown>[] {
	const db = getDatabase()
	const encryption = FileEncryption.getInstance()

	let query = `SELECT * FROM ${table}`
	if (where) {
		query += ` WHERE ${where}`
	}

	const stmt = db.prepare(query)
	const rows = params ? stmt.all(...params) : stmt.all()

	// 暗号化されたフィールドを復号化
	return (rows as Record<string, unknown>[]).map((row) => {
		const decryptedRow = { ...row }
		for (const [key, value] of Object.entries(row)) {
			if (
				typeof value === "string" &&
				(key === "description" || key === "notes")
			) {
				try {
					const encrypted = JSON.parse(value)
					if (encrypted.encrypted && encrypted.iv && encrypted.tag) {
						decryptedRow[key] = encryption.decrypt(
							encrypted.encrypted,
							encrypted.iv,
							encrypted.tag
						)
					}
				} catch (_error) {
					// 暗号化されていないデータの場合はそのまま
					logger.debug("Data not encrypted, using as-is", { key })
				}
			}
		}
		return decryptedRow
	})
}

// データベースの整合性チェック
export function verifyDatabaseIntegrity(): boolean {
	try {
		const db = getDatabase()
		const result = db.pragma("integrity_check") as Array<{
			integrity_check: string
		}>
		logger.info("Database integrity check result", result)
		return result[0]?.integrity_check === "ok"
	} catch (error) {
		logger.error("Database integrity check failed", error)
		return false
	}
}
