import type { Metadata } from "next"
import "./globals.css"

export const metadata: Metadata = {
	title: "会計アプリ",
	description: "個人事業主向けの確定申告用会計ソフト",
}

export default function RootLayout({
	children,
}: {
	children: React.ReactNode
}) {
	return (
		<html lang="ja">
			<body>{children}</body>
		</html>
	)
}
