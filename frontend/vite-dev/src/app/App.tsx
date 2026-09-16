// src/app/App.tsx
//
// Routes and the persistent shell. Every page is a route here; links are
// plain `<a href>` (preact-iso intercepts same-origin clicks).

import { useEffect } from "preact/hooks";
import { ErrorBoundary, LocationProvider, Route, Router, useLocation } from "preact-iso";
import { Toasts } from "../ui/toast";
import { TopBar, Footer } from "./Shell";
import { HomePage } from "../pages/HomePage";
import { PlaySetupPage } from "../pages/PlaySetupPage";
import { OnlineGamePage } from "../pages/OnlineGamePage";
import { LocalGamePage } from "../pages/LocalGamePage";
import { PiecesPage } from "../pages/PiecesPage";
import { PieceDetailPage } from "../pages/PieceDetailPage";
import { LabPage } from "../pages/LabPage";
import { SettingsPage } from "../pages/SettingsPage";
import { NotFoundPage } from "../pages/NotFoundPage";

export function App() {
  return (
    <LocationProvider>
      <LegacyRedirects />
      <ScrollToTop />
      <a class="skip-link" href="#main">Skip to content</a>
      <TopBar />
      <main id="main" class="main">
        <ErrorBoundary>
          <Router>
            <Route path="/" component={HomePage} />
            <Route path="/play" component={PlaySetupPage} />
            <Route path="/play/:variant" component={PlaySetupPage} />
            <Route path="/local" component={LocalGamePage} />
            <Route path="/local/:variant" component={LocalGamePage} />
            <Route path="/game/:id" component={OnlineGamePage} />
            <Route path="/pieces" component={PiecesPage} />
            <Route path="/pieces/:id" component={PieceDetailPage} />
            <Route path="/lab" component={LabPage} />
            <Route path="/settings" component={SettingsPage} />
            <Route default component={NotFoundPage} />
          </Router>
        </ErrorBoundary>
      </main>
      <Footer />
      <Toasts />
    </LocationProvider>
  );
}

/// Old URLs keep working: `/?game=<id>` / `?code=` share links from the
/// previous frontend, and the old editor page.
function LegacyRedirects() {
  const { path, query, route } = useLocation();
  useEffect(() => {
    const game = query.game ?? query.code;
    if (path === "/" && game) route(`/game/${encodeURIComponent(game)}`, true);
    else if (path === "/editor.html") route(query.fen ? `/lab?fen=${encodeURIComponent(query.fen)}` : "/lab", true);
  }, [path]);
  return null;
}

function ScrollToTop() {
  const { path } = useLocation();
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [path]);
  return null;
}
