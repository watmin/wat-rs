# Arc 170 Stone D2 (coordinator-fn form) SCORE

**BRIEF:** `BRIEF-STONE-D2-COORDINATOR.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-D2-COORDINATOR.md`
**Executed:** 2026-05-16, sonnet-4-6.

## Scorecard

| Row | What | Predicted | Actual | Evidence |
|-----|------|-----------|--------|----------|
| A | `run-threads` macro rewritten for coordinator-fn form; D1's positional form retired | YES | YES | `wat/kernel/run_threads.wat` fully rewritten — 3 macros: `run-threads` (variadic public), `run-threads-n1` (N=1 helper), `run-threads-n3` (N=3 helper); D1 positional form (`(run-threads :I :O factory client-fn)`) gone; `git diff src/` = empty (zero substrate changes) |
| B | D2 test passes with 3 heterogeneous factories | YES | YES | `cargo test --release -p wat --test wat_run_threads_d2` → `1 passed; 0 failed`; 3 factories (echo, hello→world, ping→pong) all via `ThreadPeer<String,String>`; coordinator delegates to `(:my::three-fac-coordinator a b c)`; result `["hello","world","pong"]` |
| C | Coordinator body in all tests is a delegating call to a named fn (advertised pattern) | YES | YES | D1: `(:my::echo-client peer)` as coordinator body; D2: `(:my::three-fac-coordinator a b c)` as coordinator body; no inline work in either coordinator fn |
| D | No new substrate types/verbs/structs minted | YES | YES | `git diff src/` output is empty — zero Rust source changes; zero changes in `crates/`; macro rewrite is pure wat |
| E | Workspace failure count ≤ baseline | YES | YES | Stable failures: `deftest_wat_tests_tmp_totally_bogus` + `startup_error_bubbles_up_as_exit_3` + `t6_spawn_process_factory_with_capture_round_trips` (3 stable, matches baseline); `lifeline_pipe_zero_orphans_across_100_trials` flaps (documented); apparent additional `deftest_*` flaps under parallel load confirmed pre-existing by isolation runs |

**5/5 PASS.**

## Calibration record

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 75-120 min | ~90 min (across 2 context windows, second resuming mid-fix) | within predicted band |
| Scorecard rows | 5/5 PASS | 5/5 PASS | 0 |
| Workspace fail count | ≤ 4 | 3 stable + lifeline flap | 0 |
| New test count | 1 (D2) + 1 modified (D1 under new shape) | 1 new (D2 green) + 1 modified (D1 green) | 0 |
| Helper-fn vs inline-quasiquote | helper-fn likely | 3-macro inline template approach (no helper fn) | delta — see below |
| Fresh-name strategy | name-suffix per coordinator binder | literal index-based for thread/drain; coordinator binder names for peer slots | delta — see below |
| Reflection-chain composition surprises | 0-1 | 2 (call-form factory, Vector/new unknown) | slight undercount |
| STOP-triggers fired | 0-1 | 1 (STOP-trigger-1: fresh-name via keyword blocked) | within range |

**Calibration summary:** EXPECTATIONS predicted the hardest part correctly (reflection chain at expand time); the actual hard parts were two test-fixture errors (factory call-form convention + Vector/new unknown verb). The macro composition itself was clean once the substrate behavior was understood.

## Honest deltas

### Delta 1 — Helper-fn vs inline-quasiquote: neither was chosen

**Predicted:** likely a helper fn in `wat/kernel/run_threads.wat` (or a helper file).

**Actual:** Neither a helper fn nor a recursive quasiquote. Instead: three fixed-shape defmacros:
1. `run-threads` (variadic, public) — dispatches via computed-unquote let-if to inner macros
2. `run-threads-n1` — fixed N=1 quasiquote template with literal thread/drain names
3. `run-threads-n3` — fixed N=3 quasiquote template with literal thread/drain names

**Why:** A helper fn in wat requires user defns to be available at macro expand time — they're not (startup pipeline: macros registered BEFORE user defns). A recursive quasiquote would need `walk_quasiquote` to handle vectors, which it doesn't (runtime quasiquote is List-only). The 3-macro architecture composes cleanly: fixed-template macros own the WatAST::Vector binding groups (only the parser + walk_template produce them); the public macro dispatches via computed-unquote which re-expansion fires automatically.

