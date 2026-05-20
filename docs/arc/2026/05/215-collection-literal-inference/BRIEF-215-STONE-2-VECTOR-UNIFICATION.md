# BRIEF — Arc 215 Stone 2 — `[...]` Vector unification + `{...}` keyword-key lift

**Stone:** complete arc 215's Clojure-data-literal flexibility goal in one atomic ship — unify `[...]` expression-position literal with the `:wat::type::Infer` mechanism AND lift the keyword-key restriction on `{...}` so both K and V slots use the unified inference machinery.
**Type:** Sonnet Mode A.
**Time budget:** 45-75 min target; 90 min STOP.
**Depends on:** Stone 1 (#406, commit 2a93fed) — `:wat::type::Infer` minted; HashMap + HashSet inference extended.
**Closes:** arc 215's stated goal of literal completion (except `'(...)` list literal which is **permanently deferred** per LLM-first analysis — idiomatic Clojure usage of list literal is statistically zero; verb form `(:wat::core::List/of ...)` plus task #283's runtime LinkedList substrate suffice for the rare cases requiring linked-list semantics).

## Goal

Complete the literal-syntax unification with **two coordinated changes**:

### Change A — `[...]` Vector unification

Currently three literals work, but `[...]` uses a different mechanism than `{...}` and `#{...}`:

- `{...}` map literal — desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer k v ...)`; routes through `infer_hashmap_constructor` with `:wat::type::Infer`-aware fresh-variable substitution
- `#{...}` set literal — desugars to `(:wat::core::HashSet :wat::type::Infer x y z ...)`; routes through `infer_hashset_constructor`
- `[...]` vector literal — parser produces `WatAST::Vector(items, span)` direct AST variant; bypasses `infer_list_constructor`; runtime evaluates directly to Vec

This change routes `[...]` expression-position through the unified `:wat::type::Infer` machinery, eliminating the dual-mechanism class.

**User-visible behavior of `[1 2 3]` does NOT change.** This is structural unification, not semantic pivot.

### Change B — `{...}` keyword-key restriction lifted

Stone 1's `{...}` desugar pinned K to `:wat::core::keyword` (P2 inheritance) and rejected non-keyword keys at parse with `MalformedBraceLiteral`. This was an arbitrary restriction at the parser layer — substrate truth (arc 057 slice 3: `hashmap_key accepts HolonAST`) supports arbitrary HolonAST keys.

This change:
- Drops the parse-time `MalformedBraceLiteral` for non-keyword keys
- Changes `{...}` desugar K from `:wat::core::keyword` to `:wat::type::Infer`
- K and V both now use unified `:wat::type::Infer` inference (symmetric with `#{...}` and `[...]`)
- Type discipline preserved: K and V each must be uniform within one literal (mixed-K or mixed-V fails at check, position-named)

**User-visible behavior change:** `{1 "v" 2 "w"}` now parses cleanly (int keys); type-checks as `HashMap<i64, String>`. Previously this failed at parse.

ProgramEnv's keyword-key constraint moves to its proper layer — function signature unification at the spawn-program call site, not language restriction at parse.

## Failure-engineering framing

Two classes of failure eliminated by this stone:

1. **"Dual-mechanism for one concept"** — `{...}` and `#{...}` routed through unified path; `[...]` routed through direct AST handler. Change A unifies all three under `:wat::type::Infer`.

2. **"Arbitrary parser-layer restriction blocks idiomatic Clojure-data-literal syntax"** — keyword-key requirement was a CONVENTION imposed at parse time; substrate already supported arbitrary HolonAST keys. Change B lifts the restriction; K and V handled symmetrically.

After this stone, **arc 215's LLM-first claim is structurally operational**: any LLM that knows Clojure data literals can write meaningful wat literals with no friction. Three shapes (`{...}` `#{...}` `[...]`), one mental model (first-unit inference + uniform unification + verb form for power-user cases).

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

### Change A deliverables

