// src/pages/SettingsPage.tsx
//
// Profile, appearance (theme picker, colour customizer, custom CSS) and
// board behaviour. Everything applies instantly and persists locally.

import { useEffect, useMemo, useState } from "preact/hooks";
import { Board } from "../components/board/Board";
import { Toggle } from "../components/ui/Controls";
import { autoFlipLocal, displayName, showCoordinates, showMoveHints } from "../app/prefs";
import { parseFEN } from "../core/fen";
import { THEMES, SYSTEM_THEMES, findTheme, type ThemeManifest } from "../theme/manifest";
import {
  SYSTEM_CHOICE,
  activeTheme,
  customCss,
  customCssEnabled,
  isThemeModified,
  resetTheme,
  resetThemeVar,
  setThemeVar,
  themeChoice,
  themeOverrides,
} from "../theme/store";
import { THEME_VAR_GROUPS, readCssVar, toHex, type VarMeta } from "../theme/vars";

const PREVIEW_FEN = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - tr=full p=0";

export function SettingsPage() {
  return (
    <div class="container page settings">
      <header class="page-head">
        <h1 class="page-title">Settings</h1>
        <p class="page-lede">Make Chess 2 yours. Everything here saves on this device and applies instantly.</p>
      </header>

      <ProfileSection />
      <ThemeSection />
      <CustomizeSection />
      <CustomCssSection />
      <BoardSection />
    </div>
  );
}

function ProfileSection() {
  return (
    <section class="panel settings-section" id="profile">
      <h2 class="panel-title">Profile</h2>
      <label class="field">
        <span class="field-label">Display name</span>
        <input
          class="input"
          value={displayName.value}
          maxLength={24}
          placeholder="Anonymous"
          autoComplete="nickname"
          onInput={(e) => (displayName.value = e.currentTarget.value)}
        />
      </label>
      <p class="muted small">Shown to opponents and spectators. No account needed; accounts are coming later.</p>
    </section>
  );
}

function ThemeSection() {
  const choice = themeChoice.value;
  // Subscribe to overrides so "Modified" badges update.
  void themeOverrides.value;
  const light = findTheme(SYSTEM_THEMES.light)!;
  const dark = findTheme(SYSTEM_THEMES.dark)!;

  return (
    <section class="panel settings-section" id="themes">
      <h2 class="panel-title">Theme</h2>
      <div class="theme-grid" role="radiogroup" aria-label="Theme">
        <button
          type="button"
          role="radio"
          aria-checked={choice === SYSTEM_CHOICE}
          class={`theme-card${choice === SYSTEM_CHOICE ? " is-active" : ""}`}
          onClick={() => (themeChoice.value = SYSTEM_CHOICE)}
        >
          <span class="theme-swatch theme-swatch-split">
            <ThemeSwatch theme={light} />
            <ThemeSwatch theme={dark} />
          </span>
          <span class="theme-card-name">Match system</span>
          <span class="muted small">
            {light.name} by day, {dark.name} by night
          </span>
        </button>
        {THEMES.map((t) => (
          <button
            key={t.id}
            type="button"
            role="radio"
            aria-checked={choice === t.id}
            class={`theme-card${choice === t.id ? " is-active" : ""}`}
            onClick={() => (themeChoice.value = t.id)}
          >
            <span class="theme-swatch">
              <ThemeSwatch theme={t} />
            </span>
            <span class="theme-card-name">
              {t.name}
              {isThemeModified(t.id) && <span class="chip chip-soft">Modified</span>}
            </span>
            <span class="muted small">{t.description}</span>
          </button>
        ))}
      </div>
      <p class="muted small">
        Want your own? A theme is a single CSS file. See <code>docs/THEMES.md</code> in the repository.
      </p>
    </section>
  );
}

function ThemeSwatch({ theme }: { theme: ThemeManifest }) {
  const [bg, accent, secondary] = theme.colors;
  const [light, dark] = theme.board;
  return (
    <span class="swatch" style={{ background: bg }}>
      <span class="swatch-board" style={{ background: `repeating-conic-gradient(${dark} 0 25%, ${light} 0 50%) 0 0 / 50% 50%` }} />
      <span class="swatch-dots">
        <span style={{ background: accent }} />
        <span style={{ background: secondary }} />
      </span>
    </span>
  );
}

