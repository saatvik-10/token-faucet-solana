import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills'

// https://vite.dev/config/
export default defineConfig({
  plugins: [tailwindcss(), nodePolyfills()],
  optimizeDeps: {
    include: ['@coral-xyz/borsh']
  }
});
