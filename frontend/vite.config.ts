import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react'

const backendHost = process.env.CHAT_ENV === 'prod' ? 'chatservice-backend' : 'localhost';

// TODO: Use YAML
export default defineConfig({
  plugins: [
    react(),
  ],
  server: {
    proxy: {
      // Forward REST API requests to backend server
      '/api': {
        target: `http://${backendHost}:9000`,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      // Expose backend WebSocket port through frontend
      '/ws': {
        target: `ws://${backendHost}:9001`,
        ws: true,
      }
    }
  }
});
