# Arc 170 Stone D2 BRIEF — multi-factory heterogeneous `run-threads`

**Phase:** Second sub-stone of decomposed Stone D. See STONES.md § Stone D + INTERSTITIAL § 2026-05-16 (Stone D design pass).

**Predecessors:**
- Stone A SHIPPED — `Thread/drain-and-join`
- Stone C1 SHIPPED — `ThreadPeer<I, O>` + verbs
- D1 SHIPPED + REFACTORED — minimal `run-threads`, single-factory, clean call form via arc 143 slice 2's computed-unquote (commit `257e8ca`)
- Arc 199 REJECTED 2026-05-16 — substrate already sufficient via computed-unquote pattern

**Successor:** D3 — panic cascade + `ProcessGroupErr`.

## Goal

Extend `:wat::kernel::run-threads` to support N factories with heterogeneous types. The call form unifies — D1's 4-arg shape RETIRES in favor of a 2-arg shape that covers N=1..N uniformly. Per the four-questions outcome: nested vectors `[[:I :O factory] ...]` won YES YES YES YES on grouping + visual separation; unified macro won YES YES YES YES on one-canonical-path.

## Required call form (post-D2)

```scheme
(:wat::kernel::run-threads
  [[:I₁ :O₁ factory₁]
   [:I₂ :O₂ factory₂]
   ...
   [:Iₙ :Oₙ factoryₙ]]
  client-fn)
```

