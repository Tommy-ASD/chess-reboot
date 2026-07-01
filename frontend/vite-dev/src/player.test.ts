import { afterEach, describe, expect, it, vi } from "vitest";

// player.ts is the client half of the account seam: a localStorage-backed
// identity plus the `authHeaders()` every online request carries. The
// module caches identity in module scope and reads the `localStorage`
// global, so each test runs against a fresh module instance (vi.resetModules)
// with an in-memory localStorage shim stubbed in — no jsdom needed.

const STORAGE_KEY = "chess.identity";
const UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

type Shim = {
  getItem(k: string): string | null;
  setItem(k: string, v: string): void;
  removeItem(k: string): void;
  clear(): void;
  store: Record<string, string>;
};

function makeLocalStorage(opts: { throwOnSet?: boolean } = {}): Shim {
  const store: Record<string, string> = {};
  return {
    store,
    getItem: (k) => (k in store ? store[k] : null),
    setItem: (k, v) => {
      if (opts.throwOnSet) throw new Error("localStorage disabled (private mode)");
      store[k] = v;
    },
    removeItem: (k) => {
      delete store[k];
    },
    clear: () => {
      for (const k of Object.keys(store)) delete store[k];
    },
  };
}

/// Load a pristine copy of player.ts bound to `ls`. Resetting modules drops
/// the cached identity so each test starts clean.
async function freshPlayer(ls: Shim) {
  vi.resetModules();
  vi.stubGlobal("localStorage", ls);
  return import("./player");
}

afterEach(() => vi.unstubAllGlobals());

describe("getIdentity — minting", () => {
  it("mints a v4 UUID with a blank name and persists it", async () => {
    const ls = makeLocalStorage();
    const p = await freshPlayer(ls);
    const id = p.getIdentity();
    expect(id.playerId).toMatch(UUID_V4);
    expect(id.name).toBe("");
    // Persisted eagerly so the id survives the next reload.
    expect(JSON.parse(ls.store[STORAGE_KEY]).playerId).toBe(id.playerId);
  });

  it("is stable across calls within a session (cached)", async () => {
    const p = await freshPlayer(makeLocalStorage());
    expect(p.getIdentity().playerId).toBe(p.getIdentity().playerId);
  });
});

describe("getIdentity — loading existing", () => {
  it("adopts a well-formed stored identity verbatim", async () => {
    const ls = makeLocalStorage();
    ls.store[STORAGE_KEY] = JSON.stringify({
      playerId: "11111111-1111-4111-8111-111111111111",
      name: "Alice",
    });
    const p = await freshPlayer(ls);
    expect(p.getIdentity()).toEqual({
      playerId: "11111111-1111-4111-8111-111111111111",
      name: "Alice",
    });
    expect(p.getName()).toBe("Alice");
  });

  it("defaults a stored record with no name to an empty name", async () => {
    const ls = makeLocalStorage();
    ls.store[STORAGE_KEY] = JSON.stringify({ playerId: "22222222-2222-4222-9222-222222222222" });
    const p = await freshPlayer(ls);
    expect(p.getIdentity().name).toBe("");
    expect(p.getIdentity().playerId).toBe("22222222-2222-4222-9222-222222222222");
  });

  it("mints fresh on corrupt JSON", async () => {
    const ls = makeLocalStorage();
    ls.store[STORAGE_KEY] = "{ not json";
    const p = await freshPlayer(ls);
    const id = p.getIdentity();
    expect(id.playerId).toMatch(UUID_V4);
    expect(id.name).toBe("");
  });

  it("mints fresh when the stored record lacks a usable playerId", async () => {
    const ls = makeLocalStorage();
    // Missing playerId — the whole record is discarded, name included.
    ls.store[STORAGE_KEY] = JSON.stringify({ name: "Nameless" });
    const p = await freshPlayer(ls);
    expect(p.getIdentity().playerId).toMatch(UUID_V4);
    expect(p.getIdentity().name).toBe("");
  });

  it("mints fresh when playerId is an empty string", async () => {
    const ls = makeLocalStorage();
    ls.store[STORAGE_KEY] = JSON.stringify({ playerId: "", name: "Ghost" });
    const p = await freshPlayer(ls);
    expect(p.getIdentity().playerId).toMatch(UUID_V4);
    expect(p.getIdentity().name).toBe("");
  });
});

describe("setName", () => {
  it("trims, persists, and reflects through getName", async () => {
    const ls = makeLocalStorage();
    const p = await freshPlayer(ls);
    p.setName("  Bob  ");
    expect(p.getName()).toBe("Bob");
    expect(JSON.parse(ls.store[STORAGE_KEY]).name).toBe("Bob");
  });

  it("allows clearing the name back to empty", async () => {
    const ls = makeLocalStorage();
    ls.store[STORAGE_KEY] = JSON.stringify({
      playerId: "33333333-3333-4333-8333-333333333333",
      name: "Carol",
    });
    const p = await freshPlayer(ls);
    p.setName("   ");
    expect(p.getName()).toBe("");
  });
});

describe("authHeaders", () => {
  it("always carries X-Player-Id and omits the name when blank", async () => {
    const p = await freshPlayer(makeLocalStorage());
    const h = p.authHeaders();
    expect(h["X-Player-Id"]).toBe(p.getIdentity().playerId);
    expect("X-Player-Name" in h).toBe(false);
  });

  it("carries X-Player-Name once a name is set", async () => {
    const p = await freshPlayer(makeLocalStorage());
    p.setName("Dana");
    const h = p.authHeaders();
    expect(h["X-Player-Id"]).toBe(p.getIdentity().playerId);
    expect(h["X-Player-Name"]).toBe("Dana");
  });
});

describe("degraded storage (private mode)", () => {
  it("still yields a stable session identity when persistence throws", async () => {
    const ls = makeLocalStorage({ throwOnSet: true });
    const p = await freshPlayer(ls);
    const id = p.getIdentity();
    expect(id.playerId).toMatch(UUID_V4);
    // Cached for the session even though nothing could be written.
    expect(p.getIdentity().playerId).toBe(id.playerId);
    expect(STORAGE_KEY in ls.store).toBe(false);
  });
});
