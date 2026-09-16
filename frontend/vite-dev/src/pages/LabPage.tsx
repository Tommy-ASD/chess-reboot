// src/pages/LabPage.tsx
//
// The Lab: paint any position (pieces, special squares, conditions), tune
// the rules flags, then play it, save it as a community variant, or share
// it as a link. Editing rules live in `lab/editor.ts`; this page is layout
// and state.

import type { ComponentChildren } from "preact";
import { useMemo, useState } from "preact/hooks";
import { useLocation, useRoute } from "preact-iso";
import { Board } from "../components/board/Board";
import { DuckSprite, PieceSprite } from "../components/board/PieceSprite";
import { CopyField, Segmented, copyText } from "../components/ui/Controls";
import { displayName } from "../app/prefs";
import { fetchStatus } from "../api/board";
import { DEFAULT_FLAGS, parseFEN, parseFENFlags, serializeFullFEN, type BoardFlags } from "../core/fen";
import { MIRROR_AXES, MIRROR_LABELS, mirrorPosition, type MirrorAxis } from "../core/mirror";
import { HALF_LABELS, HALF_SOURCES, shouldInvertColors, symmetrizeBoard, type ColorMode } from "../core/symmetrize";
import { squareTypeIconByType } from "../core/signal_icons";
import { getEmitterTargets, getReceiverId, isEmitter, isReceiver } from "../core/signal_payload";
import type { Coord, Square } from "../core/types";
import { saveVariant, shareUrl, variantQuery } from "../content/userVariants";
import {
  BLACK_PIECES,
  CHESS2_PIECES,
  CONDITIONS,
  EMPTY_FEN,
  SQUARE_TYPES,
  STARTING_FEN,
  TEMPLATES,
  TRAIN_PIECES,
  TYPE_ACCENT,
  TYPE_LABELS,
  WHITE_PIECES,
  applyBrush,
  cloneBoard,
  describeBrush,
  inBounds,
  resizeBoard,
  sameBrush,
  type Brush,
} from "../lab/editor";
import { Inspector } from "../lab/Inspector";
import { pieceForSymbol } from "../content/pieces";
import { showError, toast } from "../ui/toast";

interface Position {
  board: Square[][];
  flags: BoardFlags;
}

function loadPosition(fen: string): Position | null {
  if (!fen.trim()) return null;
  try {
    const board = parseFEN(fen);
    if (board.length === 0 || (board[0]?.length ?? 0) === 0) return null;
    return { board, flags: parseFENFlags(fen) };
  } catch {
    return null;
  }
}

