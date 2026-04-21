// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: [
    vitePreprocess({ script: true }),
    {
      // Strip lang="ts" after TypeScript has been processed
      // Svelte 5 compiler doesn't support lang="ts" natively
      markup({ content, filename }) {
        if (filename && filename.includes('node_modules')) return;
        const processed = content
          .replace(/<script\s+lang="ts">/g, '<script>')
          .replace(/<script\s+lang="ts"\s+/g, '<script ')
          .replace(/<script\s+([^>]*)lang="ts"([^>]*)>/g, '<script $1$2>');
        if (processed !== content) {
          return { code: processed };
        }
      }
    }
  ],
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: '200.html',
      precompress: false,
      strict: true
    }),
  },
  compilerOptions: {},
};

export default config;
