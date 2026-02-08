import { useContext } from "react"
import { I18nContext } from "./i18n-context"
import type { Messages } from "./i18n"

export function useTranslations<N extends keyof Messages>(namespace: N) {
	const { messages } = useContext(I18nContext)

	return (key: keyof Messages[N]) => {
		const section = messages[namespace] as Record<string, string> | undefined
		const value = section?.[String(key)]
		return value ?? `${String(namespace)}.${String(key)}`
	}
}
