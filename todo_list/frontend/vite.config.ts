import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    // Для разработки оставляем прокси
    proxy: {
      "/todos": {
        target: "http://127.0.0.1:8080",
        changeOrigin: true,
      },
    },
  },
});