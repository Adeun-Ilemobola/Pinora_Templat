import type { ElectrobunConfig } from "electrobun";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = dirname(fileURLToPath(import.meta.url));

export default {
    app: {
        name: "UI_Templates",
        identifier: "reacttailwindvite.electrobun.dev",
        version: "0.0.1",
    },

    build: {
        mainProcess: "bun",

        bun: {
            entrypoint: "src/bun/index.ts",

            external: [
                "serialport",
            ],

            alias: {
                "@app": resolve(projectRoot, "src/mainview"),
                "@modules": resolve(projectRoot, "src/mainview/Modules"),
                "@src": resolve(projectRoot, "src"),
            },
        },

        copy: {
            "dist/index.html": "views/mainview/index.html",
            "dist/assets": "views/mainview/assets",
        },

        watchIgnore: [
            "dist/**",
        ],

        mac: {
            bundleCEF: false,
        },

        linux: {
            bundleCEF: false,
        },

        win: {
            bundleCEF: false,
        },
    },
} satisfies ElectrobunConfig;