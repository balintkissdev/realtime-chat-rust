import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react'

// TODO: Use YAML
export default defineConfig({
  plugins: [
    react(),
  ],
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
