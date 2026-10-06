import { defineConfig } from "cf/config";

// A static site: no Worker code, just the files that Vite builds from this directory.
// It is served at https://naskah.<your-subdomain>.workers.dev
export default defineConfig({
	worker: {
		name: "naskah",
		compatibilityDate: "2026-10-01",
		assets: {},
	},
});
