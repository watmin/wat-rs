# SCORE — Arc 221 Stone 221.1 — `HolonAST::Char(char)` leaf in holon-rs

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 8/8 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `Char(char)` variant added to enum | PASS | `holon-rs/src/kernel/holon_ast.rs` — `Char(char)` placed after `Bool` in the Vocabulary leaves section; doc comment cites arc 220 Stone 220.2 + BMP-only surface concern; PRIM_TAG_CHAR distinctness documented |
| 2 | Enum doc-comment count updated | PASS | "Twelve variants" → "Thirteen variants" at the top of the file (line 47 after edit); inline one-character change; not deferred |
| 3 | Debug + PartialEq + Hash arms | PASS | Debug: `f.debug_tuple("Char").field(c).finish()`; PartialEq: `(HolonAST::Char(a), HolonAST::Char(b)) => a == b`; Hash: `(*c as u32).hash(state)` after outer `discriminant(self).hash(state)` fires |
| 4 | canonical_edn_holon arm | PASS | `HolonAST::Char(c) => write_atom_payload(&mut out, PRIM_TAG_CHAR, &(*c as u32).to_le_bytes())` — 4-byte LE u32 payload; `char_distinct_from_string` + `char_distinct_from_symbol` tests confirm byte-level distinctness |
| 5 | PRIM_TAG_CHAR constant | PASS | `const PRIM_TAG_CHAR: &str = "char";` added alongside PRIM_TAG_STRING/I64/F64/BOOL at the PRIM_TAG block |
| 6 | Constructor `char_(c: char)` | PASS | `pub fn char_(c: char) -> Self { HolonAST::Char(c) }` in `impl HolonAST` block after `bool_`; trailing underscore avoids Rust keyword collision; doc comment cites BMP-only surface concern |
| 7 | 3 new tests | PASS | `char_leaf_round_trip` (variant equality + Hash determinism via DefaultHasher), `char_distinct_from_string` (canonical bytes asserted ≠), `char_distinct_from_symbol` (canonical bytes asserted ≠); all 3 PASS in `cargo test --release` |
| 8 | All test suites + clippy green | PASS | `cargo build --release` — 0 warnings, OK; `cargo test --release` — 257 unit/integration + 19 doc tests, 276 PASS / 0 FAIL (includes all 3 new Char tests); `cargo clippy --release -- -D warnings` — 0 warnings; wat-rs untouched (confirmed via `git -C wat-rs diff --name-only` → empty) |

## Deltas from EXPECTATIONS

### Delta 1 — Exhaustive match sites beyond the BRIEF's 5 arms

The BRIEF listed 5 arms (Debug / PartialEq / Hash / canonical_edn_holon / constructor). Four additional exhaustive match sites in the file required Char arms to compile cleanly:

- `template()` method: `Char(_)` added to the leaf pass-through arm (alongside Bool/I64/F64/String/Symbol)
- `collect_slots()` helper: `Char(_)` added to the no-op leaf arm
- `collect_ranges()` helper: `Char(_)` added to the no-op leaf arm
- `encode()` function: `Char(c)` arm added using `leaf_seed(PRIM_TAG_CHAR, &(*c as u32).to_le_bytes(), vm.global_seed())` pattern — consistent with Bool/I64/F64

All four are mechanical mirrors of the existing Bool/I64 leaf pattern. No new logic introduced. STOP-1 did not trigger — Rust's exhaustive-match compiler caught every site.

### Delta 2 — holon-rs branch is `main`, not `arc-170-gap-j-v5-deadlock-state`

BRIEF says "Branch: same as wat-rs's current." The wat-rs active branch is `arc-170-gap-j-v5-deadlock-state`; holon-rs active branch is `main`. Surfaced per BRIEF instruction ("surface as a question before committing anything"). Stone does not commit — orchestrator commits. Orchestrator decides whether to branch holon-rs or commit to main.

## Verification summary

```
holon-rs/ (working dir):
  cargo build --release                         — OK (0 warnings)
  cargo test --release                          — 276/276 PASS (257 unit + 19 doc)
  cargo clippy --release -- -D warnings         — 0 warnings

wat-rs/ contamination check:
  git -C wat-rs/ diff --name-only               — empty (no wat-rs files touched)
```

New tests confirmed passing:
```
test kernel::holon_ast::tests::char_distinct_from_string ... ok
test kernel::holon_ast::tests::char_distinct_from_symbol ... ok
test kernel::holon_ast::tests::char_leaf_round_trip     ... ok
```

## Files changed (1 file)

Holon-rs:
- `holon-rs/src/kernel/holon_ast.rs` (~+50 lines): Char(char) variant + doc comment count update + Debug/PartialEq/Hash/canonical_edn_holon/encode/template/collect_slots/collect_ranges arms + PRIM_TAG_CHAR constant + char_() constructor + 3 tests

SCORE doc (wat-rs docs dir, code changes in holon-rs):
- `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.1.md` (this file)

**Total: 1 modified source file + 1 new SCORE doc.**

## STOP triggers

- **STOP-1 (existing holon-rs test regression):** DID NOT TRIGGER. 276 tests PASS; new variant is purely additive.
- **STOP-2 (canonical_bytes tests confirm identity collapse):** DID NOT TRIGGER. `char_distinct_from_string` and `char_distinct_from_symbol` both PASS — PRIM_TAG_CHAR creates distinct canonical bytes byte-for-byte.
- **STOP-3 (90 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (wat-rs touched accidentally):** DID NOT TRIGGER. `git -C wat-rs/ diff` is empty.
- **EXTRA — interop handshakes NOT required this stone:** confirmed per BRIEF.

## Calibration check

- **Target runtime:** 30-60 min
- **Actual sonnet duration:** ~25 min (reading BRIEF + EXPECTATIONS + full file scan + 8 targeted edits + build/test/clippy verification + SCORE)
- **Within prediction band?** YES — under lower bound; BRIEF's holon-rs unfamiliarity adjustment was conservative; exhaustive-match sites added ~5 min but compiler caught them mechanically.

## Substrate state

- `HolonAST::Char(char)` minted as a proper primitive leaf — not a `String("char:a")` wrapper
- `PRIM_TAG_CHAR = "char"` distinct from `PRIM_TAG_STRING = "String"` — canonical bytes differ byte-for-byte
- `char_()` constructor available (trailing underscore per Rust keyword doctrine)
- Full Unicode `char` accepted at substrate; BMP-only enforcement is Stone 221.2 wat-rs surface concern
- `encode()` dispatches `Char(c)` via `leaf_seed(PRIM_TAG_CHAR, ...)` → deterministic VSA vector distinct from Symbol/String vectors

## Unblocks

- Stone 221.2 (wat-rs `value_to_atom` Char + Uuid arms + `is_atomizable` Char) — now unblocked
- Arc 220 Slice 5 closure chain (Stone 221.2 → 221.3 → ... → 221.6 INSCRIPTION)
