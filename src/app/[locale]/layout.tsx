import type { Metadata } from "next"
import { notFound } from "next/navigation"
import { NextIntlClientProvider } from "next-intl"
import { getMessages } from "next-intl/server"
import { locales } from "@/i18n/request"
import "../globals.css"

export const metadata: Metadata = {
	title: "会計アプリ",
	description: "個人事業主向けの確定申告用会計ソフト",
}

interface Props {
	children: React.ReactNode
	params: Promise<{ locale: string }>
}

export default async function LocaleLayout({ children, params }: Props) {
	const { locale } = await params

	if (!locales.includes(locale as (typeof locales)[number])) {
		notFound()
	}

	const messages = await getMessages()

	return (
		<html lang={locale}>
			<body>
				<NextIntlClientProvider messages={messages}>
					{children}
				</NextIntlClientProvider>
			</body>
		</html>
	)
}

export function generateStaticParams() {
	return locales.map((locale) => ({ locale }))
}
