import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig(({ mode }) => ({
  base: mode === 'pages' ? '/battlecontrol/' : '/',
  plugins: [tailwindcss(), svelte(), wasm()],
}));