function CustomizeSection() {
  const theme = activeTheme.value;
  const overrides = themeOverrides.value[theme.id] ?? {};
  // Computed values only exist once the theme stylesheet has applied; re-read
  // shortly after a theme switch.
  const [readKey, setReadKey] = useState(0);
  useEffect(() => {
    const t = setTimeout(() => setReadKey((k) => k + 1), 80);
    return () => clearTimeout(t);
  }, [theme.id]);
  const squares = useMemo(() => parseFEN(PREVIEW_FEN), []);
  const modified = Object.keys(overrides).length > 0;

  return (
    <section class="panel settings-section" id="customize">
      <div class="settings-section-head">
        <div>
          <h2 class="panel-title">Customize {theme.name}</h2>
          <p class="muted small">Tweaks belong to this theme only. Switch themes and each keeps its own.</p>
        </div>
        <button type="button" class="btn btn-ghost btn-sm" disabled={!modified} onClick={() => resetTheme(theme.id)}>
          Reset all
        </button>
      </div>
      <div class="customize-layout">
        <div class="customize-groups">
          {THEME_VAR_GROUPS.map((group) => (
            <fieldset key={group.label} class="var-group">
              <legend class="field-label">{group.label}</legend>
              {group.vars.map((v) => (
                <VarRow key={v.name} meta={v} themeId={theme.id} override={overrides[v.name]} readKey={readKey} />
              ))}
            </fieldset>
          ))}
        </div>
        <div class="customize-preview">
          <Board squares={squares} coords={showCoordinates.value} selected={{ file: 5, rank: 5 }} targets={[{ target: { file: 6, rank: 3 }, kind: "move" }, { target: { file: 7, rank: 4 }, kind: "move" }]} lastMove={{ from: { file: 5, rank: 6 }, to: { file: 5, rank: 5 } }} label="Theme preview" />
          <div class="button-row">
            <button type="button" class="btn btn-primary">Primary</button>
            <button type="button" class="btn btn-secondary">Secondary</button>
            <span class="chip chip-warn">Chip</span>
          </div>
        </div>
      </div>
    </section>
  );
}

function VarRow({ meta, themeId, override, readKey }: { meta: VarMeta; themeId: string; override?: string; readKey: number }) {
  const [live, setLive] = useState("");
  useEffect(() => {
    setLive(readCssVar(meta.name));
  }, [readKey, override, meta.name]);
  const value = override ?? live;
  const hex = toHex(value);

  return (
    <div class={`var-row${override !== undefined ? " is-overridden" : ""}`}>
      <span class="var-label">{meta.label}</span>
      {hex ? (
        <input type="color" class="var-color" value={hex} aria-label={`${meta.label} colour`} onInput={(e) => setThemeVar(themeId, meta.name, e.currentTarget.value)} />
      ) : (
        <span class="var-color var-color-static" style={{ background: value }} aria-hidden="true" />
      )}
      <input
        class="input input-mono input-sm"
        value={value}
        aria-label={`${meta.label} value`}
        onChange={(e) => {
          const v = e.currentTarget.value.trim();
          if (v) setThemeVar(themeId, meta.name, v);
          else resetThemeVar(themeId, meta.name);
        }}
      />
      <button
        type="button"
        class="btn btn-ghost btn-icon"
        title="Reset to theme default"
        aria-label={`Reset ${meta.label}`}
        disabled={override === undefined}
        onClick={() => resetThemeVar(themeId, meta.name)}
      >
        ↺
      </button>
    </div>
  );
}

function CustomCssSection() {
  const [draft, setDraft] = useState(customCss.value);
  const dirty = draft !== customCss.value;
  return (
    <section class="panel settings-section" id="custom-css">
      <div class="settings-section-head">
        <div>
          <h2 class="panel-title">Custom CSS</h2>
          <p class="muted small">
            Applied after the theme and your colour tweaks, so it wins. Every colour, font and radius is a CSS variable (e.g.{" "}
            <code>--board-light</code>, <code>--font-display</code>).
          </p>
        </div>
        <Toggle checked={customCssEnabled.value} onChange={(v) => (customCssEnabled.value = v)} label="Enabled" />
      </div>
      <textarea
        class="input input-mono code-editor"
        rows={8}
        spellcheck={false}
        placeholder={":root {\n  --radius: 2px;\n}\n.board-frame { border-radius: 0; }"}
        value={draft}
        onInput={(e) => setDraft(e.currentTarget.value)}
      />
      <div class="button-row">
        <button type="button" class="btn btn-primary" disabled={!dirty} onClick={() => (customCss.value = draft)}>
          Apply
        </button>
        <button type="button" class="btn btn-ghost" disabled={!dirty} onClick={() => setDraft(customCss.value)}>
          Discard changes
        </button>
      </div>
    </section>
  );
}

function BoardSection() {
  return (
    <section class="panel settings-section" id="board">
      <h2 class="panel-title">Board</h2>
      <Toggle checked={showCoordinates.value} onChange={(v) => (showCoordinates.value = v)} label="Coordinates" description="Show a–h and 1–8 along the edges of standard boards." />
      <Toggle checked={showMoveHints.value} onChange={(v) => (showMoveHints.value = v)} label="Move hints" description="Mark the squares a selected piece can move to." />
      <Toggle checked={autoFlipLocal.value} onChange={(v) => (autoFlipLocal.value = v)} label="Auto-flip on this device" description="In same-device games, turn the board to face whoever is to move." />
    </section>
  );
}