1. **Extend `infer_list_constructor`** (`src/check.rs:11085`) to accept `:wat::type::Infer` for T:
   - Mirror Stone 1's HashSet pattern: if `args[0]` is `:wat::type::Infer`, set `elem_ty = fresh.fresh()` (don't error)
   - The existing unification loop (lines 11132+) handles the rest — verifies all elements unify against `elem_ty`; substitution resolves the fresh variable to a concrete type from the actual elements
   - Empty `(:wat::core::Vector :wat::type::Infer)` (zero values) — `elem_ty` stays fresh; HM-correct behavior

2. **Identify the expression-position `WatAST::Vector` handler** in check.rs (probably one of lines 3336, 4641, 7047 — sonnet to confirm via tracing what fires for a bare `[1 2 3]` expression at check time).
   - Refactor it to route through `infer_list_constructor` with synthesized `:wat::type::Infer` as the type-arg
   - OR: have the parser emit `(:wat::core::Vector :wat::type::Infer x y z)` for `[...]` at expression position (likely cleaner)
   - Whichever path you pick, document the choice in the SCORE
   - **Critical constraint:** binder-position `WatAST::Vector` handling stays unchanged (tuple destructure at let-binder LHS, fn params, match patterns). Only expression-position routes through the unified path.

### Change B deliverables

3. **Lift the keyword-key parse-time check on `{...}`** (`src/parser.rs` — the `parse_map_literal_body` function from Stone 1):
   - Drop the per-key `WatAST::Keyword(_, _)` structural check that emits `MalformedBraceLiteral` for non-keyword first child
   - Any value shape is accepted as a key at parse time; check phase handles type uniformity
   - The "odd-count" rule stays (alternating k/v pairs); only the keyword-only-key rule lifts

4. **Change `{...}` desugar K from `:wat::core::keyword` to `:wat::type::Infer`**:
   - Stone 1's desugar shape: `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer :k v ...)`
   - New desugar shape: `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer k v ...)`
   - K and V both inferred symmetrically; `infer_hashmap_constructor` already accepts `:wat::type::Infer` for K (Stone 1 extended this)
   - Empty `{}` desugar becomes `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer)` — both K and V fresh variables

### Common deliverables

