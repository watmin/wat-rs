# BRIEF — Stone 241.6 — Phase 2 opens: `{...}` metadata-map storage on `def`; defn inherits

You are sonnet (the Shadowdancer). Phase 2 first stone. Substrate STORAGE for binding-level metadata; reflection verb (Stone 241.7) reads what this stone stores.

## What to do

Extend the `def` parser to accept an optional `{...}` HashMap between the binding name and the value-expr. Store metadata in SymbolTable. Update `defn` macro to thread metadata through. NO reflection verb (that's 241.7); NO HARD CUTs (those are 241.8-241.10).

### S1 — Extend `try_parse_fn_shape_def` at `src/runtime.rs:3868`

Current: accepts EXACTLY 3 items `(def :name <fn-form>)`. Extended:
- 3 items `(def :name <fn-form>)` — no metadata; UNCHANGED behavior
- 4 items `(def :name <metadata-map> <fn-form>)` — metadata at items[2]; value-expr at items[3]

Detection: items[2] is `WatAST::List` with head keyword `:wat::core::HashMap` → metadata-map; else → no metadata (3-item path).

When metadata detected, EXTRACT the key-value pairs from the HashMap list (skip the head + K-type + V-type, then alternating key/value pairs), STORE in SymbolTable.

### S2 — Plain-value `def` parser path (not fn-shape)

Find the substrate's plain-value def handling (probably in eval_define or a sibling fn). Same discrimination:
- `(def :name 42)` — 3 items; no metadata
- `(def :name {:k :v} 42)` — 4 items with HashMap at items[2]

Extend symmetrically.

### S3 — `defn` macro expansion threads metadata

Find the defn macro definition (likely in `wat/runtime.wat` or `wat/core.wat`). Current expansion:
- `(defn :name [args] -> :ret body)` → `(def :name (fn [args] -> :ret body))`

Extended expansion:
- `(defn :name {meta} [args] -> :ret body)` → `(def :name {meta} (fn [args] -> :ret body))`
- `(defn :name [args] -> :ret body)` → `(def :name (fn [args] -> :ret body))` (UNCHANGED)

Discrimination in the macro: if the form after the name is a HashMap, treat as metadata; else proceed as today.

### S4 — Storage: SymbolTable extension

Add `pub binding_metadata: HashMap<String, HashMap<String, WatAST>>` to SymbolTable (or use a more typed structure if the substrate has one — sonnet investigates).

Outer key: binding name (`:my::ns::name` as String). Inner key: metadata keyword (`:doc` as String). Inner value: the metadata value-WatAST.

When def-with-metadata succeeds, insert into binding_metadata.

### S5 — Empty `{}` rejection

Per FORM-COLLAPSE-NOTES: empty metadata is ILLEGAL (divide-by-zero). If the parser already rejects empty brace literals as `MalformedBraceLiteral` (parser.rs:69), Stone 241.6 inherits that. If not, def-level validation rejects with a clear error.

**Verify**: write/run a contract that tests `(def :x {} 42)` — if it fails at parse time (via `MalformedBraceLiteral` or similar), no Stone 241.6 work needed. If it succeeds at parse but def doesn't reject, add the rejection.

(Note: Stone 241.6 probe contract 06 verifies this; currently PASSES at HEAD which suggests the parser already rejects empty `{}` — confirm.)

### S6 — `def-restricted` (arc 203) UNCHANGED

`try_parse_fn_shape_def_restricted` at runtime.rs:3948 stays as-is. The HARD CUT replacing `def-restricted` with `def + {:restricted-to ...}` is a LATER stone (241.10 territory). Stone 241.6 adds the new metadata-map alongside; legacy continues.

## Discipline

- **`src/argspec/*` UNCHANGED.** Canonical home is exceptional + rune-free.
- **`src/lib.rs` UNCHANGED.**
- **Stone 241.1/2/3/4/5 probes UNCHANGED** at their current PASS counts (15+10+6+8 + Gate 1).
- **`try_parse_fn_shape_def_restricted` UNCHANGED** (arc 203 path stays).
- **No new ArgSpecError / ClauseFailureReason variants** in this stone.
- **No reflection verb minted** (Stone 241.7).
- **No HARD CUTs of legacy surface** (Stones 241.8-241.10).
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.6.md` — this doc
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.6.md` — D1-D10 + T1-T10 + STOP
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` — doctrinal source for `{...}` discrimination + empty-`{}`-illegal
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.5.md` — prior stone calibration
6. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 3855-4030 (try_parse_fn_shape_def + def_restricted + surrounding context)
7. `/home/watmin/work/holon/wat-rs/src/parser.rs` lines 220-290 (brace-form dispatch; how `{...}` parses to `(:wat::core::HashMap ...)`)
8. Search `grep -rn "macro.*defn\b" wat/ src/` to find the defn macro definition
9. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone6_def_metadata_map.rs` — 6-contract FM 2-bis probe (3 PASS / 3 FAIL at HEAD)
10. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.6.md` — scorecard

## Implementation sketch

1. Read substrate + probe + FORM-COLLAPSE-NOTES
2. Baseline check:
   - `cargo test --release --lib -p wat` (834 PASS)
   - `cargo test --release --test probe_arc241_stone6_def_metadata_map` (3 PASS / 3 FAIL at HEAD)
3. Find defn macro location: `grep -rn "defn" wat/ src/macros.rs | head` or similar
4. Find SymbolTable definition; pick the extension point for binding_metadata
5. **S1+S2**: extend def parsers (fn-shape + plain-value) for 4-item with metadata-map discrimination
6. **S4**: SymbolTable.binding_metadata insertion logic
7. **S3**: defn macro expansion update (thread metadata through)
8. **S5**: verify empty `{}` rejection still holds (or add)
9. Run Stone 241.6 probe; iterate until 6/6 PASS
10. Run lib tests + Stone 241.x probes; address any cascade
11. Final verification:
    - `cargo test --release --lib -p wat` ≥834
    - `cargo test --release --test probe_arc241_stone6_def_metadata_map` 6/6
    - All Stone 241.x probes preserved
    - Arc 237/238 probes preserved
    - `cargo build --release --tests --workspace` clean
    - `cargo clippy --release` ≤ 904
12. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.6.md`
13. **DO NOT COMMIT.** Orchestrator commits.

## STOP triggers — REJECTION

1. Compile errors not traced to migration sites
2. Lib < 834
3. 50 min elapsed
4. holon-rs touched
5. Files outside `src/runtime.rs`, `src/check.rs` (if needed), defn macro file (`wat/runtime.wat` or similar), `tests/probe_arc241_stone6_*`, SCORE doc, test files with assertion updates. `src/argspec/*` + `src/lib.rs` MUST stay unchanged. Stone 241.x probes MUST stay at current PASS counts.
6. Scope creep: minting metadata-of reflection verb (241.7); HARD CUTting def-restricted (241.10); HARD CUTting struct (241.8); new ArgSpecError / ClauseFailureReason variants; new namespaced home
7. Stone 241.6 probe < 6/6 PASS
8. Stone 241.x probes regress; arc 237/238 probes regress
9. Clippy > 904

## SCORE doc spec

Mirror SCORE-STONE-241.5.md structural shape (no vigilia section assuming legacy flat substrate; if you DO introduce a namespaced home, surface in honest deltas):

- Header (Mode A/B; runtime; one-line summary)
- 10-row Phase A scorecard
- 5-row structural verification
- Migration audit (per-file deltas)
- Final post-stone code shapes (verbatim discrimination logic + SymbolTable extension + defn macro update)
- Honest deltas (defn macro location; SymbolTable extension shape; any cascade)
- PHASE 2 OPENS inscription: metadata-map storage shipped; Stone 241.7 reflection verb queued
- NO Vigilia Convergence section (legacy flat substrate per DESIGN D7 default)

## Post-strike

Return with one-paragraph status covering: discrimination logic landed; SymbolTable extension approach taken; defn macro location + expansion update; Stone 241.6 probe 6/6; cascade depth.

Phase 2 opens with this stone. Five more stones to ship before arc 241 closes. The rhythm continues.
