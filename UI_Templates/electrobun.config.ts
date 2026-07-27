import type { ElectrobunConfig } from "electrobun";

export default {
	app: {
		name: "UI_Templates",
		identifier: "reacttailwindvite.electrobun.dev",
		version: "0.0.1",
	},
	build: {
		bun: {
			entrypoint: "src/bun/index.ts",
			external: ["serialport"],
		},
		// Vite builds to dist/, we copy from there
		copy: {
			"dist/index.html": "views/mainview/index.html",
			"dist/assets": "views/mainview/assets",
			"node_modules/serialport":
				"app/node_modules/serialport",

			"node_modules/@serialport":
				"app/node_modules/@serialport",

		},
		// external: ["serialport"],
		// Ignore Vite output in watch mode — HMR handles view rebuilds separately
		watchIgnore: ["dist/**"],
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
