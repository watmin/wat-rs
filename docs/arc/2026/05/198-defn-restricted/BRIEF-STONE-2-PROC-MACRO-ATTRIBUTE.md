# Arc 198 Slice 2 Stone 2 BRIEF — `#[restricted_to(...)]` proc-macro attribute

**Arc:** 198 slice 2, Stone 2 of 4.
**Task:** #328
**Predecessor:** Stone 1 (commit `51c69a1`) — `RestrictionEntry` struct + `inventory::collect!` + setup iteration are in place.
**Successors:**
- Stone 3 — apply `#[restricted_to(...)]` to `eval_kernel_*_join_result`
- Stone 4 — delete arc 170 Stone B's ad-hoc rule + update Stone B's 4 tests

## Goal

Mint the `#[restricted_to(...)]` proc-macro attribute in `crates/wat-macros/`. The attribute:

1. Parses positional string-literal args
2. Generates a sibling `inventory::submit! { wat::RestrictionEntry { wat_name, prefixes } }` alongside the annotated fn
3. The annotated fn body is passed through UNCHANGED

This stone is **proc-macro-only**: no application to real substrate fns yet (Stone 3); no Stone B deletion (Stone 4).

**Verification:** ONE probe fn annotated with `#[restricted_to(...)]` in a test file. Test asserts the inventory entry exists in `RestrictionEntry`'s collected set AND lands in `defined_value_restrictions` after `startup_from_source`.

## Form shape (settled per arc 198 architecture)

```rust
#[restricted_to(":wat::probe::test/fn", ":wat::", ":my::specific::fn")]
pub(crate) fn probe_fn(...) -> ... { ... }
```

**Arg parsing (sub-decision for sonnet):**

- **(a) Positional:** first arg = wat name; rest = allowed-caller prefixes (variadic)
- **(b) Named:** `#[restricted_to(name = "...", from = [":wat::"])]`

Default to (a) — matches Rust attribute conventions (e.g., `#[derive(...)]` is positional variadic). Sonnet's call if (b) reads dramatically cleaner.

**Prefix matching rules** (MUST match arc 198 slice 1's wat-side rules):
- Trailing `::` → namespace prefix match
- No trailing `::` → exact FQDN match

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — proc-macro arg parsing approach, codegen shape, sub-decision (a) vs (b), `RestrictionEntry` field types (`&'static str` vs owned `String` — likely `&'static str` since literals), test fixture shape. Do NOT trust orchestrator claims without grep verification.

## Substrate state pointers (verified)

- `crates/wat-macros/src/lib.rs` — proc-macro crate entry; existing `#[wat_dispatch]` precedent
- `crates/wat-macros/src/codegen.rs` — existing codegen patterns to mirror/extend
- `crates/wat-macros/Cargo.toml` — likely needs `syn` / `quote` deps (already present for wat_dispatch)
- `src/restriction_entry.rs` (Stone 1) — `RestrictionEntry` struct + `inventory::collect!`
- `src/freeze.rs` (Stone 1) — setup iteration at step 6.8 (drains `inventory::iter::<RestrictionEntry>` into `symbols.defined_value_restrictions`)
- `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` — Stone 1's verification test (precedent shape — probe submits a `RestrictionEntry` manually; this stone's test uses the attribute to generate that submit automatically)

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** All pointers above. Pay special attention to:
   - How `#[wat_dispatch]` is structured in `crates/wat-macros/src/lib.rs` + `codegen.rs` (the parser/codegen precedent)
   - How Stone 1's test wires `inventory::submit!` manually (this stone's test does the same via the attribute)

