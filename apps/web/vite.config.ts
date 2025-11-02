import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 3001,
		strictPort: false, // Try next port if 3001 is taken
		proxy: {
			'/api': {
				target: 'http://localhost:3002',
				changeOrigin: true,
				// Retry failed proxy requests
				configure: (proxy, _options) => {
					proxy.on('error', (err, _req, res) => {
						console.error('Proxy error:', err.message);
						if (res.writeHead) {
							res.writeHead(503, {
								'Content-Type': 'application/json'
							});
							res.end(JSON.stringify({
								error: 'Backend service unavailable. Please ensure sync_service is running on port 3002.'
							}));
						}
					});
					proxy.on('proxyReq', (proxyReq, req) => {
						console.log(`[PROXY] ${req.method} ${req.url} -> ${proxyReq.path}`);
					});
				}
			},
			'/ws': {
				target: 'ws://localhost:3002',
				ws: true,
				changeOrigin: true
			}
		},
		hmr: {
			overlay: true, // Show errors in overlay
			clientPort: 3001 // Ensure HMR works correctly
		},
		// Improve dev server responsiveness
		watch: {
			usePolling: false, // Use native file watching
			ignored: ['**/node_modules/**', '**/.git/**', '**/dist/**', '**/build/**']
		}
	},
	optimizeDeps: {
		// Pre-bundle dependencies for faster dev server startup
		include: ['svelte', '@sveltejs/kit']
	},
	build: {
		sourcemap: true, // Enable source maps for debugging
		rollupOptions: {
			output: {
				manualChunks: {
					// Split vendor code for better caching
					'vendor': ['svelte', '@sveltejs/kit']
				}
			}
		}
	}
});