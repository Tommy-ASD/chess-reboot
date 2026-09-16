// src/app/prefs.ts
//
// Player preferences that aren't appearance: identity and board behaviour.
// All persisted; all editable in Settings.

import { effect, signal } from "@preact/signals";
import { isBoolean, persistedSignal } from "../lib/persisted";
import { getName, setName } from "../api/player";

/** Display name, mirrored into `player.ts` (which owns the request headers). */
export const displayName = signal<string>(getName());
effect(() => {
  if (displayName.value.trim() !== getName()) setName(displayName.value);
});

/** Show a–h / 1–8 around 8×8 boards. */
export const showCoordinates = persistedSignal("chess2.pref.coordinates", true, isBoolean);

/** Dot the squares a selected piece can move to. */
export const showMoveHints = persistedSignal("chess2.pref.move-hints", true, isBoolean);

/** Same-device games: turn the board to face whoever is to move. */
export const autoFlipLocal = persistedSignal("chess2.pref.auto-flip", false, isBoolean);