2. **Write tests FIRST** in `tests/wat_arc198_slice2_stone_2_attribute.rs`:
   - **Test 1 (single prefix):** probe fn annotated `#[restricted_to(":probe::test/single", ":wat::")]` — verify `defined_value_restrictions.get(":probe::test/single")` returns `Some(vec![":wat::".to_string()])` after startup
   - **Test 2 (multi-prefix):** probe fn annotated with 2+ prefixes — verify all prefixes in the entry
   - **Test 3 (exact-FQDN match):** probe fn annotated `#[restricted_to(":probe::test/exact", ":wat::specific::name")]` (no trailing `::`) — verify the entry preserves the exact-FQDN form
   - RUN; CONFIRM all 3 fail (attribute doesn't exist yet)

3. **Implement the proc-macro attribute** in `crates/wat-macros/`:
   - Add `restricted_to` to the proc-macro entry (in `lib.rs`)
   - Parse args: positional first = wat name string literal; rest = variadic prefix string literals
   - Codegen: pass through original fn unchanged + emit `inventory::submit! { wat::RestrictionEntry { wat_name: <first>, prefixes: &[<rest>...] } }`
   - Reference path to `wat::RestrictionEntry` (or `::wat::RestrictionEntry`) — sonnet decides the right path syntax for the generated code

4. **Build clean.** `cargo build --release --workspace --tests`.

5. **Run new tests.** All 3 green.

6. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (3 pre-existing — lifeline was flake; t6, totally_bogus, startup_error are stable failures).

7. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; absolute paths route correctly.
- DO NOT apply attribute to real substrate fns (`eval_kernel_*_join_result` etc.) — that's Stone 3.
- DO NOT delete Stone B's ad-hoc walker rule — that's Stone 4.
- DO NOT modify Stone 1's `RestrictionEntry` struct or iteration logic.
- DO NOT touch arc 198 slice 1's wat-side `def-restricted` / `defn-restricted` forms.
- DO NOT modify Stone A's `drain-and-join` helpers.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / this BRIEF / this EXPECTATIONS / superseded slice-2 monolithic BRIEF + EXPECTATIONS.
- DO NOT update USER-GUIDE / docs.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (5 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `#[restricted_to(...)]` proc-macro attribute defined in `crates/wat-macros/` | `grep -nA 5 "restricted_to" crates/wat-macros/src/lib.rs` shows the attribute registration + parser |
| B | Codegen emits `inventory::submit!` block with `RestrictionEntry { wat_name, prefixes }` | grep in `crates/wat-macros/src/` shows the codegen path |
| C | Variadic prefix arg parsing works (test with 1+ and 2+ prefixes both pass) | targeted tests pass |
| D | 3 new tests pass — single-prefix + multi-prefix + exact-FQDN | `cargo test --release -p wat --test wat_arc198_slice2_stone_2_attribute` → all green |
| E | Workspace test failure count ≤ baseline (3 pre-existing: t6, totally_bogus, startup_error; lifeline flake within rotation band) | full workspace cargo test failures ≤ baseline + flake variance |

## STOP triggers

- Proc-macro infrastructure in `wat-macros` doesn't extend cleanly to variadic string args → STOP and surface
- Codegen can't reference `wat::RestrictionEntry` from the consumer crate's generated code (path resolution issue) → STOP
- Sub-decision (a) positional vs (b) named runs into clear blocker → STOP, propose alternative
- Migration breaks existing tests (SHOULDN'T HAPPEN — purely additive) → STOP and investigate
- > 3 unexpected substrate-finding surfaces → STOP; this stone's scope may need decomposition

## Workspace baseline (commit `51c69a1`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures (t6 unquote, totally_bogus, startup_error) + lifeline flake (rotation band)

Post-Stone-2 target:
- ≥ baseline + 3 passes (3 new attribute tests)
- ≤ baseline failures (purely additive)

## Time-box

90 min predicted. Hard stop 120 min. If approaching stop, write partial SCORE describing state-at-stop.

## On completion

Write `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-2-PROC-MACRO-ATTRIBUTE.md`:
- 5 rows YES/NO with grep-able evidence
- Honest deltas: sub-decision (a) vs (b) chosen + rationale, codegen path reference, `RestrictionEntry` field type compatibility (string literal vs owned), workspace test count vs baseline
- Calibration record (predicted vs actual)

Return final summary: rows passed/failed + sub-decision (a vs b) + workspace delta + path to SCORE.

You are launching now. T-minus 0.
