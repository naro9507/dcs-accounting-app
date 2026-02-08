import { createContext, useMemo } from "react"
import type { ReactNode } from "react"
import { defaultLocale, getMessages, isLocale } from "./i18n"
import type { Locale, Messages } from "./i18n"

type I18nContextValue = {
	locale: Locale
	messages: Messages
}

export const I18nContext = createContext<I18nContextValue>({
	locale: defaultLocale,
	messages: getMessages(defaultLocale),
})

type I18nProviderProps = {
	locale: string
	children: ReactNode
}

export function I18nProvider({ locale, children }: I18nProviderProps) {
	const normalizedLocale = isLocale(locale) ? locale : defaultLocale
	const value = useMemo<I18nContextValue>(
		() => ({
			locale: normalizedLocale,
			messages: getMessages(normalizedLocale),
		}),
		[normalizedLocale]
	)

	return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}
