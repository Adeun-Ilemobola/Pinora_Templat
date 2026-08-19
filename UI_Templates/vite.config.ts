import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url"

import tailwindcss from "@tailwindcss/vite"

const fromProjectRoot = (path: string) =>
	fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
	plugins: [react() ,tailwindcss() ],
	root: "src/mainview",
	build: {
		outDir: "../../dist",
		emptyOutDir: true,
	},
	server: {
		port: 5173,
		strictPort: true,
	},
	resolve: {
		alias: {
			"@": fromProjectRoot("./src/mainview"),
			"@app": fromProjectRoot("./src/mainview"),
			"@modules": fromProjectRoot("./src/mainview/Modules"),
			"@src": fromProjectRoot("./src"),
		},
	},
});
