import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path"
import { fileURLToPath, URL } from "node:url"

import tailwindcss from "@tailwindcss/vite"

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
     "@": fileURLToPath(new URL("./src/mainview", import.meta.url)),
    },
  },
});