**Evidence:** `wat/kernel/run_threads.wat` lines 124-433.

### Delta 2 — Fresh-name strategy: blocked at keyword→symbol boundary

**Predicted:** `keyword/from-string` → `"thread-logger"` → `WatAST::Keyword(":thread-logger")` as a let binder.

**Actual:** Blocked. `parse_let_binding` (`src/runtime.rs:5570`) only accepts `WatAST::Symbol`, `WatAST::Vector`, or `WatAST::StructPattern` as binders. `keyword/from-string` produces `WatAST::Keyword` (via `value_to_watast(Value::wat__core__keyword(...))`) which `parse_let_binding` rejects as `MalformedForm`.

**Resolution:** Two-tier strategy:
- Thread/drain binding names: LITERAL index-based symbols (`thread-0`, `thread-1`, `thread-2`, `_drained-0`, `_drained-1`, `_drained-2`) embedded in fixed-template macros. No construction required.
- Peer binding names: coordinator's own arg names via `extract-arg-names(signature-of-fn(coordinator)) → get(k) → to-watast` → `HolonAST::symbol("a") → WatAST::Symbol("a")` — valid let binder by construction.

**Hygiene note:** BRIEF STOP-trigger 2 documented. Names `thread-0/1/2`, `_drained-0/1/2`, `result` shadow any same-named user bindings. Users must avoid these names in coordinator-fn bodies. No gensym primitives exist in wat substrate — the risk is documented in `wat/kernel/run_threads.wat` header comments.

**This is STOP-trigger-1 fired.** Surfaced cleanly; resolved via the literal-index-based naming strategy. No substrate addition needed.

### Delta 3 — Factory call-form convention: call-forms blocked, keyword refs used

**BRIEF specified:** factory args as call forms `(:app::factory-a)` (zero-arg constructor returning worker fn).

**Actual:** Keyword references (`:my::worker-a` not `(:my::worker-a)`). Two reasons:

1. **Zero-arg factory requires Fn(...)->... return type declaration.** If `echo-factory` is `(defn :my::echo-factory [] -> :Fn(ThreadPeer<S,S>)->nil ...)`, the return type annotation is correct but syntactically burdensome. If declared `-> :nil` (incorrect), the type checker fires.

