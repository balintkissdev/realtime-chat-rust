import { defineConfig } from 'vite';

// TODO: Use YAML
export default defineConfig({
  server: {
    proxy: {
      // Forward REST API requests to backend server
      '/api': {
        target: 'http://localhost:9000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      // Expose backend WebSocket port through frontend
      '/ws': {
        target: 'ws://localhost:9001',
        ws: true,
      }
    }
  }
});