export function LabPage() {
  const { query } = useRoute();
  const { route } = useLocation();
  const [pos, setPos] = useState<Position>(() => (query.fen ? loadPosition(query.fen) : null) ?? loadPosition(STARTING_FEN)!);
  const [brush, setBrush] = useState<Brush | null>(null);
  const [inspected, setInspected] = useState<Coord | null>(null);
  const [fenDraft, setFenDraft] = useState<string | null>(null);
  const [colorMode, setColorMode] = useState<ColorMode>("auto");
  const [saving, setSaving] = useState(false);

  const { board, flags } = pos;
  const fen = useMemo(() => serializeFullFEN(board, flags), [board, flags]);
  const rows = board.length;
  const cols = board[0]?.length ?? 0;
  const draftInvalid = fenDraft !== null && loadPosition(fenDraft) === null;

  const commit = (next: Partial<Position>) => {
    setPos((p) => ({ ...p, ...next }));
    setFenDraft(null);
  };
  const setFlags = (patch: Partial<BoardFlags>) => commit({ flags: { ...flags, ...patch } });

  const onSquareClick = (at: Coord) => {
    if (!brush) {
      toast("Pick a brush first", "info", 1800);
      return;
    }
    const result = applyBrush(board, brush, at, inspected);
    commit({ board: result.board });
    setInspected(result.inspected);
  };

  const loadFen = (text: string) => {
    const next = loadPosition(text);
    if (!next) return false;
    setPos(next);
    if (!inBounds(next.board, inspected)) setInspected(null);
    return true;
  };
  const replaceWith = (text: string) => {
    loadFen(text);
    setFenDraft(null);
    setInspected(null);
  };

  /// The engine is the authority on whether a position is playable.
  const validated = async (): Promise<boolean> => {
    try {
      await fetchStatus(fen);
      return true;
    } catch (err) {
      showError(err);
      return false;
    }
  };

  // Inspect-mode marks: the open square, plus receivers wired to an open emitter.
  const inspectedSquare = inBounds(board, inspected) ? board[inspected.rank][inspected.file] : null;
  const marks: Record<string, string> = {};
  const wires: { from: Coord; to: Coord }[] = [];
  if (inspected && inspectedSquare) {
    marks[`${inspected.file},${inspected.rank}`] = "is-inspected";
    if (isEmitter(inspectedSquare)) {
      const targets = getEmitterTargets(inspectedSquare);
      board.forEach((row, r) =>
        row.forEach((sq, f) => {
          const rid = isReceiver(sq) ? getReceiverId(sq) : null;
          if (rid !== null && targets.includes(rid)) {
            marks[`${f},${r}`] = "is-wired";
            wires.push({ from: inspected, to: { file: f, rank: r } });
          }
        }),
      );
    }
  }

  const brushButton = (b: Brush, label: string, visual: ComponentChildren, title?: string) => (
    <button type="button" class={`brush${sameBrush(brush, b) ? " is-active" : ""}`} title={title ?? label} aria-pressed={sameBrush(brush, b)} onClick={() => setBrush(sameBrush(brush, b) ? null : b)}>
      <span class="brush-visual">{visual}</span>
      <span class="brush-label">{label}</span>
    </button>
  );
  const pieceBrush = (symbol: string) => {
    const def = pieceForSymbol(symbol);
    return brushButton({ kind: "piece", piece: symbol }, def?.name ?? symbol, <PieceSprite symbol={symbol} />, `${def?.name ?? symbol} (${symbol.split("(")[0]})`);
  };

  return (
    <div class="container page lab">
      <header class="page-head lab-head">
        <div>
          <p class="eyebrow">Lab</p>
          <h1 class="page-title">Build a position</h1>
          <p class="page-lede">Paint any setup, then play it, save it as your own variant, or share it as a link.</p>
        </div>
      </header>

      <section class="panel fen-bar">
        <label class="field-label" for="lab-fen">
          FEN
        </label>
        <input
          id="lab-fen"
          class={`input input-mono${draftInvalid ? " is-invalid" : ""}`}
          spellcheck={false}
          value={fenDraft ?? fen}
          aria-invalid={draftInvalid}
          onInput={(e) => {
            const text = e.currentTarget.value;
            setFenDraft(text);
            loadFen(text);
          }}
          onBlur={() => !draftInvalid && setFenDraft(null)}
        />
        <button type="button" class="btn btn-secondary btn-sm" onClick={() => copyText(fen, "FEN")}>
          Copy
        </button>
        <button
          type="button"
          class="btn btn-secondary btn-sm"
          onClick={async () => {
            try {
              const text = await navigator.clipboard.readText();
              if (!loadFen(text.trim())) toast("That isn't a readable FEN", "warn");
              else setFenDraft(null);
            } catch {
              toast("Clipboard access was blocked; paste into the box instead", "warn");
            }
          }}
        >
          Paste
        </button>
        <select
          class="input input-sm"
          aria-label="Load a template"
          value=""
          onChange={(e) => {
            const t = TEMPLATES.find((x) => x.name === e.currentTarget.value);
            if (t) replaceWith(t.fen);
          }}
        >
          <option value="">Templates…</option>
          {TEMPLATES.map((t) => (
            <option key={t.name} value={t.name}>
              {t.name}
            </option>
          ))}
        </select>
        {draftInvalid && <span class="field-error">Can't read that FEN yet. The board shows the last valid position.</span>}
      </section>

      <div class="lab-grid">
        <aside class="panel lab-palette" aria-label="Brushes">
          <PaletteGroup title="White">{WHITE_PIECES.map(pieceBrush)}</PaletteGroup>
          <PaletteGroup title="Black">{BLACK_PIECES.map(pieceBrush)}</PaletteGroup>
          <PaletteGroup title="Chess 2">{CHESS2_PIECES.map(pieceBrush)}</PaletteGroup>
          <PaletteGroup title="Trains & extras">
            {TRAIN_PIECES.map(pieceBrush)}
            {brushButton({ kind: "duck" }, "Duck", <DuckSprite />)}
          </PaletteGroup>
          <PaletteGroup title="Squares">
            {SQUARE_TYPES.map((t) => {
              const svg = squareTypeIconByType(t);
              return brushButton(
                { kind: "type", squareType: t },
                TYPE_LABELS[t],
                svg ? <span class="brush-svg" style={{ color: TYPE_ACCENT[t] }} dangerouslySetInnerHTML={{ __html: svg }} /> : <span class="brush-glyph">□</span>,
              );
            })}
            {brushButton({ kind: "inspect" }, "Inspect", <span class="brush-glyph">⌕</span>, "Inspect and wire special squares")}
          </PaletteGroup>
          <PaletteGroup title="Conditions">
            {CONDITIONS.map((c) =>
              brushButton({ kind: "condition", condition: c.value }, c.label, <span class={`brush-cond brush-cond-${c.value.split(":")[0].toLowerCase()}`} />),
            )}
          </PaletteGroup>
          <PaletteGroup title="Erase">
            {brushButton({ kind: "erase-piece" }, "Piece", <span class="brush-glyph">✕</span>)}
            {brushButton({ kind: "erase-conditions" }, "Conditions", <span class="brush-glyph">✕</span>)}
            {brushButton({ kind: "erase-type" }, "Square", <span class="brush-glyph">✕</span>)}
            {brushButton({ kind: "erase-all" }, "All", <span class="brush-glyph">✖</span>)}
          </PaletteGroup>
        </aside>

        <div class="lab-board">
          <p class={`brush-status${brush ? " is-active" : ""}`}>{describeBrush(brush)}</p>
          <Board squares={board} marks={marks} onSquareClick={onSquareClick} coords label="Lab board">
            {wires.length > 0 && (
              <svg class="wiring-overlay" viewBox={`0 0 ${cols} ${rows}`} preserveAspectRatio="none" aria-hidden="true">
                {wires.map((w, i) => (
                  <g key={i}>
                    <line x1={w.from.file + 0.5} y1={w.from.rank + 0.5} x2={w.to.file + 0.5} y2={w.to.rank + 0.5} />
                    <circle cx={w.to.file + 0.5} cy={w.to.rank + 0.5} r={0.08} />
                  </g>
                ))}
              </svg>
            )}
          </Board>
          {brush?.kind === "inspect" || inspected ? (
            <section class="panel">
              <div class="settings-section-head">
                <h2 class="panel-title">Inspector</h2>
                {inspected && (
                  <button type="button" class="btn btn-ghost btn-sm" onClick={() => setInspected(null)}>
                    Close
                  </button>
                )}
              </div>
              <Inspector
                square={inspectedSquare}
                at={inspected}
                rows={rows}
                update={(mutate) => {
                  if (!inspected) return;
                  const next = cloneBoard(board);
                  mutate(next[inspected.rank][inspected.file]);
                  commit({ board: next });
                }}
              />
            </section>
          ) : null}
        </div>

        <aside class="lab-side">
          <section class="panel">
            <h2 class="panel-title">Use this position</h2>
            <div class="stack">
              <button
                type="button"
                class="btn btn-primary btn-block"
                onClick={async () => {
                  if (await validated()) route(`/local?${variantQuery({ name: "Lab position", fen })}`);
                }}
              >
                Play on this device
              </button>
              <button
                type="button"
                class="btn btn-secondary btn-block"
                onClick={async () => {
                  if (await validated()) route(`/play?${variantQuery({ name: "Lab position", fen, author: displayName.value.trim() })}`);
                }}
              >
                Play online…
              </button>
              {saving ? (
                <SaveVariantForm
                  onCancel={() => setSaving(false)}
                  onSave={async (name, description) => {
                    if (!(await validated())) return;
                    const v = saveVariant({ name, description, fen, author: displayName.value.trim() || "Anonymous" });
                    setSaving(false);
                    toast(`Saved “${v.name}”. It's on the home page under Community variants.`, "success", 4500);
                  }}
                />
              ) : (
                <button type="button" class="btn btn-secondary btn-block" onClick={() => setSaving(true)}>
                  Save as a variant…
                </button>
              )}
            </div>
            <CopyField label="Share link" value={shareUrl({ name: "Lab position", fen, author: displayName.value.trim() })} />
          </section>

          <section class="panel">
            <h2 class="panel-title">Position</h2>
            <div class="field">
              <span class="field-label">Side to move</span>
              <Segmented
                label="Side to move"
                value={flags.sideToMove}
                onChange={(v) => setFlags({ sideToMove: v })}
                options={[
                  { value: "w", label: "White" },
                  { value: "b", label: "Black" },
                ]}
              />
            </div>
            <div class="field-row">
              <DimensionField label="Width" value={cols} onCommit={(v) => commit({ board: resizeBoard(board, v, rows) })} />
              <DimensionField label="Height" value={rows} onCommit={(v) => commit({ board: resizeBoard(board, cols, v) })} />
            </div>
            <div class="field">
              <span class="field-label">Mirror</span>
              <div class="tool-grid">
                {MIRROR_AXES.map((axis: MirrorAxis) => (
                  <button
                    key={axis}
                    type="button"
                    class="btn btn-secondary btn-sm"
                    title={MIRROR_LABELS[axis].title}
                    onClick={() => {
                      const next = mirrorPosition(board, flags, axis);
                      commit(next);
                      setInspected(null);
                    }}
                  >
                    {MIRROR_LABELS[axis].short}
                  </button>
                ))}
              </div>
            </div>
            <div class="field">
              <span class="field-label">Copy one half onto the other</span>
              <Segmented<ColorMode>
                label="Piece colours on the copied half"
                value={colorMode}
                onChange={setColorMode}
                options={[
                  { value: "auto", label: "Auto", title: "Swap colours top ↔ bottom, keep them left ↔ right" },
                  { value: "keep", label: "Keep", title: "Copied pieces keep their colour" },
                  { value: "invert", label: "Swap", title: "Copied pieces swap colour" },
                ]}
              />
              <div class="tool-grid">
                {HALF_SOURCES.map((source) => (
                  <button
                    key={source}
                    type="button"
                    class="btn btn-secondary btn-sm"
                    title={HALF_LABELS[source].title}
                    onClick={() => commit({ board: symmetrizeBoard(board, source, shouldInvertColors(source, colorMode)) })}
                  >
                    {HALF_LABELS[source].short}
                  </button>
                ))}
              </div>
            </div>
            <div class="button-row">
              <button type="button" class="btn btn-ghost btn-sm" onClick={() => replaceWith(STARTING_FEN)}>
                Reset to standard
              </button>
              <button type="button" class="btn btn-ghost btn-sm" onClick={() => replaceWith(EMPTY_FEN)}>
                Clear board
              </button>
            </div>
          </section>

          <details class="panel lab-advanced">
            <summary>Advanced rules</summary>
            <div class="field">
              <span class="field-label">Castling rights</span>
              <div class="check-grid">
                {([
                  ["K", "White O-O"],
                  ["Q", "White O-O-O"],
                  ["k", "Black O-O"],
                  ["q", "Black O-O-O"],
                ] as const).map(([key, label]) => (
                  <label key={key} class="inline-check">
                    <input type="checkbox" checked={flags.castling[key]} onChange={(e) => setFlags({ castling: { ...flags.castling, [key]: e.currentTarget.checked } })} />
                    {label}
                  </label>
                ))}
              </div>
            </div>
            <label class="field field-inline">
              <span class="field-label">En passant square</span>
              <input
                class="input input-sm input-mono"
                maxLength={3}
                placeholder="-"
                value={flags.enPassant ?? ""}
                onChange={(e) => {
                  const v = e.currentTarget.value.trim();
                  setFlags({ enPassant: v === "" || v === "-" ? null : v });
                }}
              />
            </label>
            <label class="inline-check">
              <input
                type="checkbox"
                checked={flags.variants.includes("duck_chess")}
                onChange={(e) =>
                  setFlags({
                    variants: e.currentTarget.checked ? [...flags.variants, "duck_chess"] : flags.variants.filter((v) => v !== "duck_chess"),
                    duckPhase: "piece",
                  })
                }
              />
              Duck Chess rules
            </label>
            <label class="field field-inline">
              <span class="field-label">Trains move</span>
              <select
                class="input input-sm"
                value={flags.trainTickRate.kind}
                onChange={(e) => {
                  const kind = e.currentTarget.value;
                  setFlags({
                    trainTickRate:
                      kind === "EveryNPly"
                        ? { kind, n: flags.trainTickRate.kind === "EveryNPly" ? flags.trainTickRate.n : 2 }
                        : kind === "EveryPly"
                          ? { kind }
                          : { kind: "EveryFullTurn" },
                  });
                }}
              >
                <option value="EveryFullTurn">Every full turn</option>
                <option value="EveryPly">Every move</option>
                <option value="EveryNPly">Every N moves</option>
              </select>
            </label>
            {flags.trainTickRate.kind === "EveryNPly" && (
              <DimensionField label="N" value={flags.trainTickRate.n} onCommit={(n) => setFlags({ trainTickRate: { kind: "EveryNPly", n } })} />
            )}
            <label class="field field-inline">
              <span class="field-label">Ply count</span>
              <input
                class="input input-sm input-mono"
                type="number"
                min={0}
                value={flags.plyCount}
                onChange={(e) => {
                  const n = Number.parseInt(e.currentTarget.value, 10);
                  if (Number.isFinite(n) && n >= 0) setFlags({ plyCount: n });
                }}
              />
            </label>
            <button type="button" class="btn btn-ghost btn-sm" onClick={() => setFlags({ ...DEFAULT_FLAGS, sideToMove: flags.sideToMove })}>
              Reset rule flags
            </button>
          </details>
        </aside>
      </div>
    </div>
  );
}

function PaletteGroup({ title, children }: { title: string; children: ComponentChildren }) {
  return (
    <div class="palette-group">
      <h2 class="field-label">{title}</h2>
      <div class="palette">{children}</div>
    </div>
  );
}

function DimensionField({ label, value, onCommit }: { label: string; value: number; onCommit: (v: number) => void }) {
  return (
    <label class="field">
      <span class="field-label">{label}</span>
      <input
        class="input input-sm input-mono"
        type="number"
        min={1}
        max={32}
        value={value}
        onChange={(e) => {
          const n = Number.parseInt(e.currentTarget.value, 10);
          if (Number.isFinite(n) && n >= 1 && n <= 32) onCommit(n);
          else e.currentTarget.value = String(value);
        }}
      />
    </label>
  );
}

function SaveVariantForm({ onSave, onCancel }: { onSave: (name: string, description: string) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  return (
    <form
      class="save-form"
      onSubmit={(e) => {
        e.preventDefault();
        if (name.trim()) onSave(name.trim(), description.trim());
      }}
    >
      <label class="field">
        <span class="field-label">Variant name</span>
        <input class="input" required maxLength={40} value={name} onInput={(e) => setName(e.currentTarget.value)} autoFocus />
      </label>
      <label class="field">
        <span class="field-label">One-line description</span>
        <input class="input" maxLength={120} value={description} placeholder="What makes it fun?" onInput={(e) => setDescription(e.currentTarget.value)} />
      </label>
      <div class="button-row">
        <button type="submit" class="btn btn-primary btn-sm" disabled={!name.trim()}>
          Save
        </button>
        <button type="button" class="btn btn-ghost btn-sm" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}
