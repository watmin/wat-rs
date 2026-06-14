# Arc 262 — the `wat-scripts/` graveyard: migrate the stale examples to current syntax

**Status:** STUB — captured 2026-06-14, not scoped into stones. Surfaced while building the
self-hosted fix-wat runner (arc 251, Song #96): writing one *current* `wat-scripts/` program hit four
retired-form walls because the existing examples are pre-historic.

---

## The finding

`wat-scripts/` (telemetry interrogation scripts + ping-pong demos) has not been migrated since the
arc-153/170/241 surface changes. Every example there uses **retired forms** and would fail today:

1. **`:()` (bare unit type)** → `:wat::core::nil` (arc 153; checker rejects `:()` with a diagnostic).
2. **`:wat::core::define`** → `:wat::core::defn` (arc 241.11/241.16; `define` is retired).
3. **`:user::main` with stdio params** `(stdin :IOReader)(stdout :IOWriter)(stderr :IOWriter)` →
   **`main []`** (arc 170 slice 1e — main takes no args; stdio is the substrate services
   `:wat::kernel::readln` / `println` / `eprintln`; argv via `(:wat::runtime::argv)`).
4. **Trailing CLI args** — the `wat` CLI takes only `<entry.wat>`; programs read input from **stdin**,
   not argv-after-the-script.

Confirmed stale (non-exhaustive): `count-logs.wat`, `seed-fixture.wat` (both use `define` + `:()` +
stdio-param `main`). The `ping-pong*` / `aggregator` / `dispatch` / `router` / `metrics-summary`
scripts need an audit pass.

## Why it's worth a stub (not urgent)

These are interrogation/demo scripts, not load-bearing stdlib — nothing depends on them at runtime, so
their rot is latent. But they are the **documented examples** a reader (human or LLM) reaches for to
learn the program shape, and they teach a dead dialect. A reader who copies `count-logs.wat` writes a
program the checker rejects four ways. That is the recolligere trap at the example layer: the map
lies. (This is exactly how the self-hosted runner got bitten — copying `seed-fixture.wat`'s shape.)

## The fitting approach — do it AS a fix-wat job

This is a natural second customer for the arc-251 self-hosted runner, and a good test of it on
**non-stdlib** files:

- Author `fix-form` rules in `fix-wat.wat` for the mechanical retired-form rewrites where they're
  decidable from the form alone:
  - `:()` → `:wat::core::nil` (type-slot keyword rewrite — span-edit, comment-faithful).
  - `:wat::core::define` head → `:wat::core::defn` **+ argspec reshape** (paren-pairs → `[name <- :T]`
    vector) — the harder one; the paren-pair → bracket transform is non-trivial and may not be fully
    mechanical (defaults, multi-arity). Scope carefully; some may be manual.
  - `main`-signature rewrite (drop the stdio params; rewrite their *uses* to `readln`/`println`) —
    likely **NOT** purely mechanical (body rewrites referencing the dropped params); treat as
    assisted, not automatic.
- Run the rules via `wat-scripts/` (the runner migrates the runners — pleasingly recursive), audit the
  diff, fix the residual by hand.
- Each rule added stays in the `fix-wat.wat` ledger (the accumulating history; arc 251 doctrine).

## Open questions before scoping

1. How much of the `define`→`defn` argspec reshape is mechanically decidable vs. needs a human/LLM
   pass? (The param-type and unit-type rewrites are mechanical; the structural reshape may not be.)
2. Is the goal "make every script run" (full migration) or "delete the dead ones + migrate the few
   still useful"? Some scripts may be obsolete (the telemetry `.db` path may have moved).
3. Should `wat-scripts/README.md` gain a "these are tested-current" gate so the graveyard can't
   silently re-form? (A CI smoke that `--check`s every script would make staleness a red build —
   extirpare's "check that fires," lifting the class above "convention.")

## Cross-references

- Trigger: arc 251 self-hosted runner (Song #96 *Again We Rise*); `wat-scripts/fixes/fix-macro-param-types.wat`
  is the current-syntax reference shape.
- The retired forms: arc 153 (unit→nil), arc 170 slice 1e (main `[]` + stdio services), arc 241.11/241.16 (`define` retired).
- `wat/fix.wat` — where the new `fix-form` rules would accrue.
