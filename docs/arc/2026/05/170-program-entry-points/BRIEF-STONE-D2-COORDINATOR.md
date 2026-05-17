# Arc 170 Stone D2 BRIEF — coordinator-fn `run-threads` (coordinator form)

**Phase:** Second sub-stone of decomposed Stone D. Supersedes `BRIEF-STONE-D2.md` (nested-vectors form). See INTERSTITIAL § "design phase complete" (line 1764) + § "Stone D design pass" (line 2013-2110) for the iterative design conversation that settled on this form.

**Predecessors:**
- Stone A SHIPPED — `Thread/drain-and-join`
- Stone C1 SHIPPED — `ThreadPeer<I, O>` + verbs
- D1 SHIPPED — minimal `run-threads`, single-factory, positional-types call form (`wat/kernel/run_threads.wat:85-112`)
- Arc 200 SHIPPED — macro-layer Vector/List splice symmetry
- Arc 201 CLOSED — structured type-AST in reflection layer (signature-of-fn + extract-arg-names + extract-arg-types + Bundle/children + Bundle/first)

**Successor:** D3 — panic cascade + `ProcessGroupErr`.

## Goal

Replace D1's positional-types `run-threads` macro with the coordinator-fn form that scales uniformly from N=1 to N≥3. The new form uses arc 201's reflection chain to extract names + types from a coordinator anonymous fn; factories follow variadically. Coordinator body is ALWAYS a single delegating call to a named fn — that's the demonstrated + only-advertised pattern.

## Required call form

```scheme
(:wat::kernel::run-threads
  (:wat::core::fn
    [logger   <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
     counter  <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::i64>
     reporter <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
    -> :wat::core::String
    (:user::actual-run-threads-main-fn logger counter reporter))   ;; THE only advertised body shape
  (:app::logger-worker)
  (:app::counter-worker)
  (:app::reporter-worker))
```

**Coordinator-fn structural rule:** the inline `(:wat::core::fn ...)` body is ALWAYS a single delegating call to a named fn (e.g. `(:user::actual-run-threads-main-fn logger counter reporter)`). This is the documented + tested + USER-GUIDE-exemplified pattern. The inline fn carries the binder declarations (names + types) for reflection; the real work lives in a named fn that's independently testable, reflectable (arc 143/144), and reusable. Same pattern as `:wat::runtime::define-alias` (substrate-precedent for "macro generates a thin delegating wrapper").

## Required macro algorithm (per INTERSTITIAL § Q4 lines 112-122)

1. **Macro receives** the coordinator AST + N variadic factory call form ASTs
2. **At expand time** — eval the coordinator AST via computed-unquote `~(...)` to produce a `Value::wat__core__fn(Arc<Function>)`
3. **signature-of-fn** that fn-value → structured signature HolonAST (slice 3)
4. **extract-arg-names** sig → `Vector<keyword>` of param names (arc 143 slice 3)
5. **extract-arg-types** sig → `Vector<HolonAST>` of structured arg type-ASTs (arc 201 slice 5)
6. **For each k of N args:**
   - `name-k` = arg-names[k] (e.g. `:logger`)
   - `type-k` = arg-types[k] (e.g. `Bundle [Atom(:ThreadPeer), Atom(:S), Atom(:S)]`)
   - `Bundle/children type-k` → unpack the type AST; slot 1 = `I` type, slot 2 = `O` type (slot 0 = head `Atom(:ThreadPeer)`)
   - `factory-k` = variadic-args[k] (a call form AST like `(:app::logger-worker)`)
   - Construct fresh binding names from `name-k`: `thread-{name-k}`, `peer-{name-k}`, `drained-{name-k}` via the same string-concat-then-keyword-construct pattern D1 uses for `Receiver<I>` / `Sender<O>`
7. **Emit** the `let` form with:
   - Per-factory: `[thread-{name-k} (spawn-thread <wrap-fn-k>) peer-{name-k} (ThreadPeer/new ...)]`
   - `result (~coordinator peer-{name-1} peer-{name-2} ... peer-{name-n})` — coordinator fn applied to peers in binder order
   - Per-factory drain: `[_drained-{name-k} (Thread/drain-and-join thread-{name-k})]`
   - Final value: `result`

## Macro expansion (target) — N=2 worked example

For input:

```scheme
(:wat::kernel::run-threads
  (:wat::core::fn
    [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>
     b <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
    -> :wat::core::i64
    (:my::a-b-coordinator a b))
  (:my::factory-a)
  (:my::factory-b))
```

Expands to (approximately):

