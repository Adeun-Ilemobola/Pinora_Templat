import type { ElectrobunConfig } from "electrobun";
import type { BunPlugin } from "bun";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = dirname(fileURLToPath(import.meta.url));
const tsconfig = fileURLToPath(new URL("./tsconfig.json", import.meta.url));

const aliases: Record<string, string> = {
	"@app": join(projectRoot, "src/mainview"),
	"@modules": join(projectRoot, "src/mainview/Modules"),
	"@runtime": join(projectRoot, "src/Runtime"),
	"@shared": join(projectRoot, "src/shared"),
	"@src": join(projectRoot, "src"),
};

const aliasPlugin: BunPlugin = {
	name: "pinora-path-aliases",
	setup(build) {
		build.onResolve(
			{ filter: /^@(app|modules|runtime|shared|src)\// },
			({ path }) => {
				const separator = path.indexOf("/");
				const alias = path.slice(0, separator);
				const target = join(aliases[alias], path.slice(separator + 1));

				return { path: Bun.resolveSync(target, projectRoot) };
			},
		);
	},
};

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
			plugins: [aliasPlugin],
			root: projectRoot,
			tsconfig,
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
