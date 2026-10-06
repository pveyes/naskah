import { cloudflare } from "@cloudflare/vite-plugin";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [cloudflare()],
	build: {
		// two pages: the playground and the tutorial
		rollupOptions: {
			input: { main: "index.html", belajar: "belajar.html", guru: "guru.html" },
		},
	},
});
