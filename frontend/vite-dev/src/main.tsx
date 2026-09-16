// src/main.tsx
//
// Boot: install the theme (and wait briefly for its stylesheet so the first
// paint is already themed), then mount the app.
//
// Dev: `npm run dev`. Production: `npm run build` (tsc + vite build).

import { render } from "preact";
import "./styles/base.css";
import { installTheme } from "./theme/apply";
import { App } from "./app/App";

installTheme().then(() => {
  render(<App />, document.getElementById("app")!);
});
