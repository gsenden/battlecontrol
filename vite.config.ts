import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig(({ mode }) => ({
  base: mode === 'pages' ? '/battlecontrol/' : '/',
  plugins: [tailwindcss(), svelte(), wasm()],
  resolve: {
    alias: {
      'game-logic-wasm': fileURLToPath(new URL('./pkg/game_logic_wasm.js', import.meta.url)),
    },
  },
}));
