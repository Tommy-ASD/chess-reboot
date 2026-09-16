// src/online/useLobby.ts
//
// Live lobby feed for as long as a component is mounted: open public games
// (joinable) and public games in progress (watchable). The lobby WebSocket
// pushes the open list on every change; the in-progress list is refetched
// alongside it.

import { useEffect, useState } from "preact/hooks";
import * as online from "../api/online";
import type { GameState } from "../api/online";

export interface Lobby {
  open: GameState[];
  live: GameState[];
  status: "loading" | "ready" | "offline";
}

export function useLobby(): Lobby {
  const [lobby, setLobby] = useState<Lobby>({ open: [], live: [], status: "loading" });

  useEffect(() => {
    let alive = true;

    const refresh = async (open?: GameState[]) => {
      try {
        const [openList, liveList] = await Promise.all([open ?? online.listGames(), online.listLive()]);
        if (alive) setLobby({ open: openList, live: liveList, status: "ready" });
      } catch {
        if (alive) setLobby((l) => ({ ...l, status: "offline" }));
      }
    };

    refresh();
    const sub = online.subscribeLobby((open) => refresh(open));
    return () => {
      alive = false;
      sub.close();
    };
  }, []);

  return lobby;
}
