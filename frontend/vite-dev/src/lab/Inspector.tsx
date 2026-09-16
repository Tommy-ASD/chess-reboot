// src/lab/Inspector.tsx
//
// Details editor for the inspected square: signal substrate payloads
// (switch / plate targets, junction, gate, track direction) and train carts
// sitting on it. Edits go through `update`, which applies a mutation to a
// copy of the square and commits the new board.

import type { ComponentChildren } from "preact";
import {
  ALL_PRESSURE_TRIGGERS,
  ALL_TRACK_DIRS,
  getEmitterTargets,
  getGateId,
  getGateOpen,
  getJunctionBranches,
  getJunctionId,
  getJunctionState,
  getPlateTrigger,
  getTrackDir,
  setEmitterTargets,
  setGateId,
  setGateOpen,
  setJunctionBranches,
  setJunctionId,
  setJunctionState,
  setPlateTrigger,
  setTrackDir,
  type PressureTrigger,
} from "../core/signal_payload";
import { ALL_TRAIN_HEADINGS, isTrainCart, parseTrainCart, serializeTrainCart, type TrainCart } from "../core/train_payload";
import { squareName } from "../core/notation";
import type { Coord, Square } from "../core/types";
import { TYPE_LABELS } from "./editor";
import { useState } from "preact/hooks";

interface InspectorProps {
  square: Square | null;
  at: Coord | null;
  rows: number;
  update: (mutate: (sq: Square) => void) => void;
}

const TRIGGER_LABELS: Record<PressureTrigger, [string, string]> = {
  ANY: ["Any", "Fires for any piece"],
  W: ["White", "Fires only for white pieces"],
  B: ["Black", "Fires only for black pieces"],
  N: ["Neutral", "Fires only for train carts"],
};

export function Inspector({ square, at, rows, update }: InspectorProps) {
  if (!square || !at) {
    return <p class="muted small">Pick the Inspect tool, then tap a switch, junction, gate, plate, track or train to edit it.</p>;
  }
  const train = isTrainCart(square.piece);
  const editable = square.squareType !== "STANDARD" || train;

  return (
    <div class="inspector">
      <p class="inspector-coord mono">
        {squareName(at, rows)} · {TYPE_LABELS[square.squareType]}
      </p>
      {square.squareType === "SWITCH" && (
        <>
          <TargetsEditor square={square} update={update} />
          <p class="muted small">With this switch open, tap a junction or gate to wire or unwire it.</p>
        </>
      )}
      {square.squareType === "PLATE" && (
        <>
          <TargetsEditor square={square} update={update} />
          <Field label="Fires for">
            <Choices
              options={ALL_PRESSURE_TRIGGERS.map((t) => ({ value: t, label: TRIGGER_LABELS[t][0], title: TRIGGER_LABELS[t][1] }))}
              isOn={(t) => getPlateTrigger(square) === t}
              onPick={(t) => update((sq) => setPlateTrigger(sq, t))}
            />
          </Field>
          <p class="muted small">With this plate open, tap a junction or gate to wire or unwire it.</p>
        </>
      )}
      {square.squareType === "JUNCTION" && (
        <>
          <NumberField label="Signal ID" value={getJunctionId(square)} onCommit={(v) => update((sq) => setJunctionId(sq, v))} />
          <NumberField label="State" value={getJunctionState(square)} onCommit={(v) => update((sq) => setJunctionState(sq, v))} />
          <Field label="Branches">
            <Choices
              options={ALL_TRACK_DIRS.map((d) => ({ value: d, label: d }))}
              isOn={(d) => getJunctionBranches(square).includes(d)}
              onPick={(d) =>
                update((sq) => {
                  const next = getJunctionBranches(sq);
                  const i = next.indexOf(d);
                  if (i === -1) next.push(d);
                  else next.splice(i, 1);
                  setJunctionBranches(sq, next);
                })
              }
            />
          </Field>
        </>
      )}
      {square.squareType === "GATE" && (
        <>
          <NumberField label="Signal ID" value={getGateId(square)} onCommit={(v) => update((sq) => setGateId(sq, v))} />
          <label class="inline-check">
            <input type="checkbox" checked={getGateOpen(square)} onChange={(e) => update((sq) => setGateOpen(sq, e.currentTarget.checked))} />
            Open
          </label>
        </>
      )}
      {square.squareType === "TRACK" && (
        <Field label="Direction (outgoing)">
          <Choices options={ALL_TRACK_DIRS.map((d) => ({ value: d, label: d }))} isOn={(d) => getTrackDir(square) === d} onPick={(d) => update((sq) => setTrackDir(sq, d))} />
        </Field>
      )}
      {train && <TrainEditor square={square} update={update} />}
      {!editable && <p class="muted small">Nothing to edit on this square.</p>}
    </div>
  );
}

