import { useEffect } from "react"
import { HashRouter, NavLink, Navigate, Route, Routes, useParams } from "react-router-dom"
import { defaultLocale, isLocale } from "@/i18n/i18n"
import { I18nProvider } from "@/i18n/i18n-context"
import { useTranslations } from "@/i18n/use-translations"
import { HomePage } from "@/pages/HomePage"
import { IncomePage } from "@/pages/income/IncomePage"
import { AdminApp } from "@/pages/admin/AdminApp"

function LocaleLayout() {
	const params = useParams()
	const locale = params.locale && isLocale(params.locale) ? params.locale : defaultLocale

	useEffect(() => {
		document.documentElement.lang = locale
	}, [locale])

	return (
		<I18nProvider locale={locale}>
			<div className="min-h-screen bg-slate-50 text-slate-900">
				<SiteNav locale={locale} />
				<Routes>
					<Route index element={<HomePage />} />
					<Route path="income" element={<IncomePage />} />
					<Route path="admin/*" element={<AdminApp />} />
					<Route path="*" element={<Navigate to={`/${locale}`} replace />} />
				</Routes>
			</div>
		</I18nProvider>
	)
}

function SiteNav({ locale }: { locale: string }) {
	const t = useTranslations("navigation")

	const linkClass = ({ isActive }: { isActive: boolean }) =>
		`rounded px-3 py-2 text-sm font-medium transition ${
			isActive ? "bg-slate-900 text-white" : "text-slate-700 hover:bg-slate-200"
		}`

	return (
		<header className="border-b border-slate-200 bg-white">
			<div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
				<span className="text-lg font-semibold">DCS会計</span>
				<nav className="flex gap-2">
					<NavLink to={`/${locale}`} className={linkClass} end>
						{t("dashboard")}
					</NavLink>
					<NavLink to={`/${locale}/income`} className={linkClass}>
						{t("income")}
					</NavLink>
					<NavLink to={`/${locale}/admin`} className={linkClass}>
						React Admin
					</NavLink>
				</nav>
			</div>
		</header>
	)
}

export function App() {
	return (
		<HashRouter>
			<Routes>
				<Route path="/" element={<Navigate to={`/${defaultLocale}`} replace />} />
				<Route path="/:locale/*" element={<LocaleLayout />} />
				<Route path="*" element={<Navigate to={`/${defaultLocale}`} replace />} />
			</Routes>
		</HashRouter>
	)
}
