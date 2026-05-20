# BRIEF — Arc 215 Stone 1 — `_infer` placeholder + literal completion

**Stone:** mint `:wat::core::_infer` type-placeholder + extend HashMap/HashSet inference + adjust `{...}` desugar + add `#{...}` literal.
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 120 min STOP.
**Depends on:** P1 (#403, commit 564d5e6) + P2 (#404, commit 3230a9d). Both shipped.
**Unblocks:** arc 214 Slice 4 (#385) — ProgramEnv can now use the literal directly with inferred types.

## Goal

Pivot literal desugaring from "auto-wrap everything in Atom" to "infer the concrete type via substrate's HM unification." Probe 5's class of failure (`Atom(HashMap)` runtime mismatch) is structurally eliminated because the desugar no longer goes through Atom for values.

## Pre-flight verified

- `:wat::core::_infer` doesn't exist anywhere in `src/types.rs` or `src/check.rs` (confirmed via grep).
- Existing tests green pre-spawn:
  - `cargo test --release --test probe_brace_map_literal` → 9/9 PASS
  - `cargo test --release --test probe_hashmap_ctor_vector_symmetric` → 9/9 PASS
  - `cargo test --release --test wat_arc169_struct_destructure` → 11/11 PASS
- `infer_hashmap_constructor` at `src/check.rs:10584` — currently emits MalformedForm if `args[0]` or `args[1]` isn't a type-keyword
- `infer_hashset_constructor` at `src/check.rs:9702` — currently emits MalformedForm if `args[0]` isn't a type-keyword
- Both already fall back to `fresh.fresh()` for the malformed path — the HM machinery is THERE; we just need to route `_infer` to it instead of erroring

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/` — stay inside; never touch holon root repo (frozen)
- Branch: `arc-170-gap-j-v5-deadlock-state` — your work lands here
- NEVER use `.claude/worktrees/*` paths
- Linux-only; no Windows/macOS/BSD
- Zero Mutex; if you reach for `std::sync::Mutex/RwLock/CondVar`, STOP and read `docs/ZERO-MUTEX.md`
- No `--no-verify`; no skipped hooks
- `cargo test` (through `tests/test.rs`) is the verification path

## Your scope (sonnet ships)

1. **Mint `:wat::core::_infer`** in `src/types.rs` (or wherever registered keyword-types live)
   - Add it to the registered-keyword-types list so `parse_type_expr` accepts it without errors
   - Represent at the TypeExpr level as a fresh type variable (or a dedicated `TypeExpr::Infer` variant that immediately translates to a fresh during inference)
   - Document at the definition site: "Placeholder for HM-style type inference; appears in type-arg slots of parametric constructor calls to delegate type to check.rs"

2. **Extend `infer_hashmap_constructor` (`src/check.rs:10584`)**
   - Before the MalformedForm path: if `args[0]` is `:wat::core::_infer`, set `k_ty = fresh.fresh()` (don't error)
   - If `args[1]` is `:wat::core::_infer`, set `v_ty = fresh.fresh()` (don't error)
   - The existing unification loop (lines 10664+) handles the rest — verifies all keys unify against `k_ty`, all values unify against `v_ty`, substitution resolves the fresh variables to concrete types
   - When both are `_infer` AND the literal is empty (no pairs), `k_ty` and `v_ty` stay as fresh variables — that's the existing HM-correct behavior

3. **Extend `infer_hashset_constructor` (`src/check.rs:9702`)**
   - Same pattern: if `args[0]` is `:wat::core::_infer`, set `t_ty = fresh.fresh()`
   - Existing unification loop verifies element types
   - Empty `#{}` → `t_ty` stays fresh

4. **Adjust `{...}` parser desugar** (`src/parser.rs`)
   - Currently emits `(:wat::core::HashMap :wat::core::keyword :wat::holon::HolonAST :k (:wat::holon::Atom v))`
   - Change to emit `(:wat::core::HashMap :wat::core::keyword :wat::core::_infer :k v)` — V slot is `_infer`; values pass through without Atom wrap
   - K remains `:wat::core::keyword` explicitly (structural rule; non-keyword keys still rejected at parse via `MalformedBraceLiteral`)
   - Empty `{}` desugars to `(:wat::core::HashMap :wat::core::keyword :wat::core::_infer)` — both K and V "concrete" but V infers fresh

5. **Add `#{...}` parser dispatch** (`src/parser.rs`)
   - New parser rule: `#{x y z ...}` → `(:wat::core::HashSet :wat::core::_infer x y z ...)`
   - The `#` prefix before `{` is the discriminator; existing `{...}` brace-form parser stays
   - Empty `#{}` desugars to `(:wat::core::HashSet :wat::core::_infer)`
   - Errors:
     - `#{...` (unclosed) → existing brace-unclosed error
     - `#{:k v}` with key/value pairs → treat as set (no key/value distinction); user wanted a set, gets a set; no special diagnostic
   - **Note:** `#{...}` does NOT have key/value structure; all elements are values. No structural rule beyond "must be valid wat forms"

6. **Probe matrix** — new `tests/probe_arc215_collection_literal_inference.rs`:
   - **`{...}` probes (extend P2's coverage):**
     1. Single pair with inferred V: `{:foo 42}` → length 1; get :foo returns Some(42) (i64, not HolonAST-wrapped)
     2. Multi pair with inferred V: `{:a 1 :b 2 :c 3}` → length 3; get :b returns Some(2)
     3. String-valued: `{:a "hello" :b "world"}` → length 2; get :a returns Some("hello")
     4. **Nested map (Probe 5 resolution):** `{:outer {:inner 42}}` → outer length 1; get :outer returns Some(inner-map); inner length 1; type-checks AND runtime-works
     5. Mixed-value-type rejection: `{:a 1 :b "two"}` → check fails with TypeMismatch naming the offending value's span
     6. Empty literal: `{}` → length 0; type-check succeeds with fresh type variables (concrete-type may surface later if value is used)
   - **`#{...}` probes:**
     7. Empty set: `#{}` → length 0
     8. Single element: `#{42}` → length 1; contains 42 returns true
     9. Multi element: `#{1 2 3}` → length 3; contains 2 returns true
     10. Dedup at construction: `#{1 1 2 2 3}` → length 3 (dedup happens)
     11. Mixed-type rejection: `#{1 :foo "x"}` → check fails with TypeMismatch
   - **Cross-literal:**
     12. Map of sets: `{:a #{1 2} :b #{3 4}}` → outer V = HashSet<i64>; both inner sets have length 2

7. **Documentation:**
   - **`docs/WAT-CHEATSHEET.md` § 8** — update to reflect the new `{...}` / `#{...}` desugar shapes; add a row for `_infer`; note that explicit verb-form with `:T` types still works
   - **arc 058 row** — find the live arc-058 spec file (most-recently-touched row is the model); add a row for `_infer` mint + literal completion
   - **`docs/CONVENTIONS.md`** — if it has a "type-placeholders" section, add `_infer`; if not, add a brief mention near the type-keyword namespace docs

8. **Retroactive amendment to P2's SCORE:**
   - File: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-PARSER-PIVOT-P2-MAP-LITERAL.md`
   - Add a **Probe 5 LIMITATION resolution** subsection at the end stating arc 215 Stone 1 resolves the limitation; cite this stone's commit
   - Do NOT rewrite the original SCORE rows — those are historical record per `feedback_inscription_immutable`
   - The amendment is APPENDED at the end as a forward-reference

9. **SCORE doc** for this stone: `docs/arc/2026/05/215-collection-literal-inference/SCORE-215-STONE-1.md`
   - Row count: ~22 rows (similar to P1/P2 scorecards)
   - Mode declaration (A)
   - Honest deltas section
   - PASS/FAIL per row with citation

## Out of scope (DO NOT TOUCH)

- **`[...]` retarget** — existing `WatAST::Vector` path stays; future arc may unify literal paths
- **`'(...)` list literal** — needs task #283 first
- **Match-arm patterns** for `{...}` / `[...]` / `#{...}` — task #402 separate
- **WARD-PASS** — parser + check + types out-of-zone per `feedback_ward_zone_comms_only`
- **INTERSTITIAL entry** — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`
- **`Atom` signature changes** — the Atom polymorphism stays load-bearing for explicit-construction use cases; we just stop forcing literals through it
- **Backporting `[...]` to use `_infer`** — design choice deferred; not in this stone

## STOP triggers

- **STOP-1** — `_infer` integration with HM unification breaks something subtle. The fresh-variable approach should be clean (existing fall-back in the same functions uses `fresh.fresh()`), but if you discover the unifier has special-case handling that interacts poorly with the new path, STOP, surface the interaction, ask for direction.
- **STOP-2** — `parse_type_expr` rejects `:wat::core::_infer` even after adding it to the registered list. STOP, surface what's missing.
- **STOP-3** — existing tests fail after the `{...}` desugar change (other than the expected P2-probe updates). Some downstream consumer may depend on the old `HolonAST` V shape. STOP, surface the failure, ask for direction.
- **STOP-4** — time hits 120 min with any deliverable incomplete. STOP, report what shipped, defer remainder.

## Verification command

```
cargo build --release
cargo test --release --test probe_arc215_collection_literal_inference -p wat
cargo test --release --test probe_brace_map_literal -p wat                  # P2 probes
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat       # P1 probes
cargo test --release --test wat_arc169_struct_destructure -p wat              # arc 169
cargo clippy --release -- -D warnings
```

P2's probes may need adjustment after the desugar change:
- Probe 5 (`probe_5_map_of_map_auto_wrap_limitation`) should now SUCCEED (not LIMITATION). Update the probe to assert success instead of failure; rename the test if helpful. Keep the historical record in the SCORE amendment.
- Probe 9 (keyword in binder) stays as-is — orthogonal to this stone.

## Style conventions (orchestrator-side)

- No realization-voice content (INTERSTITIAL writing is orchestrator-direct post-ship)
- No commits during sweep; orchestrator commits after review
- SCORE doc honesty: PASS only what genuinely passes; log honest deltas explicitly

## When you finish

Report back with:
- Final PASS count out of ~22
- Any honest deltas
- Verification command output summary (build / probe counts / clippy)
- Elapsed time
- Anything you discovered that wasn't in the BRIEF (substrate gaps, test rot, ripple)

Now read this BRIEF + EXPECTATIONS, then execute.
