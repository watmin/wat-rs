# BRIEF — Stone 234.6 — `:wat::holon::defrecord` migration + HARD CUT retirement

**Status:** READY TO SPAWN.

**Predecessors:**
- Stone 234.2b (`:wat::Record::def` macro shipped at wat/Record.wat) — the replacement
- Stone 234.5 (`:wat::holon::*` auto-dispatch on Value::wat__Record) — the semantic-equivalence mechanism
- Stone 234.4.match (SHIPPED `bf329ebe` 11/11) — most recent arc 234 ship; SCORE template

## What to do

Migrate ALL `:wat::holon::defrecord` callers in wat-rs to `:wat::Record::def`. After migration, HARD CUT retire the OLD macro: delete `wat/holon/defrecord.wat`; remove registry entries in `src/stdlib.rs`; update `wat/Record.wat:76` D12 comment (no co-existence claim).

After this stone ships, `:wat::holon::defrecord` is STRUCTURALLY UNREPRESENTABLE in wat-rs source. No transitional alias. No deprecation. The substrate refuses the legacy name.

Per the DESIGN's order-of-operations (D5), execute in this strict sequence:
1. Bulk find-replace `:wat::holon::defrecord` → `:wat::Record::def` in 7 caller files (everything EXCEPT `wat/holon/defrecord.wat` which gets deleted)
2. Run `cargo test --release --lib -p wat --no-fail-fast` — verify probes pass against new macro
3. Update `wat/Record.wat:76` D12 comment to affirmative naming
4. Delete `wat/holon/defrecord.wat`
5. Remove `:wat::holon::defrecord` registry entries in `src/stdlib.rs` (2 sites per audit)
6. Run `cargo build --release -p wat` + full test suite — verify substrate-as-teacher cascade catches any missed caller

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.6.md` — 10 locked decisions + 10 trap-doors
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.6.md` — 11-row scorecard
3. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.match.md`** — most recent arc 234 ship (template for SCORE shape; mirror the discipline)
4. **`docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.2.md`** — migration cascade precedent (47-fn sibling-flip; substrate-as-teacher cascade in similar shape)
5. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — the replacement macro's shipment record
6. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — auto-dispatch shipment (the semantic-equivalence proof)
7. `wat/Record.wat` (line ~76 for D12 comment update) + `wat/holon/defrecord.wat` (the file to delete)
8. `src/stdlib.rs` (search for `:wat::holon::defrecord` — 2 registry sites)

## Audit (already done; substrate truth)

```
8 files; ~75 references:
  56  tests/probe_arc227_stone2_defrecord.rs         (arc 227 v3 macro probe)
   6  wat/holon/defrecord.wat                        (the macro source — DELETE)
   4  tests/probe_diagnostic_typed_entities_reflection.rs
   4  tests/probe_diagnostic_defprotocol_dispatch.rs
   2  src/stdlib.rs                                  (registry — RETIRE)
   1  wat/Record.wat                                 (D12 comment — UPDATE)
   1  tests/probe_arc234_stone2b_defrecord_macro.rs
   1  tests/probe_diagnostic_polymorphic_type.rs
```

7 files for find-replace (everything except wat/holon/defrecord.wat which gets deleted).

## Implementation pattern

### Step 1 — bulk find-replace

```bash
# Find all callers
grep -rln ":wat::holon::defrecord" --include="*.wat" --include="*.rs" wat/ wat-tests/ tests/ src/ crates/ examples/ | grep -v "wat/holon/defrecord.wat"

