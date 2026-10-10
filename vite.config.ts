import {defineConfig} from 'vite';

export default defineConfig({
  server: {
    watch: {
      // Older snapshots and native build caches are not inputs to the app.
      ignored: ['**/archive/**', '**/engine/target/**', '**/.build-backups/**'],
    },
  },
});
