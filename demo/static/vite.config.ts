import { cloudflare } from "@cloudflare/vite-plugin";
import { defineConfig } from "vite";

// tanya(...) needs shared memory, which browsers only give to isolated pages. In production
// public/_headers does this; this is the same for the local server.
const isolated = {
	"Cross-Origin-Opener-Policy": "same-origin",
	"Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
	plugins: [cloudflare()],
	server: { headers: isolated },
	preview: { headers: isolated },
	build: {
		// two pages: the playground and the tutorial
		rollupOptions: {
			input: { main: "index.html", belajar: "belajar.html", guru: "guru.html" },
		},
	},
});
