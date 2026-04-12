import { readFileSync } from 'node:fs';
import { fileURLToPath, URL } from 'node:url';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';
import { defineConfig } from 'vite';
import { generateI18n, getI18nWatchFiles } from './scripts/i18n-generator.js';

const appVersion = readFileSync(
	fileURLToPath(new URL('../VERSION', import.meta.url)),
	'utf-8',
).trim();

function sharedI18nPlugin() {
	let generatedOnce = false;

	function regenerate() {
		const result = generateI18n();
		if (!generatedOnce) {
			generatedOnce = true;
			console.log(`Generated ${result.outFile} with ${result.keys.length} keys in ${result.langCodes.length} languages`);
		}
	}

	return {
		name: 'battlecontrol-shared-i18n',
		buildStart() {
			regenerate();
			for (const file of getI18nWatchFiles()) {
				this.addWatchFile(file);
			}
		},
		configureServer(server: import('vite').ViteDevServer) {
			const watchFiles = getI18nWatchFiles();
			const onChange = (changedFile: string) => {
				if (!watchFiles.includes(changedFile)) {
					return;
				}
				regenerate();
				server.ws.send({ type: 'full-reload' });
			};

			server.watcher.add(watchFiles);
			server.watcher.on('change', onChange);

			return () => {
				server.watcher.off('change', onChange);
			};
		},
	};
}

export default defineConfig({
	define: {
		__APP_VERSION__: JSON.stringify(appVersion),
	},
	plugins: [sharedI18nPlugin(), tailwindcss(), sveltekit(), wasm()],
	resolve: {
		alias: {
			'game-logic-wasm': fileURLToPath(
				new URL('../pkg/game_logic_wasm.js', import.meta.url),
			),
		},
	},
	optimizeDeps: {
		exclude: ['game-logic-wasm'],
	},
	server: {
		strictPort: true,
		fs: {
			allow: ['..'],
		},
		proxy: {
			'/auth': 'http://127.0.0.1:3000',
			'/games': 'http://127.0.0.1:3000',
			'/uploads': 'http://127.0.0.1:3000',
		},
	},
	worker: {
		plugins: () => [wasm()],
	},
});
