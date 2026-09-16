import { defineConfig } from "vite";
import preact from "@preact/preset-vite";

// Single-page app: every route is served by index.html and resolved
// client-side by preact-iso (see src/app/App.tsx). Static hosts need the
// same fallback in production (`vite preview` already does it).
export default defineConfig({
  plugins: [preact()],
  appType: "spa",
});
