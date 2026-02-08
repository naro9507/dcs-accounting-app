import ja from "./messages/ja.json"
import en from "./messages/en.json"

export const locales = ["ja", "en"] as const
export type Locale = (typeof locales)[number]

export const defaultLocale: Locale = "ja"

export type Messages = typeof ja

const messageMap: Record<Locale, Messages> = {
	ja,
	en,
}

export function isLocale(locale: string): locale is Locale {
	return (locales as readonly string[]).includes(locale)
}

export function getMessages(locale: Locale): Messages {
	return messageMap[locale]
}