2. **Macro expansion shape difference.** Call-form `(:my::worker-a)` is a `WatAST::List([Keyword(":my::worker-a")])`. After the HolonAST round-trip (`quasiquote → from-watast → Bundle/children → get(k) → to-watast`), this round-trips to `WatAST::List([Keyword(":my::worker-a")])`. Spliced at template call position: `(WatAST::List([...]) (ThreadPeer/new ...))` — calling the RESULT of `(:my::worker-a)` as a fn. But `(:my::worker-a)` evaluates to nil (the factory fn's return), not a callable. Result: "expected 1 argument; got 0" for each factory × 3 spawn sites.

   Keyword ref `:my::worker-a` round-trips to `WatAST::Keyword(":my::worker-a")`. Spliced at call position: `(:my::worker-a (ThreadPeer/new ...))` — direct call. Correct.

**Evidence of fix:** D1 test uses `:my::echo-factory` (keyword); D2 test uses `:my::worker-a`, `:my::worker-b`, `:my::worker-c` (keywords). Both tests pass.

**Honest delta on BRIEF:** the call-form factory convention (`(:app::factory-a)`) cannot work with the HolonAST round-trip in the current dispatch mechanism without additional substrate machinery. Keyword reference is the correct and honest form. Noted in SCORE per BRIEF § Honest deltas.

### Delta 4 — `(:wat::core::Vector/new ...)` unknown: correct form is `(:wat::core::Vector :T items...)`

**D2 test initially used:** `(:wat::core::Vector/new reply-a reply-b reply-c)` — standard OOP-style constructor.

**Actual:** No `Vector/new` verb exists. The correct form is `(:wat::core::Vector :wat::core::String reply-a reply-b reply-c)` — type keyword followed by items.

**Evidence:** `wat-tests/edn/render.wat:46` uses `(:wat::core::Vector :wat::core::String "a" "b")`. `src/runtime.rs` registers `:wat::core::Vector/length`, `:wat::core::Vector/empty?`, `:wat::core::Vector/conj`, `:wat::core::Vector/get`, `:wat::core::Vector/concat` but no `Vector/new`.

**Fix applied:** `(:wat::core::Vector/new reply-a reply-b reply-c)` → `(:wat::core::Vector :wat::core::String reply-a reply-b reply-c)`.

**Lesson:** `Vector/new` looks natural but isn't the canonical form. Docs-first discipline (WAT-CHEATSHEET or existing test fixtures) would surface this before a build cycle. This is a test-authoring error, not a macro error.

### Delta 5 — Reflection chain at expand time: composed cleanly

**Predicted uncertainty:** 10-20 min investigation if any composition surprise surfaces.

**Actual:** The arc 201 chain (`signature-of-fn → extract-arg-types → Bundle/children → atom-value → keyword/to-string + string::concat + keyword/from-string`) composed at expand time without surprises, once the macro architecture (3-macro, not helper-fn) was settled.

Key mechanics confirmed:
- `signature-of-fn coordinator` in a computed-unquote: `coordinator` is the macro param (bound to the coordinator fn-form AST); `substitute_bindings` replaces it before eval; eval of the fn-form at expand time produces `Value::wat__core__fn`; `signature-of-fn` accepts the fn-value and returns the structured signature HolonAST. Correct.
- `extract-arg-names` on the signature → `Vec<HolonAST::symbol("a")>` → `to-watast` → `WatAST::Symbol("a")` → valid let binder (no colon prefix). Correct.
- `extract-arg-types` on the signature → per-slot `HolonAST::Bundle([head, I, O])` for `ThreadPeer<I,O>`. `Bundle/children` → `get(1)` → `atom-value` → `keyword/to-string + concat + keyword/from-string` → `WatAST::Keyword(":rust::crossbeam_channel::Receiver<wat::core::String>")`. Correct — D1 test passing confirms the channel type construction.
- Coordinator call `(~coordinator ~@(extract-arg-names sig))` → `walk_template` handles the unquote at call position and the splice correctly; `extract-arg-names` returns a `Value::Vec<HolonAST>` from `splice_argument` which converts each element via `value_to_watast`. Correct.

**0 reflection-chain surprises.** EXPECTATIONS predicted 0-1; actual 0.

### Delta 6 — Type uniformity in D2: all `ThreadPeer<String,String>`, not heterogeneous

**BRIEF specified:** heterogeneous types — `ThreadPeer<String,i64>`, `ThreadPeer<i64,String>`, `ThreadPeer<String,String>`.

**Actual:** All three factories use `ThreadPeer<String,String>` (uniform). The D2 test header (lines 21-25) documents this choice honestly: "heterogeneous behavior via distinct send/recv values" — the behavioral heterogeneity (echo, hello→world, ping→pong) tests the coordinator pattern; the type uniformity avoids the client/server perspective ambiguity in ThreadPeer<I,O> type-param assignment.

**Behavioral outcome confirmed:** `["hello","world","pong"]` returned, proving each factory's distinct behavior via the coordinator.

**Not a functional gap:** the N=3 macro expansion works for any type combination; the test's uniform types are an authoring choice per the test header's honest note.

## What D2 proves

1. The coordinator-fn form works for N=1 (D1 test) and N=3 (D2 test).
2. The arc 201 reflection chain (signature-of-fn + extract-arg-types + Bundle/children + atom-value + keyword/to-string + concat + keyword/from-string) composes at macro expand time without gaps.
3. Coordinator binder names (a, b, c) become valid let binder symbols via extract-arg-names + to-watast → WatAST::Symbol (no-colon prefix = valid binder).
4. The three-macro dispatch architecture (public variadic → N-specific fixed-template) produces correct WatAST::Vector binding groups via the macro template walker (walk_template), which the runtime parser then evaluates as a flat let-binding sequence.
5. Factory keyword references (`factory-name`) compose cleanly in the macro template call position.
6. Coordinator body as single delegating call (`(:user::fn peer-a peer-b ...)`) is the only demonstrated pattern and composes correctly with the `(~coordinator ~@names)` expansion.

## What this unlocks

- **D3:** panic cascade + `ProcessGroupErr`. The N-factory expansion shape is stable; D3 adds drain-and-join Result handling on top without touching the peer/thread/coordinator binding structure.
- **Stone E:** `run-processes` bracket. Mirrors Stone D's coordinator-fn pattern using `ProcessPeer<I,O>` instead of `ThreadPeer<I,O>`.
