# Arc 198 BRIEF — `:wat::core::def-restricted` substrate primitive + `:wat::core::defn-restricted` defmacro sugar

**Arc:** 198 (independent — does not block arc 170 stones; can interleave between Stone B and Stone C)
**Task:** #327

## Goal

Mint TWO forms:

1. **`:wat::core::def-restricted`** — substrate primitive (Rust). Binds a name to a value AND records an allowed-caller-prefix whitelist. Walker enforces caller FQDN against the whitelist at type-check time.

2. **`:wat::core::defn-restricted`** — defmacro sugar (wat-level). Expands to `def-restricted` + `fn`. Free composition.

The restriction is a property of the BINDING, not of the fn shape. Future `defmacro-restricted` etc. come for free as defmacro sugar over `def-restricted`.

## Why this exists

Verified pattern from `wat/core.wat:202-206`:

```scheme
(:wat::core::defmacro
  (:wat::core::defn ...)
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))
```

`defn` is already a defmacro over `def`. The same shape applies to restrictions: `def-restricted` is the substrate primitive; `defn-restricted` lifts via defmacro.

The IMMEDIATE consumer is arc 170 Stone B's ad-hoc walker rule (already shipped at commit `2a071f0`). Stone B's `validate_join_result_user_namespace` is a special-case rule for two specific names. Once arc 198 ships, a future refactor (not in arc 198's scope) replaces Stone B's special-case rule with `def-restricted` declarations on the two `*_join-result` substrate fns.

## Form shape

### `def-restricted` (substrate primitive)

```scheme
(:wat::core::def-restricted
  :my::name                              ;; the symbol being bound
  [:wat::kernel:: :wat::test::]          ;; Vec of allowed-caller prefixes
  <value-expr>)                          ;; the value being bound (often a fn)
```

Three positional args. Same shape as `(def name value)` plus a Vec-of-keywords whitelist between them.

**Prefix matching rules:**
- Prefix ending in `::` (e.g., `:wat::kernel::`) → namespace prefix match (caller FQDN starts with this prefix)
- Prefix NOT ending in `::` (e.g., `:wat::kernel::readln`) → exact FQDN match
- Empty whitelist `[]` → no callers allowed (probably error or substrate-internal only — sonnet to discover the right behavior)

**Walker check** (at type-check time):
- For each call site of a restricted binding, look up the binding's whitelist
- Get the caller's enclosing FQDN (the fn/def the call site is INSIDE)
- Check caller FQDN against whitelist using prefix-or-exact rules
- If no match → emit `CheckError` with helpful message naming:
  - The restricted callee
  - The caller's FQDN
  - The whitelist
  - Suggestion: "this callable is restricted to callers in {prefixes}"

### `defn-restricted` (defmacro sugar)

```scheme
(:wat::core::defn-restricted
  :my::fn-name
  [:wat::kernel:: :wat::test::]
  (x :i64) -> :i64
  body)
```

Expands to:

```scheme
(:wat::core::def-restricted
  :my::fn-name
  [:wat::kernel:: :wat::test::]
  (:wat::core::fn (x :i64) -> :i64 body))
```

Mechanical defmacro. Lives in `wat/core.wat` adjacent to existing `defn` definition.

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. THIS BRIEF describes the TARGET SHAPE. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — parser handling for the new form, AST representation, CheckEnv storage extension, walker hook landing, error type variant, defmacro registration order. Do NOT trust orchestrator claims about substrate internals without grep verification.

## Substrate state pointers (verified by orchestrator)

- `wat/core.wat:202-206` — existing `defn` defmacro (the TEMPLATE for `defn-restricted`)
- `src/check.rs:3094` — `validate_join_result_user_namespace` (Stone B's special-case rule; arc 198 generalizes its mechanism)
- `src/check.rs:1939` — `check_program` (where Stone B's walker hooks in; arc 198 likely hooks similarly)
- `src/check.rs` — CheckError variants + Display + Diagnostic (Stone B's `JoinResultUserNamespace` is the pattern for arc 198's new variant)
- Arc 157 introduced `def` form (`docs/arc/2026/05/157-core-def-form/` for context if needed)
- Arc 166 introduced `defn` form (`docs/arc/2026/05/166-core-defn-form/` for context)

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read existing state.** `wat/core.wat:202-206` for the `defn` template. `src/check.rs:3094` for Stone B's walker shape. Existing `def` parser + AST.

2. **Write 5 tests FIRST** in `tests/wat_arc198_def_restricted.rs`:
   - **Positive prefix match:** caller in `:wat::kernel::*` namespace calls restricted fn declared with `[:wat::kernel::]` whitelist → check passes.
   - **Negative prefix mismatch:** caller in `:user::*` namespace calls same restricted fn → check fails with helpful message.
   - **Exact FQDN match:** restricted fn declared with `[:wat::kernel::specific-fn]` (no trailing `::`) → only that exact caller passes; siblings fail.
   - **Multi-prefix whitelist:** restricted fn declared with `[:wat::kernel:: :wat::test::]` → callers in either namespace pass.
   - **defn-restricted expansion:** `(defn-restricted name [prefixes] sig body)` works equivalently to `(def-restricted name [prefixes] (fn sig body))`.

   RUN; CONFIRM all 5 fail (forms not yet defined).

3. **Implement `def-restricted` substrate primitive.**
   - Parser: register the new keyword + accept the (name, prefix-vec, value) shape
   - AST: extend `WatAST` variant OR reuse `Def` with optional restriction field (sonnet's call)
   - CheckEnv: store per-binding `Option<Vec<String>>` allowed-prefix list
   - Walker: new `validate_def_restricted_caller_namespace` (or similar) hooked into `check_program`
   - CheckError: new variant `DefRestrictedCallerNotAllowed` with Display + Diagnostic
   - Runtime: evaluation is identical to `def` (binding happens; restriction is a CheckEnv concern, not runtime)

4. **Implement `defn-restricted` defmacro** in `wat/core.wat`. One-line expansion to `def-restricted` + `fn`.

5. **Build + run tests.** All 5 green.

6. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Baseline post-Stone-B: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error). Post-arc-198 must match.

7. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/` per `feedback_no_worktrees` (FM 7-bis). Absolute paths route to main tree.
- DO NOT modify Stone B's `validate_join_result_user_namespace` walker rule — refactoring it to use `def-restricted` is OUT of scope for arc 198. That's a follow-up step after arc 198 lands.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / arc 170 STONE BRIEFs / arc 170 STONE EXPECTATIONS / arc 170 STONE SCOREs.
- DO NOT touch existing `def` or `defn` forms — they remain unchanged. Arc 198 ADDS new forms; it doesn't modify existing ones.
- DO NOT mint `defmacro-restricted` or other variants — defer per user direction. Only `def-restricted` (primitive) + `defn-restricted` (sugar).
- DO NOT update USER-GUIDE / docs — that's a follow-up after the refactor lands.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.
- ANCHOR cwd at `/home/watmin/work/holon/wat-rs/`. Verify with `pwd` periodically. Do not let cwd drift to `/home/watmin/work/holon/` (frozen root).

## Scorecard (6 rows, YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::core::def-restricted` substrate primitive defined (parser + AST + eval + CheckEnv storage) | `grep -n "def.restricted\|DefRestricted" src/parser.rs src/check.rs src/runtime.rs src/ast.rs` (or wherever) shows the new form |
| B | Walker check for restricted caller-namespace defined + hooked into `check_program` | grep shows the new check fn + dispatch arm |
| C | `:wat::core::defn-restricted` defmacro defined in `wat/core.wat` | `grep -n "defn-restricted" wat/core.wat` shows the defmacro |
| D | 5 new tests pass — positive prefix + negative prefix + exact-FQDN + multi-prefix + defn-restricted-expansion | `cargo test --release -p wat --test wat_arc198_def_restricted` → all green |
| E | `cargo build --release --workspace --tests` clean | build output Finished |
| F | Workspace test failure count ≤ baseline (Stone B end: 4 pre-existing failures) | full workspace cargo test failures ≤ 4 |

## STOP triggers

- The parser doesn't accept a Vec-of-keyword positional arg cleanly → STOP and surface; may need different arg shape.
- CheckEnv storage doesn't have an obvious place for per-binding metadata → STOP; structural refactor may be needed.
- The defmacro expansion doesn't typecheck because of arg-shape mismatch → STOP; refine the form.
- Migration breaks existing tests (SHOULDN'T HAPPEN — change is purely additive) → STOP and investigate.
- > 5 unexpected substrate-finding surfaces → STOP; this arc's scope may need decomposition.

## Workspace baseline (commit `2a071f0`)

- `cargo build --release --workspace --tests`: clean per Stone B SCORE
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-arc-198 target:
- ≥ baseline + 5 passed (5 new arc 198 tests)
- ≤ 4 failed (no regressions; arc 198 is purely additive)

## Time-box

60-90 min predicted. Hard stop 150 min. If approaching stop, write a partial SCORE describing state-at-stop.

## On completion

Write `SCORE.md` (at `docs/arc/2026/05/198-defn-restricted/SCORE.md`). 6 rows YES/NO. Honest deltas — especially:
- Parser handling of Vec-of-keyword positional arg (any surprises?)
- AST representation (new variant vs extended Def)
- CheckEnv extension point
- Walker hook landing
- defmacro expansion shape
- Workspace test count vs baseline
- Calibration record (predicted vs actual)

## What this enables

After arc 198 ships:
- A follow-up step (separate from arc 198's scope) refactors Stone B's `validate_join_result_user_namespace` to use `def-restricted` on the two `*_join-result` substrate fns — eliminating the ad-hoc special-case rule
- Future restricted forms (`defmacro-restricted`, etc.) come for free as defmacros over `def-restricted` — no new substrate primitives needed
- Substrate-internal forms can declare their callable surface explicitly at definition site, readable in the source

The substrate teaches; we listen; we generalize once the pattern is real.
