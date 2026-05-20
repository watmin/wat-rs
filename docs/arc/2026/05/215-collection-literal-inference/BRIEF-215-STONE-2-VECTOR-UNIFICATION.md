# BRIEF — Arc 215 Stone 2 — `[...]` Vector unification under `:wat::type::Infer`

**Stone:** unify `[...]` expression-position literal with the `:wat::type::Infer` mechanism shipped by Stone 1.
**Type:** Sonnet Mode A.
**Time budget:** 30-50 min target; 75 min STOP.
**Depends on:** Stone 1 (#406, commit 2a93fed) — `:wat::type::Infer` minted; HashMap + HashSet inference extended.
**Closes:** arc 215's stated goal of literal completion (except `'(...)` list literal which stays deferred per task #283).

## Goal

Complete the literal-syntax unification. Currently three literals work, but `[...]` uses a different mechanism than `{...}` and `#{...}`:

- `{...}` map literal — desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer k v ...)`; routes through `infer_hashmap_constructor` with `:wat::type::Infer`-aware fresh-variable substitution
- `#{...}` set literal — desugars to `(:wat::core::HashSet :wat::type::Infer x y z ...)`; routes through `infer_hashset_constructor`
- `[...]` vector literal — parser produces `WatAST::Vector(items, span)` direct AST variant; bypasses `infer_list_constructor`; runtime evaluates directly to Vec

This stone routes `[...]` expression-position through the unified `:wat::type::Infer` machinery, eliminating the dual-mechanism class.

**User-visible behavior of `[1 2 3]` does NOT change.** This is structural unification, not semantic pivot.

## Failure-engineering framing

Class of failure: "two mechanisms for one concept" — future readers see `{...}` and `#{...}` routing through one path, `[...]` routing through another, and must learn both. The discipline says: one mechanism per concept; eliminate the dual-path class structurally.

## Pre-flight verified