```scheme
(:wat::core::let
  [thread-a       (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- :rust::crossbeam_channel::Receiver<wat::core::String>
                       server-tx <- :rust::crossbeam_channel::Sender<wat::core::i64>]
                      -> :wat::core::nil
                      ((:my::factory-a) (:wat::kernel::ThreadPeer/new server-rx server-tx))))
   peer-a         (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread-a)
                    (:wat::kernel::Thread/input  thread-a))
   thread-b       (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- :rust::crossbeam_channel::Receiver<wat::core::i64>
                       server-tx <- :rust::crossbeam_channel::Sender<wat::core::String>]
                      -> :wat::core::nil
                      ((:my::factory-b) (:wat::kernel::ThreadPeer/new server-rx server-tx))))
   peer-b         (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread-b)
                    (:wat::kernel::Thread/input  thread-b))
   result         ((:wat::core::fn
                    [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>
                     b <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
                    -> :wat::core::i64
                    (:my::a-b-coordinator a b))
                   peer-a peer-b)
   _drained-a     (:wat::kernel::Thread/drain-and-join thread-a)
   _drained-b     (:wat::kernel::Thread/drain-and-join thread-b)]
  result)
```

Key shapes:
- Coordinator is spliced as a CALLABLE (`(~coordinator peer-1 peer-2 ...)`); it's a fn value that the let binding invokes with the peers in binder order
- Per-factory: 2 bindings (thread, peer) BEFORE coordinator call; 1 binding (drain) AFTER
- Fresh names from coordinator binder names (e.g. `thread-a`, `peer-a`, `_drained-a`)
- Server-side: each spawn-thread wraps `(<factory-form> (ThreadPeer/new server-rx server-tx))` — invokes the user's factory call form, which returns a fn that takes a peer

## Required path (NO new substrate primitives)

Arc 201 closed the reflection chain. Arc 200 closed the splice symmetry. Substrate path is proven; D2 is wat-side macro composition.

Likely shape:
- **Wat helper fn** in `wat/kernel/run_threads.wat` or `wat/kernel/run_threads_helpers.wat` (sonnet picks): takes coordinator fn-value + factories Vector → returns the let-binding clauses Vector. Macro body uses `~@(:wat::kernel::build-run-threads-bindings ...)` to splice.
- **Or inline quasiquote-recursion** if it composes cleanly.

Sonnet picks based on what reads + composes existing primitives. Per `feedback_simple_is_uniform_composition`: uniform repetition IS simple; don't abstract for the sake of abstracting.

## Tasks

### 1. Rewrite `wat/kernel/run_threads.wat` macro for coordinator-fn form

Replace D1's 4-arg positional macro with the new coordinator-fn form. Header comments fully rewritten to document:
- The coordinator-fn form
- The delegating-body convention (always single named-fn call)
- The macro algorithm (using arc 201 reflection chain)
- N=1 case still works through this same macro

### 2. Update D1's test (`tests/wat_run_threads_d1.rs`) to coordinator-fn form

Change the test program from D1's positional form to the coordinator-fn form. Coordinator body is a delegating call to a named echo fn. N=1 smoke test stays as the minimal case under the new shape.

### 3. Add D2 multi-factory test (`tests/wat_run_threads_d2.rs`)

Single test: `run_threads_d2_three_factories_heterogeneous`.

Three factories with heterogeneous types:
- Factory A: `ThreadPeer<String, i64>` — reads String "ping", writes i64 1
- Factory B: `ThreadPeer<i64, String>` — reads i64 N, writes String "got-N"
- Factory C: `ThreadPeer<String, String>` — reads String "hello", writes String "world"

Coordinator-fn binders: `[a <- :ThreadPeer<S,i64> b <- :ThreadPeer<i64,S> c <- :ThreadPeer<S,S>]`. Coordinator body: `(:my::three-fac-coordinator a b c)`. Named fn `:my::three-fac-coordinator` does the parent-side work:
1. Sends "ping" to peer-a; reads i64 reply
2. Sends 42 to peer-b; reads String reply
3. Sends "hello" to peer-c; reads String reply
4. Returns a Vector<String> assembled from the three replies

Test asserts the returned Vector matches the expected three-element shape.

### 4. Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_run_threads_d1
cargo test --release -p wat --test wat_run_threads_d2
cargo test --release --workspace --no-fail-fast
```

Both D1 and D2 tests must pass. Workspace baseline: 2328 passed / 3 stable failures (or 4 with lifeline flake variance).

### 5. STONES.md update

Edit `docs/arc/2026/05/170-program-entry-points/BRACKET-IMPLEMENTATION-STONES.md`:
- § D2 sub-stone: tick all items
- § Status line 205: replace `[ ] D2 — multi-factory heterogeneous expansion (unblocked — ...)` with `[x] D2 — multi-factory heterogeneous expansion shipped via coordinator-fn form (2026-05-16, <minutes> min, 2/2 tests green: D1 N=1 + D2 N=3 heterogeneous)`

### 6. SCORE

Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-D2-COORDINATOR.md` (distinct from the historical `SCORE-STONE-D2.md` per `feedback_inscription_immutable`). 5 rows YES/NO + evidence; honest deltas (fresh-name strategy, helper-fn vs inline approach, hygiene, reflection-chain composition quirks, baseline delta).

## STOP triggers (true emergencies — surface, do not paper over)

