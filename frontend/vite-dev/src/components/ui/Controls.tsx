// src/components/ui/Controls.tsx
//
// Small reusable controls.

import { useState } from "preact/hooks";
import { toast } from "../../ui/toast";

export async function copyText(text: string, what = "Link"): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    toast(`${what} copied`, "success", 1800);
    return true;
  } catch {
    window.prompt(`Copy this ${what.toLowerCase()}:`, text);
    return false;
  }
}

/// A read-only value with a Copy button (share links, game codes).
export function CopyField({ value, label, what = "Link" }: { value: string; label: string; what?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div class="copy-field">
      <label class="field-label">{label}</label>
      <div class="copy-field-row">
        <input class="input input-mono" readOnly value={value} onFocus={(e) => e.currentTarget.select()} aria-label={label} />
        <button
          type="button"
          class="btn btn-primary"
          onClick={async () => {
            if (await copyText(value, what)) {
              setCopied(true);
              setTimeout(() => setCopied(false), 1500);
            }
          }}
        >
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
    </div>
  );
}

export interface SegmentOption<T extends string> {
  value: T;
  label: string;
  title?: string;
}

export function Segmented<T extends string>({ options, value, onChange, label }: {
  options: SegmentOption<T>[];
  value: T;
  onChange: (v: T) => void;
  label: string;
}) {
  return (
    <div class="segmented" role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={o.value === value}
          class={`segmented-option${o.value === value ? " is-active" : ""}`}
          title={o.title}
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Toggle({ checked, onChange, label, description }: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  description?: string;
}) {
  return (
    <label class="toggle-row">
      <span class="toggle-text">
        <span class="toggle-label">{label}</span>
        {description && <span class="muted">{description}</span>}
      </span>
      <input type="checkbox" class="toggle" role="switch" checked={checked} onChange={(e) => onChange(e.currentTarget.checked)} />
    </label>
  );
}
