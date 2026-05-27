import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import path from "node:path"

export default defineConfig({
	root: __dirname,
	plugins: [react()],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
		},
	},
	build: {
		outDir: "dist",
		emptyOutDir: true,
		target: ["es2021", "chrome105", "safari15"],
	},
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
	},
})
