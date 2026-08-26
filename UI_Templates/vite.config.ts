import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "node:path";
import { fileURLToPath, URL } from "node:url";

import { electrobunViteAliases } from "./.hutch/devkit/api/config/electrobun-vite";

const fromProjectRoot = (path: string) =>
    fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
    ],

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
        alias: [
            ...electrobunViteAliases(
                resolve(process.cwd(), ".hutch/devkit"),
            ),

            {
                find: "@",
                replacement: fromProjectRoot("./src/mainview"),
            },
            {
                find: "@app",
                replacement: fromProjectRoot("./src/mainview"),
            },
            {
                find: "@modules",
                replacement: fromProjectRoot("./src/mainview/Modules"),
            },
            {
                find: "@src",
                replacement: fromProjectRoot("./src"),
            },
        ],
    },
});