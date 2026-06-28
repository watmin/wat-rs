# SCORE — 293.4d: field members are accessors too → THE ACCEPTANCE DEMO GREEN (R1 FORMA SOLA SUFFICIT)

**Verdict: GREEN, weighed by the orchestrator's own re-run.** `cargo nextest run --release` = **4094 passed / 0 failed
/ 92 skipped**. The acceptance demo runs: a foreign built-in taught to be a Shape it never declared, field and method
backing the same accessor interchangeably, dispatch routing by runtime shape. **R1 — FORMA SOLA SUFFICIT — is fulfilled.**

## Scorecard (each row re-run by the orchestrator)
| # | what | result |
|---|---|---|
| 1 | focused field-member probe GREEN | **PASS** — `(:t::probe)` → "red" |
| 2 | THE ACCEPTANCE DEMO GREEN | **PASS** — `(:geo::demo)` = `"red circle(r=2) area=12.56636  \|  blue square(s=3) area=9  \|  grey vector[3] area=3"` |
| 3 | 293.4a/b/c un-regressed | **PASS** (all four prior probes) |
| 4 | satisfaction still bounded | **PASS** — `non_extended_foreign_type` rejects |
| 5 | whole workspace green | **PASS** — 4094 / 0 / 92 (own forced run) |

## What shipped — the last seam of "methods are accessors"
Every surface member (Field OR Method) is an accessor `:T/name`; dispatch `:Surface/name s → :<T>/name` and satisfaction
`:T/name resolves` are uniform across both. The change was a **broadening**, not a rewrite:
- **`src/check.rs`** (the 293.4b surface call arm) — member-find broadened from `Method`-only to `Field | Method`; a Field
  member types as the field's `TypeExpr`, a Method member as before.
- **`src/runtime.rs`** (the 293.4b dispatch arm) — the member guard broadened to Field-or-Method; the `:<T>/<name>`
  lookup is UNCHANGED — a record's field accessor is already registered at `:<T>/<field>` by `register_record_methods`
  (runtime.rs:1443; STOP-1 clear).
- **`src/types/surface.rs`** — a Field member is satisfied by a struct field OR a `:<T>/name` accessor (the union — so a
  foreign type backs a Field member with an extend-type METHOD). Bounded: neither → not satisfied (STOP-3 clear).
- **the demo `.wat`** — fixed 3 things: extend target `:wat::holon::Vector` → `:wat::core::Vector` (match the
  constructed `Value::Vec`); the method member sigs moved INSIDE the single member vector; `str` multi-arg → `string::concat`.

## Honest deltas (carried, not hidden)
1. **The demo string differs from the DESIGN's promise in f64 Display only** — `r=2` not `r=2.0`, `area=9` not `area=9.0`
   (Rust f64 Display drops trailing `.0`). The dispatch/monkeypatch behavior is exactly as promised.
2. **⚠ FOUND — a silent-swallow in `parse_defsurface` (the NEXT strike, 293.4d-fix).** The demo's ORIGINAL surface had
   the method members OUTSIDE the member vector (`[color <- :String] (area …) (label …)` — the DESIGN.md `definterface`
   form). `parse_defsurface` (arity-4 branch) read `[color]` as the members and **SILENTLY DROPPED** `(area)`/`(label)`.
   A surface can be declared weaker than written, no error → satisfaction passes types it shouldn't. **extirpare: this is
   a correctness hole the arc's own demo hit — fix at the root (parse_defsurface must ERROR on unexpected extra args),
   with a RED probe.** Queued as the immediate next micro-strike.
3. **DESIGN.md's acceptance-test snippet is stale** — it shows `definterface` with separate-arg method members; the
   shipped + crowned form is `defsurface` with all members in one vector. Update the DESIGN snippet (amend-with-recognition).

## R1 FULFILLMENT
`probe_arc293_acceptance_demo` RED→GREEN. The arc's thesis is demonstrated, not just built. R1's FORMA SOLA SUFFICIT
turns from prophecy to **PROBATUM EST** (293/REALIZATIONS.md R1 updated).

## Next
**293.4d-fix** (the silent-swallow extirpare — immediate) → **293.4e** annihilate `defprotocol` (ONE live use
`:wat::spawn::Locus`, `wat/spawn.wat:224`; rip the Rust machinery across 6 files; retirement-table the head). Then
293.1-owed `src/aggregate/` home + 293.5 close → `Seqable` → 118.
