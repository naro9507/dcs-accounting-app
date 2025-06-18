import { useTranslations } from "next-intl"

export default function Home() {
	const t = useTranslations("app")

	return (
		<main className="flex min-h-screen flex-col items-center justify-center p-24">
			<div className="z-10 max-w-5xl w-full items-center justify-center font-mono text-sm lg:flex">
				<h1 className="text-4xl font-bold text-center">{t("title")}</h1>
			</div>
			<div className="mt-8 text-center">
				<p className="text-lg">{t("description")}</p>
			</div>
		</main>
	)
}
