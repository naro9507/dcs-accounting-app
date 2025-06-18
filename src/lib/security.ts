import crypto from "node:crypto"
import path from "node:path"
import { app } from "electron"
import { createLogger } from "./logger"

const logger = createLogger("security")

const ALGORITHM = "aes-256-gcm"
const KEY_LENGTH = 32
const IV_LENGTH = 16
const _TAG_LENGTH = 16

// ファイル暗号化用のキー管理
export class FileEncryption {
	private static instance: FileEncryption
	private masterKey: Buffer | null = null

	private constructor() {}

	static getInstance(): FileEncryption {
		if (!FileEncryption.instance) {
			FileEncryption.instance = new FileEncryption()
		}
		return FileEncryption.instance
	}

	private getMasterKey(): Buffer {
		if (!this.masterKey) {
			const keyPath = path.join(app.getPath("userData"), ".master_key")
			const fs = require("node:fs")

			try {
				if (fs.existsSync(keyPath)) {
					this.masterKey = fs.readFileSync(keyPath)
					logger.debug("Master key loaded from file")
				} else {
					this.masterKey = crypto.randomBytes(KEY_LENGTH)
					fs.writeFileSync(keyPath, this.masterKey, { mode: 0o600 })
					logger.info("Generated new master key")
				}
			} catch (error) {
				logger.error("Failed to handle master key", error)
				// フォールバック: メモリ内でのみ使用するキー
				this.masterKey = crypto.randomBytes(KEY_LENGTH)
			}
		}
		if (!this.masterKey) {
			throw new Error("Master key is not initialized")
		}
		return this.masterKey
	}

	// データの暗号化
	encrypt(data: string): { encrypted: string; iv: string; tag: string } {
		try {
			const key = this.getMasterKey()
			const iv = crypto.randomBytes(IV_LENGTH)
			const cipher = crypto.createCipheriv(ALGORITHM, key, iv)
			cipher.setAAD(Buffer.from("dcs-accounting-app"))

			let encrypted = cipher.update(data, "utf8", "hex")
			encrypted += cipher.final("hex")

			const tag = cipher.getAuthTag()

			logger.debug("Data encrypted successfully")
			return {
				encrypted,
				iv: iv.toString("hex"),
				tag: tag.toString("hex"),
			}
		} catch (error) {
			logger.error("Encryption failed", error)
			throw new Error("Failed to encrypt data")
		}
	}

	// データの復号化
	decrypt(encrypted: string, iv: string, tag: string): string {
		try {
			const key = this.getMasterKey()
			const decipher = crypto.createDecipheriv(
				ALGORITHM,
				key,
				Buffer.from(iv, "hex")
			)
			decipher.setAAD(Buffer.from("dcs-accounting-app"))
			decipher.setAuthTag(Buffer.from(tag, "hex"))

			let decrypted = decipher.update(encrypted, "hex", "utf8")
			decrypted += decipher.final("utf8")

			logger.debug("Data decrypted successfully")
			return decrypted
		} catch (error) {
			logger.error("Decryption failed", error)
			throw new Error("Failed to decrypt data")
		}
	}

	// ファイルの暗号化
	encryptFile(filePath: string, outputPath: string): void {
		try {
			const fs = require("node:fs")
			const data = fs.readFileSync(filePath, "utf8")
			const { encrypted, iv, tag } = this.encrypt(data)

			const encryptedData = JSON.stringify({ encrypted, iv, tag })
			fs.writeFileSync(outputPath, encryptedData, { mode: 0o600 })

			logger.info("File encrypted successfully", { filePath, outputPath })
		} catch (error) {
			logger.error("File encryption failed", { filePath, error })
			throw new Error("Failed to encrypt file")
		}
	}

	// ファイルの復号化
	decryptFile(encryptedFilePath: string, outputPath: string): void {
		try {
			const fs = require("node:fs")
			const encryptedData = JSON.parse(
				fs.readFileSync(encryptedFilePath, "utf8")
			)
			const decrypted = this.decrypt(
				encryptedData.encrypted,
				encryptedData.iv,
				encryptedData.tag
			)

			fs.writeFileSync(outputPath, decrypted)

			logger.info("File decrypted successfully", {
				encryptedFilePath,
				outputPath,
			})
		} catch (error) {
			logger.error("File decryption failed", { encryptedFilePath, error })
			throw new Error("Failed to decrypt file")
		}
	}
}

// パスワードハッシュ化
export function hashPassword(
	password: string,
	salt?: string
): { hash: string; salt: string } {
	const actualSalt = salt || crypto.randomBytes(16).toString("hex")
	const hash = crypto
		.pbkdf2Sync(password, actualSalt, 100000, 32, "sha256")
		.toString("hex")

	return { hash, salt: actualSalt }
}

// パスワード検証
export function verifyPassword(
	password: string,
	hash: string,
	salt: string
): boolean {
	const { hash: computedHash } = hashPassword(password, salt)
	return computedHash === hash
}

// セキュアな一時ファイル作成
export function createSecureTempFile(prefix: string = "dcs_temp_"): string {
	const fs = require("node:fs")
	const os = require("node:os")

	const tempDir = os.tmpdir()
	const filename = prefix + crypto.randomBytes(8).toString("hex")
	const filePath = path.join(tempDir, filename)

	// 所有者のみ読み書き可能なファイルを作成
	fs.writeFileSync(filePath, "", { mode: 0o600 })

	logger.debug("Secure temp file created", { filePath })
	return filePath
}
