import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue({
      template: {
        compilerOptions: {
          // Performance optimizations - don't interfere with Element Plus
          whitespace: 'condense',
        },
      },
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2020',
    minify: 'esbuild',
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vue ecosystem
          if (id.includes('vue') || id.includes('vue-router') || id.includes('pinia')) {
            return 'vue-vendor'
          }

          // Element Plus UI
          if (id.includes('element-plus') || id.includes('@element-plus/')) {
            return 'element-ui'
          }

          // Charts and visualization
          if (id.includes('echarts')) {
            return 'charts'
          }

          // API and HTTP
          if (id.includes('axios')) {
            return 'api'
          }

          // Tauri APIs
          if (id.includes('@tauri-apps/')) {
            return 'tauri'
          }

          // Super Lotto specific code
          if (id.includes('super-lotto') || id.includes('superLotto')) {
            return 'super-lotto'
          }

          // Default
          return 'vendor'
        },
        // Optimize chunk size
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId ? chunkInfo.facadeModuleId.split('/').pop() : 'chunk'
          return `assets/js/${facadeModuleId}.[hash].js`
        },
        assetFileNames: (assetInfo) => {
          const info = assetInfo.name.split('.')
          const ext = info[info.length - 1]
          if (/\.(mp4|webm|ogg|mp3|wav|flac|aac)(\?.*)?$/i.test(assetInfo.name)) {
            return `assets/media/[name].[hash][extname]`
          }
          if (/\.(png|jpe?g|gif|svg)(\?.*)?$/i.test(assetInfo.name)) {
            return `assets/images/[name].[hash][extname]`
          }
          if (/\.(woff2?|eot|ttf|otf)(\?.*)?$/i.test(assetInfo.name)) {
            return `assets/fonts/[name].[hash][extname]`
          }
          return `assets/${ext}/[name].[hash][extname]`
        },
      },
    },
    // Enable source maps for debugging
    sourcemap: process.env.NODE_ENV === 'development',
    // Optimize chunks
    chunkSizeWarningLimit: 1000,
    // Set CSS code splitting
    cssCodeSplit: true,
  },
  server: {
    port: 1420,
    strictPort: true,
    // Enable HMR optimizations
    hmr: {
      overlay: false,
    },
    // Optimize dev server
    fs: {
      strict: false,
    },
  },
  // Performance optimizations
  optimizeDeps: {
    include: [
      'vue',
      'vue-router',
      'pinia',
      'element-plus',
      '@element-plus/icons-vue',
      'echarts',
      'vue-echarts',
      'axios',
      'lodash-es',
    ],
    exclude: ['@tauri-apps/api'],
  },
  // Environment variables
  envPrefix: ['VITE_', 'TAURI_'],
  clearScreen: false,
  // CSS optimizations
  css: {
    devSourcemap: false,
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
  // Experimental features for performance
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === 'js') {
        return { js: `/${filename}` }
      } else {
        return { relative: true }
      }
    },
  },
})