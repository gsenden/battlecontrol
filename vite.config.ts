import { fileURLToPath, URL } from 'node:url';
import { writeFile } from 'node:fs/promises';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';

const DEBUG_LOG_FILE = '/tmp/battlecontrol-debug.json';

export default defineConfig(({ mode }) => ({
  base: mode === 'pages' ? '/battlecontrol/' : '/',
  plugins: [
    tailwindcss(),
    svelte(),
    wasm(),
    {
      name: 'battlecontrol-debug-log',
      configureServer(server) {
        server.middlewares.use('/__debug-log', async (req, res) => {
          if (req.method !== 'POST') {
            res.statusCode = 405;
            res.end();
            return;
          }

          const chunks: Buffer[] = [];
          req.on('data', (chunk) => {
            chunks.push(Buffer.from(chunk));
          });
          req.on('end', async () => {
            await writeFile(DEBUG_LOG_FILE, Buffer.concat(chunks).toString('utf8'));
            res.statusCode = 204;
            res.end();
          });
        });
      },
    },
  ],
  resolve: {
    alias: {
      'game-logic-wasm': fileURLToPath(new URL('./pkg/game_logic_wasm.js', import.meta.url)),
    },
  },
}));
