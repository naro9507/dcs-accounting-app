import { Store } from "@tauri-apps/plugin-store"

const MASTER_KEY_STORE = "security-store.json"
const MASTER_KEY_NAME = "master_key"
const AAD = new TextEncoder().encode("dcs-accounting-app")

let _store: Store | null = null
async function getSecurityStore(): Promise<Store> {
	if (!_store) _store = await Store.load(MASTER_KEY_STORE)
	return _store
}

let _cachedKey: CryptoKey | null = null

export async function getMasterKey(): Promise<CryptoKey> {
	if (_cachedKey) return _cachedKey

	const store = await getSecurityStore()
	const rawB64 = await store.get<string>(MASTER_KEY_NAME)

	if (!rawB64) {
		const key = await crypto.subtle.generateKey(
			{ name: "AES-GCM", length: 256 },
			true,
			["encrypt", "decrypt"],
		)
		const raw = await crypto.subtle.exportKey("raw", key)
		const b64 = btoa(String.fromCharCode(...new Uint8Array(raw)))
		await store.set(MASTER_KEY_NAME, b64)
		await store.save()
		_cachedKey = key
		return key
	}

	const rawBytes = Uint8Array.from(atob(rawB64), (c) => c.charCodeAt(0))
	_cachedKey = await crypto.subtle.importKey(
		"raw",
		rawBytes,
		{ name: "AES-GCM" },
		false,
		["encrypt", "decrypt"],
	)
	return _cachedKey
}

export interface EncryptedPayload {
	encrypted: string
	iv: string
	tag: string
}

export async function encrypt(plaintext: string): Promise<EncryptedPayload> {
	const key = await getMasterKey()
	const iv = crypto.getRandomValues(new Uint8Array(12))
	const cipherBuf = await crypto.subtle.encrypt(
		{ name: "AES-GCM", iv, additionalData: AAD },
		key,
		new TextEncoder().encode(plaintext),
	)
	return {
		encrypted: btoa(String.fromCharCode(...new Uint8Array(cipherBuf))),
		iv: btoa(String.fromCharCode(...iv)),
		tag: "",
	}
}

export async function decrypt(
	encryptedB64: string,
	ivB64: string,
	_tag: string,
): Promise<string> {
	const key = await getMasterKey()
	const cipherBuf = Uint8Array.from(atob(encryptedB64), (c) => c.charCodeAt(0))
	const iv = Uint8Array.from(atob(ivB64), (c) => c.charCodeAt(0))
	const plainBuf = await crypto.subtle.decrypt(
		{ name: "AES-GCM", iv, additionalData: AAD },
		key,
		cipherBuf,
	)
	return new TextDecoder().decode(plainBuf)
}

export async function hashPassword(
	password: string,
	saltHex?: string,
): Promise<{ hash: string; salt: string }> {
	const saltBytes = saltHex
		? Uint8Array.from(
				saltHex.match(/.{2}/g)!.map((b) => Number.parseInt(b, 16)),
			)
		: crypto.getRandomValues(new Uint8Array(16))

	const keyMaterial = await crypto.subtle.importKey(
		"raw",
		new TextEncoder().encode(password),
		"PBKDF2",
		false,
		["deriveBits"],
	)
	const derived = await crypto.subtle.deriveBits(
		{ name: "PBKDF2", salt: saltBytes, iterations: 100_000, hash: "SHA-256" },
		keyMaterial,
		256,
	)
	const hashHex = Array.from(new Uint8Array(derived))
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("")
	const saltHexOut = Array.from(saltBytes)
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("")
	return { hash: hashHex, salt: saltHexOut }
}

export async function verifyPassword(
	password: string,
	hash: string,
	salt: string,
): Promise<boolean> {
	const { hash: computed } = await hashPassword(password, salt)
	return computed === hash
}

// database.ts との後方互換を維持するクラス
export class FileEncryption {
	private static _instance: FileEncryption

	static getInstance(): FileEncryption {
		if (!FileEncryption._instance) {
			FileEncryption._instance = new FileEncryption()
		}
		return FileEncryption._instance
	}

	async encrypt(data: string): Promise<EncryptedPayload> {
		return encrypt(data)
	}

	async decrypt(enc: string, iv: string, tag: string): Promise<string> {
		return decrypt(enc, iv, tag)
	}
}
