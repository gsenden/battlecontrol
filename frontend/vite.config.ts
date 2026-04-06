import { readFileSync } from 'node:fs';
import { fileURLToPath, URL } from 'node:url';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';
import { defineConfig } from 'vite';

const appVersion = readFileSync(
	fileURLToPath(new URL('../VERSION', import.meta.url)),
	'utf-8',
).trim();

export default defineConfig({
	define: {
		__APP_VERSION__: JSON.stringify(appVersion),
	},
	plugins: [tailwindcss(), sveltekit(), wasm()],
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
		fs: {
			allow: ['..'],
		},
		proxy: {
			'/auth': 'http://localhost:3000',
		},
	},
	worker: {
		plugins: () => [wasm()],
	},
});
