// src/components/board/useBoardPlay.ts
//
// Click-to-move interaction for a playable board, shared by online games,
// same-device games and the piece demos. Owns selection, the selected
// piece's legal moves (fetched from the engine), carrier passenger picking,
// the promotion choice and Duck Chess placement. The caller owns the
// position: it passes `fen`, and `submit` applies a move however that game
// applies moves (REST to the game server, or the stateless engine).

import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import { fetchMoves } from "../../api/board";
import {
  findMoveForTarget,
  isAllowedSquare,
  isPieceInCarrier,
  isSpecialMove,
  promotionsAt,
  visibleMoveTargets,
  type BoardTarget,
} from "../../core/board_helpers";
import { parseFEN, parseFENFlags, parseLastMove, type BoardFlags } from "../../core/fen";
import { getCarrierPassengers } from "../../core/train_payload";
import type { Coord, GameMove, Square } from "../../core/types";
import { pieceSide } from "../../content/pieces";
import { showError } from "../../ui/toast";

export interface BoardPlayOptions {
  fen: string;
  /** Whether the viewer may move right now (their turn, game live). */
  canMove: boolean;
  submit: (move: GameMove) => Promise<void>;
}

export interface BoardPlay {
  squares: Square[][] | null;
  flags: BoardFlags;
  selected: Coord | null;
  selectedPiece: string | null;
  targets: BoardTarget[];
  lastMove: { from: Coord; to: Coord } | null;
  duckPlacing: boolean;
  /** Side-panel actions for the selected piece (brainrot, tornado, switch). */
  specials: GameMove[];
  /** Passengers of the selected carrier, when they can get off. */
  passengers: string[];
  passengerIndex: number | null;
  setPassengerIndex: (i: number | null) => void;
  /** Non-null while waiting for a promotion choice. */
  promotion: GameMove[] | null;
  cancelPromotion: () => void;
  busy: boolean;
  onSquareClick: (c: Coord) => void;
  play: (move: GameMove) => void;
  clear: () => void;
}

const same = (a: Coord | null, b: Coord) => a !== null && a.file === b.file && a.rank === b.rank;

export function useBoardPlay({ fen, canMove, submit }: BoardPlayOptions): BoardPlay {
  const squares = useMemo(() => {
    try {
      const parsed = parseFEN(fen);
      return parsed.length > 0 && parsed[0].length > 0 ? parsed : null;
    } catch {
      return null;
    }
  }, [fen]);
  const flags = useMemo(() => parseFENFlags(fen), [fen]);
  const lastMove = useMemo(() => parseLastMove(fen), [fen]);
  const duckPlacing = flags.variants.includes("duck_chess") && flags.duckPhase === "placing";

  const [selected, setSelected] = useState<Coord | null>(null);
  const [moves, setMoves] = useState<GameMove[]>([]);
  const [passengerIndex, setPassengerIndex] = useState<number | null>(null);
  const [promotion, setPromotion] = useState<GameMove[] | null>(null);
  const [busy, setBusy] = useState(false);
  // Discards move lists that arrive after the selection or position changed.
  const requestId = useRef(0);

  const clear = () => {
    requestId.current++;
    setSelected(null);
    setMoves([]);
    setPassengerIndex(null);
    setPromotion(null);
  };

  useEffect(clear, [fen]);
  useEffect(() => {
    if (!canMove) {
      requestId.current++;
      setMoves([]);
      setPromotion(null);
    }
  }, [canMove]);

  const play = (move: GameMove) => {
    if (busy) return;
    setBusy(true);
    setPromotion(null);
    submit(move)
      .then(clear)
      .catch(showError)
      .finally(() => setBusy(false));
  };

  const onSquareClick = (c: Coord) => {
    if (!squares || busy) return;
    setPromotion(null);
    const sq = squares[c.rank]?.[c.file];
    if (!sq) return;

    // Duck Chess placement half-turn: pieces are locked; an empty square
    // receives the duck (placed on the first turn, moved afterwards).
    if (duckPlacing) {
      if (!canMove || sq.piece || sq.duck || sq.squareType === "BLOCK") return;
      const existing = findDuck(squares);
      play(
        existing
          ? { from: existing, move_type: { kind: "MoveDuck", target: { to: c } } }
          : { from: c, move_type: { kind: "PlaceDuck", target: { to: c } } },
      );
      return;
    }

    if (same(selected, c)) {
      clear();
      return;
    }

    if (canMove && isAllowedSquare(c, moves, passengerIndex)) {
      const promos = promotionsAt(c, moves);
      if (promos.length > 0 && passengerIndex === null) {
        setPromotion(promos);
        return;
      }
      const move = findMoveForTarget(c, moves, passengerIndex);
      if (move) play(move);
      return;
    }

    if (!sq.piece) {
      clear();
      return;
    }

    // Select the piece. Anyone can select to read about it; legal moves are
    // only fetched for the side to move (neutral carriers carry both sides'
    // passengers, so they're always worth asking about).
    const id = ++requestId.current;
    setSelected(c);
    setMoves([]);
    setPassengerIndex(null);
    const side = pieceSide(sq.piece);
    const toMove = flags.sideToMove === "w" ? "white" : "black";
    if (!canMove || (side !== "neutral" && side !== toMove)) return;
    fetchMoves(fen, c)
      .then((list) => {
        if (requestId.current === id) setMoves(list);
      })
      .catch((err) => {
        if (requestId.current === id) showError(err);
      });
  };

  const selectedPiece = selected && squares ? squares[selected.rank]?.[selected.file]?.piece ?? null : null;
  const canDeploy = moves.some(isPieceInCarrier);
  const passengers = selectedPiece && canDeploy ? getCarrierPassengers(selectedPiece) : [];

  return {
    squares,
    flags,
    selected,
    selectedPiece,
    targets: visibleMoveTargets(moves, passengerIndex),
    lastMove,
    duckPlacing,
    specials: moves.filter(isSpecialMove),
    passengers,
    passengerIndex,
    setPassengerIndex,
    promotion,
    cancelPromotion: () => setPromotion(null),
    busy,
    onSquareClick,
    play,
    clear,
  };
}

function findDuck(squares: Square[][]): Coord | null {
  for (let rank = 0; rank < squares.length; rank++) {
    for (let file = 0; file < squares[rank].length; file++) {
      if (squares[rank][file].duck) return { file, rank };
    }
  }
  return null;
}
