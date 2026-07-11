import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";

// Vite + Vitest configuration.
//
// `defineConfig` is imported from `vitest/config` (not `vite`) so the
// `test` block is typed — it re-exports Vite's config augmented with the
// Vitest options, keeping build and test settings in a single file.
export default defineConfig({
  // Multi-page app: every HTML entry point must be listed, otherwise
  // `vite build` treats only `index.html` as an entry and silently drops
  // the others from `dist/`. The dev server serves every .html file
  // regardless; this matters purely for the production build.
  //   home   → the welcoming landing page
  //   play   → the interactive local/online play surface (main.ts)
  //   fen    → the read-only FEN renderer (fen_page.ts)
  //   editor → the board editor (editor_page.ts)
  build: {
    rollupOptions: {
      input: {
        home: fileURLToPath(new URL("./index.html", import.meta.url)),
        play: fileURLToPath(new URL("./play.html", import.meta.url)),
        fen: fileURLToPath(new URL("./fen.html", import.meta.url)),
        editor: fileURLToPath(new URL("./editor.html", import.meta.url)),
      },
    },
  },

  // Bind the dev server to the port the `frontend` launch config expects,
  // so `npm run dev` alone lands on 5180 without needing the CLI flags.
  // `strictPort` fails loudly instead of silently hopping to 5181 when the
  // port is taken (a wrong-port preview is worse than an obvious error).
  server: {
    port: 5180,
    strictPort: true,
  },

  test: {
    // The suite deliberately runs on the plain Node environment — no jsdom.
    // The DOM-touching tests (e.g. player.test.ts) stub the specific globals
    // they need via `vi.stubGlobal`, which keeps the harness dependency-free.
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
