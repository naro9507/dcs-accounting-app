import { readFile } from "@tauri-apps/plugin-fs"
import { appDataDir } from "@tauri-apps/api/path"
import { Store } from "@tauri-apps/plugin-store"
import { getDatabase } from "@/lib/database"
import { createLogger } from "@/lib/logger"

const logger = createLogger("migration")
const MIGRATION_FLAG = "v2_encryption_migrated"
const AAD = new TextEncoder().encode("dcs-accounting-app")

// IVが32文字のhex文字列（=16バイト）なら旧Electron形式と判定
function isOldFormat(iv: string): boolean {
	return /^[0-9a-f]{32}$/.test(iv)
}

async function importOldMasterKey(): Promise<CryptoKey | null> {
	try {
		const dir = await appDataDir()
		const keyPath = `${dir}/.master_key`
		const keyBytes = await readFile(keyPath)
		return await crypto.subtle.importKey(
			"raw",
			keyBytes,
			{ name: "AES-GCM" },
			false,
			["decrypt"],
		)
	} catch {
		return null
	}
}

// 旧Node.js AES-GCM形式（hex、16バイトIV、タグ別保存）をWeb Cryptoで復号
async function decryptOldFormat(
	encryptedHex: string,
	ivHex: string,
	tagHex: string,
	oldKey: CryptoKey,
): Promise<string> {
	const cipherBytes = Uint8Array.from(
		encryptedHex.match(/.{2}/g)!.map((b) => Number.parseInt(b, 16)),
	)
	const tagBytes = Uint8Array.from(
		tagHex.match(/.{2}/g)!.map((b) => Number.parseInt(b, 16)),
	)
	const iv = Uint8Array.from(
		ivHex.match(/.{2}/g)!.map((b) => Number.parseInt(b, 16)),
	)

	// Web CryptoはAES-GCMの暗号文末尾にタグが結合されていることを期待
	const combined = new Uint8Array(cipherBytes.length + tagBytes.length)
	combined.set(cipherBytes)
	combined.set(tagBytes, cipherBytes.length)

	const plainBuf = await crypto.subtle.decrypt(
		{ name: "AES-GCM", iv, additionalData: AAD },
		oldKey,
		combined,
	)
	return new TextDecoder().decode(plainBuf)
}

async function encryptNewFormat(
	plaintext: string,
	newKey: CryptoKey,
): Promise<{ encrypted: string; iv: string }> {
	const iv = crypto.getRandomValues(new Uint8Array(12))
	const cipherBuf = await crypto.subtle.encrypt(
		{ name: "AES-GCM", iv, additionalData: AAD },
		newKey,
		new TextEncoder().encode(plaintext),
	)
	return {
		encrypted: btoa(String.fromCharCode(...new Uint8Array(cipherBuf))),
		iv: btoa(String.fromCharCode(...iv)),
	}
}

// アプリ起動時に1回だけ呼ぶ
// newKey: security.ts の getMasterKey() で取得した新しいCryptoKey
export async function runMigrationIfNeeded(newKey: CryptoKey): Promise<void> {
	const store = await Store.load("security-store.json")

	const alreadyMigrated = await store.get<boolean>(MIGRATION_FLAG)
	if (alreadyMigrated) return

	logger.info("Starting encryption format migration from Electron to Tauri")

	const oldKey = await importOldMasterKey()
	if (!oldKey) {
		logger.info("No old master key found — marking as migrated")
		await store.set(MIGRATION_FLAG, true)
		await store.save()
		return
	}

	const db = await getDatabase()
	let migratedCount = 0

	for (const table of ["incomes", "expenses"]) {
		const rows: Record<string, unknown>[] = await db.select(
			`SELECT * FROM ${table}`,
		)
		for (const row of rows) {
			for (const field of ["description", "notes"]) {
				const raw = row[field]
				if (typeof raw !== "string") continue
				try {
					const payload = JSON.parse(raw)
					if (!isOldFormat(String(payload.iv))) continue

					const plaintext = await decryptOldFormat(
						payload.encrypted,
						payload.iv,
						payload.tag,
						oldKey,
					)
					const { encrypted, iv } = await encryptNewFormat(plaintext, newKey)
					const newPayload = JSON.stringify({ encrypted, iv, tag: "" })
					await db.execute(
						`UPDATE ${table} SET ${field} = $1 WHERE id = $2`,
						[newPayload, row.id],
					)
					migratedCount++
				} catch {
					// 復号失敗は無視（別形式 or 暗号化なし）
				}
			}
		}
	}

	logger.info("Migration complete", { migratedCount })
	await store.set(MIGRATION_FLAG, true)
	await store.save()
}