The N=1 case (D1's existing test):

```scheme
(:wat::kernel::run-threads
  [[:wat::core::String :wat::core::String :my::echo-factory]]
  :my::echo-client)
```

Client-fn signature: `:Fn(ThreadPeer<O₁,I₁>, ThreadPeer<O₂,I₂>, ... ThreadPeer<Oₙ,Iₙ>) -> R` — variadic positional peers (one per factory), in the same order as the factory specs.

## Required macro expansion (target)

For N=2 caller:
```scheme
(:wat::kernel::run-threads
  [[:I₁ :O₁ f₁]
   [:I₂ :O₂ f₂]]
  client-fn)
```

Expands to (approximately):
```scheme
(:wat::core::let
  [thread-0       (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- :rust::crossbeam_channel::Receiver<I₁>
                       server-tx <- :rust::crossbeam_channel::Sender<O₁>]
                      -> :wat::core::nil
                      (f₁ (:wat::kernel::ThreadPeer/new server-rx server-tx))))
   client-peer-0  (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread-0)
                    (:wat::kernel::Thread/input  thread-0))
   thread-1       (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- :rust::crossbeam_channel::Receiver<I₂>
                       server-tx <- :rust::crossbeam_channel::Sender<O₂>]
                      -> :wat::core::nil
                      (f₂ (:wat::kernel::ThreadPeer/new server-rx server-tx))))
   client-peer-1  (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread-1)
                    (:wat::kernel::Thread/input  thread-1))
   result         (client-fn client-peer-0 client-peer-1)
   _drained-0     (:wat::kernel::Thread/drain-and-join thread-0)
   _drained-1     (:wat::kernel::Thread/drain-and-join thread-1)]
  result)
```

Key shapes:
- Per-factory: 2 bindings (thread, client-peer) before client-fn call; 1 binding (drain) after
- Drain-and-join happens AFTER client-fn returns (client-fn must release its peer handles by returning)
- Fresh names: `thread-0`, `thread-1`, ..., `client-peer-0`, `client-peer-1`, ..., `_drained-0`, `_drained-1`, ... (sonnet picks the naming convention — index-suffix or gensym; document the choice in SCORE)

## Required path (NO new substrate primitives)

Per arc 199 rejection: substrate already has everything via:
- Arc 143 slice 2's computed unquote at expand time (`~(...)` calls arbitrary substrate primitives, converts result via `value_to_watast`)
- `:wat::core::keyword/from-string` + `:wat::core::keyword/to-string` + `:wat::core::string::concat`
- `value_to_watast` lifts `Value::wat__core__keyword` → `WatAST::Keyword` and `Value::holon__HolonAST` → corresponding WatAST shape (arc 143 used HolonAST::symbol for bare-name symbol references)

D2's iteration over the factory-specs Vector at expand time requires either:
1. **Wat helper fn** — write a `:wat::kernel::build-run-threads-bindings` (or similarly-named) wat-side function that takes the factory-specs Vec, returns a Vec of binding-clause ASTs; macro body uses `~@(:wat::kernel::build-run-threads-bindings factory-specs ...)` to splice
2. **Direct quasiquote-recursion** — sonnet decides if this works for variadic iteration

Sonnet picks based on what reads cleanly + composes existing primitives. STOP if either path requires new substrate verbs.

## Tasks

### 1. Refactor D1's macro to the unified shape

`wat/kernel/run_threads.wat`: change the macro signature from D1's 4-arg form to D2's 2-arg `[[specs...] client-fn]` form. Header comments updated to reflect the unified shape covers N=1..N.

### 2. Update D1's test to the unified call form

`tests/wat_run_threads_d1.rs`: the test program changes from
```scheme
(:wat::kernel::run-threads :wat::core::String :wat::core::String :my::echo-factory :my::echo-client)
```
to
```scheme
(:wat::kernel::run-threads
  [[:wat::core::String :wat::core::String :my::echo-factory]]
  :my::echo-client)
```
D1's test stays as the N=1 smoke; D2 adds the multi-factory test in a separate file.

### 3. Add D2 test in `tests/wat_run_threads_d2.rs`

Single test: `run_threads_d2_three_factories_heterogeneous`.

Three factories with heterogeneous types:
- Factory A: `ThreadPeer<String, i64>` — reads String "ping", writes i64 1
- Factory B: `ThreadPeer<i64, String>` — reads i64 N, writes String "got-N"
- Factory C: `ThreadPeer<String, String>` — reads String "hello", writes String "world"

Client-fn signature: `:Fn(ThreadPeer<i64, String>, ThreadPeer<String, i64>, ThreadPeer<String, String>) -> :Vector<String>` — receives three peers in factory order.

Client-fn body:
1. Sends "ping" to peer-A via `Thread/println`; reads i64 reply via `Thread/readln`
2. Sends 42 to peer-B via `Thread/println`; reads String reply via `Thread/readln`
3. Sends "hello" to peer-C via `Thread/println`; reads String reply via `Thread/readln`
4. Returns Vec [i64-as-string-A, reply-B, reply-C]

Test asserts the returned Vector matches `["1" "got-42" "world"]` (or whatever the renderings come out to).

The test naming + assertions should clearly show heterogeneous types pass through the macro correctly.

### 4. Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_run_threads_d1
cargo test --release -p wat --test wat_run_threads_d2
cargo test --release --workspace --no-fail-fast
```

Both D1 and D2 tests must pass (D1 verifies the refactored macro still works at N=1; D2 verifies N=3 heterogeneous). Workspace baseline: failures ≤ 4 (3 stable + lifeline flake variance).

### 5. STONES.md update — tick D2 `[x]`

Edit `docs/arc/2026/05/170-program-entry-points/BRACKET-IMPLEMENTATION-STONES.md`:
- § D2 sub-stone: tick all items
- § Status: replace `[ ] D2` with `[x] D2 — multi-factory heterogeneous expansion (2026-05-16, <minutes> min, 1/1 test green; D1 refactored to unified shape)`

### 6. SCORE

Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-D2.md`. 5 rows YES/NO + evidence; honest deltas (fresh-name strategy, helper-fn vs inline approach, any hygiene quirks, baseline delta).

## STOP triggers (true emergencies — surface, do not paper over)

1. **Variadic iteration at expand time requires a new substrate primitive** — STOP, surface. The investigation that rejected arc 199 found everything needed already exists; if D2 hits a real gap, surface with evidence (grep + specific eval failure).
2. **Hygiene clash on fresh names** — STOP if `thread-0` / `client-peer-0` collide with user-bound names; surface and we vet either gensym or different naming.
3. **Workspace baseline regresses** — STOP, surface the new failure.
4. **Any urge to mint new substrate types, verbs, or special forms** — STOP. Arc 199 rejection lesson applied.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint any new substrate verb, type, struct, or special form (`feedback_no_new_types`).
- DO NOT modify Stone A's `Thread/drain-and-join`, Stone C1's `ThreadPeer` or its verbs, or any existing substrate Rust code.
- DO NOT extend the macro to panic cascade — that's D3.
- DO NOT touch arc 117/133 sibling-binding walker — Stone G's concern.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs other than § 5 STONES.md tick.
- DO NOT modify INTERSTITIAL-REALIZATIONS.md (orchestrator owns).
- DO NOT modify the arc 199 DESIGN.md or BRIEF-STONE-D1.md or SCORE-STONE-D1.md (historical artifacts).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.
- DO NOT write new INSCRIPTION or USER-GUIDE content (Stone H handles).

**Macro dialect (Clojure-style; confirmed in arc 199 rejection):**
- `~` = unquote
- `~@` = unquote-splicing
- `,` = whitespace literal (commas are visual separator only)

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::run-threads` macro refactored to unified 2-arg shape `[[specs...] client-fn]` | grep + macro signature inspection |
| B | D1's test updated to new call form; still passes | `cargo test --release -p wat --test wat_run_threads_d1` green |
| C | D2 test added; 3 heterogeneous factories round-trip via the bracket | `cargo test --release -p wat --test wat_run_threads_d2` green |
| D | No new substrate types/verbs/structs minted | grep verifies no register_builtin / register_eval additions |
| E | Workspace test failure count ≤ baseline (4) | full workspace cargo test failures ≤ baseline + flake variance |

## Honest deltas to capture in SCORE

- Fresh-name strategy: index-suffix (`thread-0`, `thread-1`) vs gensym vs something else? What composition produced it at expand time?
- Helper-fn approach vs inline-recursive-quasiquote approach? Which read cleanly + composed without new substrate?
- Hygiene quirks: did any fresh-name collision surface? How was it resolved?
- Macro-expansion machinery quirks: did variadic-iteration at expand time surface any substrate behaviors that needed careful handling?
- Workspace baseline preserved cleanly?

## Time-box

45-75 min predicted (more than D1 due to variadic iteration + fresh names). Hard stop 90 min.

## Workspace baseline (commit `257e8ca`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures (lifeline flake + 3 pre-existing)

Post-D2 target:
- ≥ baseline + 1 new pass (D2 test); D1 test remains passing under refactored shape
- ≤ baseline failures (purely additive)

## On completion

1. Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-D2.md` per § SCORE methodology + § Honest deltas.
2. Tick D2 in STONES.md per § 5.
3. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + any surprises observed.

You are launching now. T-minus 0.
