// src/components/game/useLocalGame.ts
//
// A game played on this device against the stateless engine API: the
// position history (so moves can be undone), the current status, and the
// move sink `useBoardPlay` submits to.

import { useEffect, useState } from "preact/hooks";
import { applyMove, fetchStatus } from "../../api/board";
import { formatLastMove } from "../../core/fen";
import type { GameMove, GameStatus } from "../../core/types";

interface Entry {
  fen: string;
  status: GameStatus;
}

export interface LocalGame {
  fen: string;
  status: GameStatus;
  /** One label per move played, oldest first. */
  moves: string[];
  canUndo: boolean;
  submit: (move: GameMove) => Promise<void>;
  undo: () => void;
  restart: () => void;
}

const ONGOING: GameStatus = { status: "Ongoing" };

export function useLocalGame(startFen: string): LocalGame {
  const [entries, setEntries] = useState<Entry[]>([{ fen: startFen, status: ONGOING }]);

  // A new starting position (different variant / demo) resets the game and
  // asks the engine whether that position is already decided.
  useEffect(() => {
    setEntries([{ fen: startFen, status: ONGOING }]);
    let alive = true;
    fetchStatus(startFen)
      .then((status) => {
        if (alive) setEntries((e) => (e.length === 1 && e[0].fen === startFen ? [{ fen: startFen, status }] : e));
      })
      .catch(() => {
        // Unreachable server surfaces on the first move attempt instead.
      });
    return () => {
      alive = false;
    };
  }, [startFen]);

  const current = entries[entries.length - 1];

  return {
    fen: current.fen,
    status: current.status,
    moves: entries.slice(1).map((e) => formatLastMove(e.fen) ?? "—"),
    canUndo: entries.length > 1,
    async submit(move) {
      const result = await applyMove(current.fen, move);
      setEntries((e) => [...e, result]);
    },
    undo() {
      setEntries((e) => (e.length > 1 ? e.slice(0, -1) : e));
    },
    restart() {
      setEntries((e) => e.slice(0, 1));
    },
  };
}