5. **Runtime considerations:**
   - For Change A: if the parser emits the verb-call form at expression position, runtime evaluation works via the existing `eval_vector_ctor`; if parser keeps `WatAST::Vector`, runtime direct handler stays
   - For Change B: no runtime change — `eval_hashmap_ctor` already accepts any key shape (Stone 1's discovery in deltas E)
   - Sonnet's design choice in deliverable 2 determines Change A's path; document in SCORE

6. **Probe matrix** — `tests/probe_arc215_stone2.rs`:
   - **Change A — Vector regression** (existing `[...]` behavior preserved):
     1. `[1 2 3]` → Vec<i64>; length 3; first element 1
     2. `[1.5 2.5]` → Vec<f64>; length 2
     3. `["a" "b"]` → Vec<String>; length 2
     4. `[]` empty → Vec with fresh T; length 0
     5. `[true false true]` → Vec<bool>; length 3
   - **Change A — New explicit-infer path:**
     6. `(:wat::core::Vector :wat::type::Infer 1 2 3)` → Vec<i64>; equivalent to `[1 2 3]`
     7. `(:wat::core::Vector :wat::type::Infer)` empty → Vec with fresh T
   - **Change A — Mixed-type rejection:**
     8. `[1 "two"]` → check fails with TypeMismatch; position-named
   - **Change A — Explicit-type path unchanged:**
     9. `(:wat::core::Vector :wat::core::i64 1 2 3)` → Vec<i64>; existing P1-style explicit form
   - **Change A — Binder-position preservation:**
     10. `(:wat::core::let [x 1 y 2] ...)` — tuple-destructure-via-Vector still works
   - **Change B — non-keyword key acceptance (homogeneous-K):**
     11. `{1 "v" 2 "w"}` → `HashMap<i64, String>`; length 2; get 1 → Some("v")
     12. `{"a" 1 "b" 2}` → `HashMap<String, i64>`; length 2; get "a" → Some(1)
   - **Change B — mixed-K rejection at check:**
     13. `{1 "v" "two" "w"}` → check fails with TypeMismatch at key #2; diagnostic names i64 expected vs String got
   - **Change B — P2 Probe 6 behavioral flip:**
     14. Update P2's `probe_6_non_keyword_key_rejected_at_parse` in `tests/probe_brace_map_literal.rs`:
         - Old assertion: `{42 :v}` fails at parse with `MalformedBraceLiteral`
         - New assertion: `{42 :v}` parses cleanly; type-checks as `HashMap<i64, keyword>`
         - Test rename in-function (e.g., `probe_6_non_keyword_key_accepted_with_inferred_k`); keep historical note via doc comment

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
- **`'(...)` list literal** — PERMANENTLY deferred per LLM-first analysis (idiomatic Clojure usage is statistically zero; verb form + task #283's LinkedList suffice). Do NOT add reader-macro speculatively.
- **Match-arm patterns** for `[...]` or `{...}` — task #402 stays separate
- **WARD-PASS** — parser + check + types out-of-zone per `feedback_ward_zone_comms_only`
- **INTERSTITIAL entry** — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`
- **Existing `[...]` source-code call sites** — they don't need migration; behavior preserved
- **holon-rs** — separate crate; out of scope entirely
- **ProgramEnv specifics** (V=HolonAST vs V=String vs polymorphic) — arc 214 Slice 4 territory; not this stone
- **`:wat::runtime::ProgramEnv/*` accessor verbs** — Slice 4 (#385) territory

## STOP triggers

- **STOP-1:** the expression-position `WatAST::Vector` handler turns out to be deeply intertwined with other WatAST::Vector uses (e.g., shared walker code that handles both binder and expression positions). The "binder vs expression" distinction may be subtler than the BRIEF anticipates. STOP if you can't cleanly route ONLY expression-position through the unified path without affecting binder semantics.
- **STOP-2:** parser emit-verb-call-form approach breaks downstream walkers that pattern-match on `WatAST::Vector` for things other than tuple destructure. STOP, surface what walks Vector, ask for direction.
- **STOP-3:** existing tests fail after the unification or keyword-key lift (beyond the expected P2 Probe 6 flip + the probes in this stone). STOP, surface the failure.
- **STOP-4:** lifting the keyword-key restriction breaks something subtle in `infer_hashmap_constructor`'s K handling — Stone 1 extended it for `:wat::type::Infer` so this SHOULD work cleanly, but if K-inference + key-position-walking has any special handling tied to keyword-only assumptions, surface and ask.
- **STOP-5:** 90 min elapsed with any deliverable incomplete.

## Verification command

```bash
cargo build --release
cargo test --release --test probe_arc215_stone2 -p wat                            # new Stone 2 probes (18 expected)
cargo test --release --test probe_arc215_collection_literal_inference -p wat       # Stone 1 preserved (12 expected)
cargo test --release --test probe_brace_map_literal -p wat                         # P2 preserved (9 — Probe 6 flipped)
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat              # P1 preserved (9 expected)
cargo test --release --test wat_arc169_struct_destructure -p wat                    # arc 169 preserved (binder paths intact)
cargo clippy --release -- -D warnings
```

Plus a workspace test pass: `cargo test --release` (broad regression check — any existing test that uses `[...]` or `{...}` should still pass; non-keyword-keyed map literals previously failed at parse, but no existing test should be exercising that rejection path — if any does, that's a discovery to log).

## Style conventions

- No realization-voice content (INTERSTITIAL writing is orchestrator-direct post-ship)
- SCORE doc honesty: PASS only what genuinely passes; log honest deltas explicitly
- Position-named diagnostics — failures cite the offending span
- No commits during sweep; orchestrator commits after review
- `feedback_inscription_immutable` — Stone 1's SCORE rows stay as historical record; Stone 2 has its own SCORE doc

## When you finish

Report back with:
- (a) Final PASS count out of 22 (matching the EXPECTATIONS scorecard exactly)
- (b) Any honest deltas (likely candidates: WatAST::Vector handler topology surprises; parser-vs-check design choice rationale; intueri Level-1 finding on `infer_list_constructor` naming logged; existing test surprises from the keyword-key lift)
- (c) Verification command output summary
- (d) Elapsed time
- (e) Anything you discovered that wasn't in the BRIEF (substrate gaps, test rot, ripple)

Now read this BRIEF + EXPECTATIONS, then execute.
