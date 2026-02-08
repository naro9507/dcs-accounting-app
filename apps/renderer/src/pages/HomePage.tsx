import { useTranslations } from "@/i18n/use-translations"

export function HomePage() {
	const t = useTranslations("app")

	return (
		<main className="flex min-h-screen flex-col items-center justify-center p-24">
			<div className="z-10 w-full max-w-5xl items-center justify-center font-mono text-sm lg:flex">
				<h1 className="text-center text-4xl font-bold">{t("title")}</h1>
			</div>
			<div className="mt-8 text-center">
				<p className="text-lg">{t("description")}</p>
			</div>
		</main>
	)
}
