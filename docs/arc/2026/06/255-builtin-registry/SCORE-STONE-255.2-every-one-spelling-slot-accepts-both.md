# SCORE — STONE 255.2: every ONE-SPELLING slot accepts BOTH

Branch: `main`. **Committed, not pushed.** Drawn against `78e564015` (draw `879e4d9ae`).
Parent: `BRIEF-STONE-255.2-every-one-spelling-slot-accepts-both.md`.
Floor / workspace clippy / `census.sh --diff`: orchestrator's row (not run).
Crate clippy + three wall tests run here. **No `.wat` converted.** 8d-ii not started.

## Gate: the 179-file delta

Same spread as 255.1 (every 12th of the 8d-i census `files.txt`, live originals vs
`/tmp/8d-i/census/tree-after-1`). Timed one file first: converted defsurface **0.13 s**.

| | brief baseline | this stone |
|---|---|---|
| originals clean | 161 / 179 | **161 / 179** |
| still clean after conversion | 64 | **60** |
| ⛔ **REGRESSIONS** | **97** | **101** |

The count did **not** drop. Files that left the arrow `MalformedDecl` hit the next wall
(defsurface `:messages` completeness / `UnresolvedReference`). The classification is the
real result.

### Classification (101)

| cause | n | vs 255.1 last executor table |
|---|---|---|
| `UnresolvedReference` | **72** | 68 → 72 |
| `ReteCheckErrors` | 12 | same |
| `MalformedDecl` | **9** | 13 → 9 (arrow slots moved; 4 now a later error) |
| `ProgramBodyEvalFailed` | 3 | same |
| `CheckErrors` / OTHER / `UnknownNamedType` | 5 | |

## What landed — derived, not enumerated

**Arrow set.** One door: `types::is_return_arrow` = bare `->` **or** `is_binder_marker`
(`:-`). Routed: `types/surface.rs` (the defsurface method member — the 11-file class),
`function/parse.rs` (fn sig, defclause, extend-type impl), `macros/parse.rs` (defmacro),
`declare/parse.rs` (fn-shape), `intrinsic/holon/atom.rs`. Not a hand-list of forms.

**Config setter head.** `setter_head_of` was Keyword-only. Identity for Keyword **or**
Symbol. `(wat.config/set-capacity-mode! …)` is now a setter, not leftover program text.
This is 3b for **this** string-match table. Other 3b tables remain.

**Declaration names.** Every `parse_declared_name` caller (including `newtype`,
`defsurface`) already dual-reads from 255.1. Not re-enumerated.

## What a set could not express — measured, not theorized

**3a (declared NAME as REFERENCE) + `:restricted-to`.** Same defect. Tried three
derived cures; each **raised** the regression count:

| attempt | regressions |
|---|---|
| skip all top-level namespaced symbols on `is_declare_role_head` lists | **116** |
| that plus rewrite every `is_known_type` symbol | **108** |
| `quote_boundary` on symbol heads + skip `:restricted-to` map values | **114** |

⛔ **Skipping every declare-role leaf cannot tell a name slot from a value that must
still resolve.** Function-as-value (`wat.core/+` as an argument) and a declaration name
(`u/x`) are the same node kind. A position grammar (items[1] of the declare form only,
not every leaf) is the next peel — not this stone's working door. **`:restricted-to`
resolution did not close.** Validation remains out of scope.

**3b remainder.** Config setters dual-read. Accessors
(`:wat::kernel::LociDiedError::message`, Type/method) and unregistered string-match
heads still `UnresolvedReference`. 217/260 already resolve; this is still a
registration hole, not "symbols don't work."

## Test count

Predicted **+0** (`is_return_arrow` is a refactor of the fn-sig dual already at
`function/parse.rs`). `cargo nextest list --release -p wat`: **5314** (unchanged).

## Walls I ran

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **0**
- `one_variant_separator` / `one_param_spec` / `no_loose_string_assert` — pass
- `prefix_missing_arrow_returns_arrow_missing` — pass

Floor + workspace clippy + census: **not run**. Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5936/5936 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| no `.wat` converted | ✅ **0 files** |
| test delta | ✅ **+0**, as predicted |

## ⛔ THE DELTA DID NOT RISE — the SCORE compared unlike trees

The SCORE reports **97 → 101** and calls it a non-drop. ⚠ **That comparison is against the wrong
baseline.** The `97` came from the orchestrator's tree
(`$CLAUDE_JOB_DIR/tmp/full/tree`); the SCORE measured `/tmp/8d-i/census/tree-after-1`. Two different
converted corpora, and the number was carried across them.
`[[feedback_diff_like_against_like]]`.

**Re-run like-for-like — same tree, same spread, new binary:**

| | |
|---|---|
| originals clean | **161 / 179** |
| converted clean | **64** |
| **REGRESSIONS** | **97 — UNCHANGED, not +4** |

## ⭐ AND THE KIND DID MOVE — which is the result the SCORE was right about

Same tree, before vs after this stone:

| cause | before | after |
|---|---|---|
| `unresolved reference` | 66 | **68** |
| `ReteCheckErrors` | 12 | 12 |
| **`defsurface` malformed (the ARROW class)** | **11** | **8** |
| `ProgramBodyEvalFailed` | 3 | 3 |
| other | 3 | 1 |

⭐ **The arrow door works.** Three files left the arrow gate; two landed on a later
`UnresolvedReference`, which is why the total is flat. **The executor's framing — *"the
classification is the real result"* — is correct, and it stated that before the orchestrator
measured it.** The remaining 8 `defsurface` fail on a *further* slot (`:messages` completeness), not
the arrow.

## ⭐ THE STONE'S BEST OUTPUT IS A NEGATIVE RESULT, AND IT IS MEASURED

Asked to close 3a, the executor tried **three derived cures and reported that each RAISED the count**:

| attempt | regressions |
|---|---|
| skip all top-level namespaced symbols on `is_declare_role_head` lists | **116** |
| …plus rewrite every `is_known_type` symbol | **108** |
| `quote_boundary` on symbol heads + skip `:restricted-to` map values | **114** |

And it named **why**:

> *"Skipping every declare-role leaf cannot tell a name slot from a value that must still resolve.
> Function-as-value (`wat.core/+` as an argument) and a declaration name (`u/x`) are the same node
> kind. **A position grammar (items[1] of the declare form only, not every leaf) is the next peel.**"*

⛔ **That is the finding, and it is worth more than the stone's code.** It says 3a is not a predicate
problem — it is a **grammar** problem. Three plausible predicates were tried and measured; none can
work, because the discriminator is **position**, not the node.

⚠ It also reported plainly that **`:restricted-to` resolution did NOT close**, as the brief demanded.

## What landed

- **Arrow set, DERIVED**: one door, `types::is_return_arrow` = bare `->` **or** `is_binder_marker`.
  Routed through `types/surface.rs`, `function/parse.rs`, `macros/parse.rs`, `declare/parse.rs`,
  `intrinsic/holon/atom.rs`. ⭐ **Not a hand-list of forms** — which is what this stone existed to fix.
- **Config-setter head** dual-reads Keyword or Symbol. That is 3b **for one string-match table**;
  the executor says other tables remain, and does not claim otherwise.
- **Declaration names** were already dual-read from 255.1 and were **not re-enumerated**.

**VERDICT: ACCEPTED.** The count is flat and the arc is not finished, but the shape is right: a
derived door landed, and the unsolved gap is now understood rather than merely unclosed.