1. **Reflection chain has a gap at expand-time** — e.g., signature-of-fn doesn't return what arc 201 SCORE describes for an inline fn whose form was just constructed via the macro body. STOP, surface what you observed.
2. **Fresh-name construction collides with user names** — `thread-logger` shadows a user binding. Per `feedback_no_known_defect_left_unfixed`: surface + propose either gensym (probably overkill given user binders are scoped to their fn body, not the macro expansion) or document the collision risk + recommend users avoid `thread-{name}` / `peer-{name}` / `_drained-{name}` prefixes.
3. **Coordinator-fn is called at the wrong position** — e.g., as `(~coordinator ...)` doesn't work because coordinator splices as AST not as a callable. Surface the structural surprise; sonnet's wat insight is authoritative on macro mechanics.
4. **Workspace baseline regresses** — STOP, surface the new failure.
5. **Any urge to mint new substrate types, verbs, or special forms** — STOP. The reflection chain is complete; D2 is wat-side composition only.
6. **Coordinator body is NOT a single delegating call in the test fixtures** — STOP. The advertised pattern is delegating wrapper. If the test makes the body do real work inline, the macro's documentation is misleading. Tests must demonstrate the delegating-body pattern.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint any new substrate verb, type, struct, or special form (`feedback_no_new_types`).
- DO NOT modify Stone A's `Thread/drain-and-join`, Stone C1's `ThreadPeer` or its verbs, or any existing substrate Rust code.
- DO NOT extend the macro to panic cascade — that's D3.
- DO NOT touch arc 117/133 sibling-binding walker — Stone G's concern.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs (the SUPERSEDED prologue on `BRIEF-STONE-D2.md` is already in place by orchestrator — do not edit it).
- DO NOT modify INTERSTITIAL-REALIZATIONS.md (orchestrator owns).
- DO NOT modify the arc 199 DESIGN.md, BRIEF-STONE-D1.md, SCORE-STONE-D1.md (historical artifacts).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.
- DO NOT write new INSCRIPTION or USER-GUIDE content (Stone H handles).

**Macro dialect (Clojure-style):**
- `~` = unquote
- `~@` = unquote-splicing
- `,` = whitespace literal (commas are visual separator only, like Clojure)

## Decay disclosure (orchestrator's mental model is partial)

The reflection-chain-at-expand-time mechanics are inferred from arc 201 SCORE-SLICE-3.md + the design conversation. Concretely:
- Arc 201 slice 3 SCORE says signature-of-fn operates on `Value::wat__core__fn(Arc<Function>)` (the fn-value post-eval)
- Inline `(:wat::core::fn ...)` forms eval to Value::wat__core__fn before primitive dispatch sees them
- Computed-unquote `~(...)` in defmacro body evaluates the expression at expand time and converts via `value_to_watast`

Sonnet verifies these compose at expand time:
- Does the coordinator AST evaluate cleanly inside the macro body via computed-unquote?
- Does signature-of-fn called on the evaluated coordinator return the expected structured signature?
- Does extract-arg-names + extract-arg-types yield the expected Vectors?
- Does the resulting binding-clause Vector splice cleanly into the let form?

If any link in the chain fails to compose at expand time, surface as STOP-trigger 1.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::run-threads` macro rewritten for coordinator-fn form; D1's positional form retired | grep + macro signature inspection; D1 test passes with new form |
| B | D2 test added; 3 heterogeneous factories round-trip via the bracket | `cargo test --release -p wat --test wat_run_threads_d2` green |
| C | Coordinator body in all tests is a delegating call to a named fn (advertised pattern preserved) | test files visual + `grep "(:my::" tests/wat_run_threads_d2.rs` confirms |
| D | No new substrate types/verbs/structs minted | `git diff src/` shows zero substrate Rust additions |
| E | Workspace test failure count ≤ baseline (3 stable + lifeline flake variance) | full workspace cargo test failures ≤ baseline |

## Honest deltas to capture in SCORE

- Fresh-name strategy: name-suffix (`thread-logger`) vs gensym vs something else? What composition produced it at expand time?
- Helper-fn approach vs inline-recursive-quasiquote approach? Which read cleanly + composed without new substrate?
- Hygiene quirks: did any fresh-name collision surface? How was it resolved?
- Reflection-chain quirks: did signature-of-fn / extract-arg-types / Bundle/children compose cleanly at expand time, or did any step need a workaround?
- Macro-expansion machinery quirks: did variadic-iteration at expand time surface any substrate behaviors that needed careful handling?
- Workspace baseline preserved cleanly?

## Time-box

75-120 min predicted. The reflection-chain composition + name-construction + variadic splicing is non-trivial but all primitives exist. Hard stop 150 min.

## Workspace baseline (commit `8701317` lab / `bab6b8e` wat-rs)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2328 passed / 3-4 failed (3 stable: deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips; lifeline flake variance ±1)

Post-D2 target:
- ≥ baseline + 1 new pass (D2 test); D1 test remains passing under new shape (no net gain on D1 — same test now passes via coordinator-fn instead of positional)
- ≤ baseline failures (purely additive)

## On completion

1. Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-D2-COORDINATOR.md` per § SCORE methodology + § Honest deltas.
2. Tick D2 in STONES.md per § 5.
3. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + helper-vs-inline decision + any reflection-chain composition surprises.

You are launching now. T-minus 0.
