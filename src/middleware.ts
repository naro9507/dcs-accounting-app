import createMiddleware from "next-intl/middleware"
import { locales } from "@/i18n/request"

export default createMiddleware({
	locales,
	defaultLocale: "ja",
})

export const config = {
	matcher: ["/((?!api|_next|_vercel|.*\\..*).*)"],
}