function Field({ label, children }: { label: string; children: ComponentChildren }) {
  return (
    <div class="field">
      <span class="field-label">{label}</span>
      {children}
    </div>
  );
}

function Choices<T extends string>({ options, isOn, onPick }: {
  options: { value: T; label: string; title?: string }[];
  isOn: (v: T) => boolean;
  onPick: (v: T) => void;
}) {
  return (
    <div class="choice-row">
      {options.map((o) => (
        <button key={o.value} type="button" class={`choice${isOn(o.value) ? " is-on" : ""}`} title={o.title} aria-pressed={isOn(o.value)} onClick={() => onPick(o.value)}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

function NumberField({ label, value, onCommit, min = 0 }: { label: string; value: number; onCommit: (v: number) => void; min?: number }) {
  return (
    <label class="field field-inline">
      <span class="field-label">{label}</span>
      <input
        class="input input-sm input-mono"
        type="number"
        min={min}
        value={value}
        onChange={(e) => {
          const n = Number.parseInt(e.currentTarget.value, 10);
          if (Number.isFinite(n) && n >= min) onCommit(n);
        }}
      />
    </label>
  );
}

function TargetsEditor({ square, update }: { square: Square; update: InspectorProps["update"] }) {
  return (
    <label class="field">
      <span class="field-label">Targets (signal IDs)</span>
      <input
        class="input input-sm input-mono"
        placeholder="e.g. 1,3,7"
        value={getEmitterTargets(square).join(",")}
        onChange={(e) => {
          const ids = e.currentTarget.value
            .split(",")
            .map((x) => Number.parseInt(x.trim(), 10))
            .filter((n) => Number.isFinite(n) && n >= 0);
          update((sq) => setEmitterTargets(sq, ids));
        }}
      />
    </label>
  );
}

/// Passenger symbols the engine accepts inside a carrier. Nested carriers
/// are refused at FEN parse engine-side, so they're refused here too.
const PASSENGER_KINDS = new Set(["p", "r", "n", "b", "q", "k", "g", "s", "m", "w"]);
const CARRIER_KINDS = new Set(["bus", "loco", "cart"]);

function TrainEditor({ square, update }: { square: Square; update: InspectorProps["update"] }) {
  const [dropped, setDropped] = useState("");
  const cart = parseTrainCart(square.piece ?? "");
  if (!cart) return <p class="muted small">Invalid train symbol.</p>;

  const commit = (mutate: (c: TrainCart) => void) =>
    update((sq) => {
      const c = parseTrainCart(sq.piece ?? "");
      if (!c) return;
      mutate(c);
      sq.piece = serializeTrainCart(c);
    });

  return (
    <div class="inspector-train">
      <p class="field-label">{cart.kind === "LOCO" ? "Locomotive" : "Carriage"}</p>
      <NumberField label="Train ID" value={cart.trainId} onCommit={(v) => commit((c) => (c.trainId = v))} />
      {cart.kind === "LOCO" ? (
        <>
          <Field label="Heading (first tick only)">
            <Choices
              options={ALL_TRAIN_HEADINGS.map((h) => ({ value: h, label: h === "F" ? "Forward" : "Reverse" }))}
              isOn={(h) => cart.heading === h}
              onPick={(h) => commit((c) => (c.heading = h))}
            />
          </Field>
          <Field label="Entered from">
            <Choices
              options={[{ value: "-", label: "—", title: "Unset: the next tick uses the tile's direction" }, ...ALL_TRACK_DIRS.map((d) => ({ value: d, label: d }))]}
              isOn={(d) => (d === "-" ? cart.lastDir === null : cart.lastDir === d)}
              onPick={(d) => commit((c) => (c.lastDir = d === "-" ? null : d))}
            />
          </Field>
        </>
      ) : (
        // Chain index 0 is reserved for the locomotive.
        <NumberField label="Chain index" min={1} value={cart.chainIndex} onCommit={(v) => commit((c) => (c.chainIndex = v))} />
      )}
      <label class="field">
        <span class="field-label">Passengers (symbols, comma-separated)</span>
        <input
          class="input input-sm input-mono"
          placeholder="e.g. P,N"
          value={cart.passengers.join(",")}
          onChange={(e) => {
            const accepted: string[] = [];
            const rejected: string[] = [];
            for (const sym of e.currentTarget.value.split(",").map((s) => s.trim()).filter(Boolean)) {
              const prefix = sym.split("(")[0].toLowerCase();
              if (CARRIER_KINDS.has(prefix)) rejected.push(`${sym} (nested carrier)`);
              else if (PASSENGER_KINDS.has(prefix)) accepted.push(sym);
              else rejected.push(sym);
            }
            setDropped(rejected.length ? `Dropped: ${rejected.join(", ")}` : "");
            commit((c) => (c.passengers = accepted));
          }}
        />
      </label>
      {dropped && <p class="field-error">{dropped}</p>}
    </div>
  );
}