- `:wat::type::Infer` minted (Stone 1; INFER_TYPE_PATH const at `src/types.rs`)
- `infer_list_constructor` exists at `src/check.rs:11085` — Vector-typing inference function; named "list" as arc 109 slice 1g retirement leftover (functionally Vector); pattern matches HashMap/HashSet's `parse_type_expr` + `fresh.fresh()` fallback
- `infer_list_constructor` is called from check.rs lines 4979, 4981, 4996 for `:wat::core::Vector` (+ legacy `:wat::core::vec` alias)
- Baseline tests green: P1 9/9, P2 9/9, arc 215 12/12, arc 169 11/11
- Parser path: `Token::LBracket` at `src/parser.rs:214` → `parse_vector_body` → emits `WatAST::Vector(items, span)`
- `WatAST::Vector` has multiple consumer sites in check.rs (line 2156, 3336, 4036, 4281, 4641, 7047, 7452, 7775, 8094, 8866, 9541 — these are mix of binder-position handlers and walker descents)
- The expression-position direct AST handler that infers Vector type for `[...]` is somewhere in check.rs (sonnet to locate); it currently does NOT route through `infer_list_constructor`

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/` — stay inside; never touch holon root repo (frozen)
- Branch: `arc-170-gap-j-v5-deadlock-state`
- NEVER use `.claude/worktrees/*` paths
- Linux only; no Windows/macOS/BSD
- Zero Mutex
- No `--no-verify`; no skipped hooks
- `cargo test` (through tests/test.rs) is the verification path

## Your scope (sonnet ships)

1. **Extend `infer_list_constructor`** (`src/check.rs:11085`) to accept `:wat::type::Infer` for T:
   - Mirror Stone 1's HashSet pattern: if `args[0]` is `:wat::type::Infer`, set `elem_ty = fresh.fresh()` (don't error)
   - The existing unification loop (lines 11132+) handles the rest — verifies all elements unify against `elem_ty`; substitution resolves the fresh variable to a concrete type from the actual elements
   - Empty `(:wat::core::Vector :wat::type::Infer)` (zero values) — `elem_ty` stays fresh; HM-correct behavior

2. **Identify the expression-position `WatAST::Vector` handler** in check.rs (probably one of lines 3336, 4641, 7047 — sonnet to confirm via tracing what fires for a bare `[1 2 3]` expression at check time).
   - Refactor it to route through `infer_list_constructor` with synthesized `:wat::type::Infer` as the type-arg
   - OR: have the parser emit `(:wat::core::Vector :wat::type::Infer x y z)` for `[...]` at expression position (likely cleaner)
   - Whichever path you pick, document the choice in the SCORE
   - **Critical constraint:** binder-position `WatAST::Vector` handling stays unchanged (tuple destructure at let-binder LHS, fn params, match patterns). Only expression-position routes through the unified path.

3. **Runtime considerations:**
   - If the parser emits the verb-call form at expression position, runtime evaluation should already work via the existing `eval_vector_ctor` (or wherever the `:wat::core::Vector` verb-call runtime handler lives)
   - If the parser keeps `WatAST::Vector`, the runtime direct handler stays; check.rs internally routes inference through `infer_list_constructor` but eval doesn't need to change
   - Sonnet's design choice in deliverable 2 determines which path; both should preserve existing runtime semantics

4. **Probe matrix** — `tests/probe_arc215_stone2_vector_unification.rs` (or extend existing):
   - **Regression** (existing `[...]` behavior preserved):
     1. `[1 2 3]` → Vec<i64>; length 3; first element 1
     2. `[1.5 2.5]` → Vec<f64>; length 2
     3. `["a" "b"]` → Vec<String>; length 2
     4. `[]` empty → Vec with fresh T; length 0
     5. `[true false true]` → Vec<bool>; length 3
   - **New explicit-infer path:**
     6. `(:wat::core::Vector :wat::type::Infer 1 2 3)` → Vec<i64>; equivalent to `[1 2 3]`
     7. `(:wat::core::Vector :wat::type::Infer)` empty → Vec with fresh T
   - **Mixed-type rejection** (verify no regression):
     8. `[1 "two"]` → check fails with TypeMismatch; position-named
   - **Explicit-type path unchanged:**
     9. `(:wat::core::Vector :wat::core::i64 1 2 3)` → Vec<i64>; existing P1-style explicit form
   - **Binder-position preservation:**
     10. `(:wat::core::let [x 1 y 2] ...)` — tuple-destructure-via-Vector still works; arc 169 / arc 167 path intact

5. **Documentation:**
   - **`docs/WAT-CHEATSHEET.md` § 8** — update to reflect `[...]` desugar shape; add row showing explicit-infer verb-form parallel; note that all three collection literals (`{...}`, `#{...}`, `[...]`) now share `:wat::type::Infer` machinery
   - **arc 058 row** — add audit history entry in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/INDEX.md` (lab repo; use `git -C` for sibling-repo ops)
   - **`docs/CONVENTIONS.md`** — if Type-placeholders section was added by Stone 1, extend with Vector unification note

6. **SCORE doc** at `docs/arc/2026/05/215-collection-literal-inference/SCORE-215-STONE-2.md`:
   - ~15-row scorecard
   - Mode declaration (A)
   - Honest deltas section
   - PASS/FAIL per row with citation
   - Document which design choice you took in deliverable 2 (parser-emits-verb-call vs check-internally-routes)

## Honest delta — intueri Level-1 finding to log (NOT in scope for this stone)

`infer_list_constructor` is named "list" but works on Vector. This is an arc 109 slice 1g retirement leftover (the `list` type was retired; the function wasn't renamed). Intueri Level-1 lie (name says one thing, body does another). Log this as a follow-up arc candidate in your SCORE's honest deltas section — DO NOT rename in this stone (renaming triggers call-site sweep across multiple sites; out of scope). Future arc territory.

## Out of scope (DO NOT TOUCH)

- **`infer_list_constructor` rename** — intueri Level-1 lie; logged as honest delta for future arc
- **`'(...)` list literal** — needs task #283 `:wat::core::List<T>` substrate
- **Match-arm patterns** for `[...]` — task #402 stays separate
- **WARD-PASS** — parser + check + types out-of-zone per `feedback_ward_zone_comms_only`
- **INTERSTITIAL entry** — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`
- **Existing `[...]` source-code call sites** — they don't need migration; behavior preserved
- **holon-rs** — separate crate; out of scope entirely

## STOP triggers

- **STOP-1:** the expression-position `WatAST::Vector` handler turns out to be deeply intertwined with other WatAST::Vector uses (e.g., shared walker code that handles both binder and expression positions). The "binder vs expression" distinction may be subtler than the BRIEF anticipates. STOP if you can't cleanly route ONLY expression-position through the unified path without affecting binder semantics.
- **STOP-2:** parser emit-verb-call-form approach breaks downstream walkers that pattern-match on `WatAST::Vector` for things other than tuple destructure. STOP, surface what walks Vector, ask for direction.
- **STOP-3:** any existing test fails after the unification (other than expected probe updates). STOP, surface the failure.
- **STOP-4:** 75 min elapsed with any deliverable incomplete.

## Verification command

```bash
cargo build --release
cargo test --release --test probe_arc215_stone2_vector_unification -p wat       # new probes
cargo test --release --test probe_arc215_collection_literal_inference -p wat    # Stone 1 preserved
cargo test --release --test probe_brace_map_literal -p wat                      # P2 preserved
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat           # P1 preserved
cargo test --release --test wat_arc169_struct_destructure -p wat                  # arc 169 preserved (binder paths)
cargo clippy --release -- -D warnings
```

Plus a workspace test pass: `cargo test --release` (broad regression check — any existing test that uses `[...]` should still pass).

## Style conventions

- No realization-voice content (INTERSTITIAL writing is orchestrator-direct post-ship)
- SCORE doc honesty: PASS only what genuinely passes; log honest deltas explicitly
- Position-named diagnostics — failures cite the offending span
- No commits during sweep; orchestrator commits after review
- `feedback_inscription_immutable` — Stone 1's SCORE rows stay as historical record; Stone 2 has its own SCORE doc

## When you finish

Report back with:
- (a) Final PASS count out of ~15
- (b) Any honest deltas (likely candidates: WatAST::Vector handler topology surprises; parser-vs-check design choice rationale; intueri Level-1 finding on `infer_list_constructor` naming logged)
- (c) Verification command output summary
- (d) Elapsed time
- (e) Anything you discovered that wasn't in the BRIEF (substrate gaps, test rot, ripple)

Now read this BRIEF + EXPECTATIONS, then execute.