# For each file, replace
:wat::holon::defrecord → :wat::Record::def
```

Use `sed -i` or `Edit` tool with `replace_all: true` per file. Verify each file's content after replacement.

### Step 2 — verify probes pass (substrate not yet retired)

```bash
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
cargo test --release --test probe_arc227_stone2_defrecord 2>&1 | tail -3
```

Both should pass. If probe arc 227 fails: investigate per T1 (test-body adjustment IF substrate-equivalent; STOP if substrate behavior actually differs).

### Step 3 — update D12 comment

`wat/Record.wat:76` currently has a co-existence claim. Replace with affirmative naming (DESIGN T4 example):

```
;; D12: :wat::Record::def is THE record-defining macro. Mints
;; Value::wat__Record with dual-form (struct + holon). Holon-form
;; access via :wat::holon::* auto-dispatch (Stone 234.5).
```

### Step 4 — delete the OLD macro source

```bash
rm wat/holon/defrecord.wat
```

### Step 5 — remove registry entries

In `src/stdlib.rs`, find the 2 `:wat::holon::defrecord` references (registry entries). Remove the entire registration calls.

### Step 6 — verify cascade

```bash
cargo build --release -p wat 2>&1 | tail -5  # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3  # 827/0
grep -rn ":wat::holon::defrecord" src/ wat/ wat-tests/ tests/ crates/ examples/  # 0 results
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"  # ≤ 54
```

If step 6 grep returns ANY results: substrate-as-teacher cascade caught a missed caller. Migrate it + re-run.

### Optional Step 4.5 — check loader file-list

If a loader has a hard-coded list of `wat/holon/*.wat` files, deleting `defrecord.wat` may leave a "file not found" error. Sonnet checks `src/lib.rs` / `src/stdlib.rs` / `src/runtime.rs` for any file-list mentioning `defrecord.wat`. If found, remove that entry alongside the file deletion.

### Docstring updates (T5)

Probe files whose docstrings reference the OLD macro by name should be updated to affirmative historical reference:

- `tests/probe_arc227_stone2_defrecord.rs` (header docstring lines 1, 11, 62 per audit) — change "Arc 227 Stone 227.2 v3 — User-defined types via `:wat::holon::defrecord` macro" to "Arc 227 Stone 227.2 v3 + Stone 234.6 migration — User-defined types via `:wat::Record::def` (formerly `:wat::holon::defrecord`)"

DO NOT rename the test file. Git history preservation > naming cleanliness.

## Discipline

- Touch ONLY: 7 caller files (find-replace) + `wat/Record.wat` (D12 comment) + `wat/holon/defrecord.wat` (delete) + `src/stdlib.rs` (registry retirement) + optionally loader-file-list (if found)
- DO NOT touch: any arc 234 historical artifacts; any arc 236 / arc 232 artifacts; holon-rs (STOP-4); lab repos (D3 workspace boundary)
- DO NOT commit (orchestrator commits)
- DO NOT mint transitional alias / deprecation warning / "defrecord-deprecated" form (D2 HARD CUT)
- DO NOT preserve `wat/holon/defrecord.wat` as deprecated-stub (D2 — full delete)
- DO NOT modify probe test ASSERTIONS beyond test-body adjustment traceable to macro shape change (T1 / STOP-11)

## Lib baseline handling

Expected: 827/0 (unchanged through all 6 steps). If ANY step regresses: investigate immediately.

The most likely failure point: arc 227 probe behavior preservation (T1). Sonnet investigates per T1 protocol.

## STOP triggers (REJECTION)

1. Unexpected compile errors not tracing to find-replace / registry retirement / file deletion
2. Lib baseline drops below 827
3. **120 min elapsed** (Mode A target 60-90 min)
4. holon-rs touched
5. Rust changes outside src/stdlib.rs (or loader file-list IF discovered)
6. arc 234 / arc 236 / arc 232 regression beyond test-body adjustment per T1
7. clippy > 54
8. Transitional alias / deprecation / "defrecord-deprecated" form minted
9. `wat/holon/defrecord.wat` preserved as deprecated-stub
10. Lab repos touched (workspace boundary violation)
11. Probe arc 227 test ASSERTIONS modified beyond what's required by macro shape change

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.6.md` (NEW). Capture:

- 11-row scorecard verbatim outputs
- File migration summary (per-file reference count before/after)
- Files touched (the 7 + Record.wat + stdlib.rs + deleted defrecord.wat + optional loader file-list)
- D12 comment update verbatim (before/after)
- Registry retirement: which lines in src/stdlib.rs got removed
- T1 outcome: did arc 227 probe pass on first try? Or did test-body adjustment surface?
- T9 outcome: did loader file-list need update?
- Defensive grep result: 0 references to `:wat::holon::defrecord` post-stone
- Honest deltas
- Rank-up evidence — predecessor SCOREs (236.2 cascade pattern + 234.4.match parity-stone discipline) effective?

Closing note: arc 234 substrate work COMPLETE. Stone 234.7 INSCRIPTION is the next move + closes arc 234.
