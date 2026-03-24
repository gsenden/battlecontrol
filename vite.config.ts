import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig(({ mode }) => ({
  base: mode === 'pages' ? '/battlecontrol/' : '/',
  plugins: [tailwindcss(), svelte()],
}));
