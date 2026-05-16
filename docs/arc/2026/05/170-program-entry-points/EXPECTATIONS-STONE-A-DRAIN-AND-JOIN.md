# Arc 170 Stone A EXPECTATIONS

**BRIEF:** `BRIEF-STONE-A-DRAIN-AND-JOIN.md`

## Independent prediction

**Runtime band:** 90-120 minutes sonnet.

Reasoning:
- Additive substrate change (no existing-fn modification)
- Two new fns + two type-signature registrations + 4 minimal tests
- Existing drain machinery + existing join machinery = drain-and-join is composition
- The "drain to disconnect" loop on Thread's typed channel + "drain-lines to EOF" on Process's stdout/stderr is similar to patterns in `-with-io-driver` at `wat/test.wat:907-961` (now in substrate)

**Time-box:** 180 min hard stop.

## SCORE methodology

6 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (Thread primitive): `grep -nA 20 "eval_kernel_thread_drain_and_join" src/runtime.rs` shows new fn; dispatch in primitive name-match.
- **Row B** (Process primitive): same for `eval_kernel_process_drain_and_join`.
- **Row C** (Type registrations): grep `Thread/drain-and-join\|Process/drain-and-join` in `src/types.rs` / `src/stdlib.rs` shows signatures registered with correct Result return shape.
- **Row D** (Tests pass): targeted `cargo test` on 4 new test names all green.
- **Row E** (Build clean): cargo build clean.
- **Row F** (Workspace baseline maintained): cargo test summed failed ≤ current baseline; ideally equal (since change is purely additive).

## Honest deltas to watch for

- **Existing drain machinery may not extract cleanly.** `drain-lines` is wat-side; substrate-side equivalent may need a small refactor. If existing internal helpers don't compose for Thread's typed channel drain (recv-until-disconnect pattern), sonnet may need to write a new internal helper.

- **Type-signature registration pattern.** Thread<I,O> and Process<I,O> as input parameters — verify the existing primitive registration pattern handles parameterized types correctly. The existing `Thread/join-result` / `Process/join-result` registrations are the template.

- **Result<nil, *DiedError> return shape.** Confirm the wat-side type alias for `nil` (per arc 169) is what's used; not `:()` (retired).

- **Test fixtures.** Existing tests in `tests/wat_arc170_program_contracts.rs` exercise Thread<i64,i64> / Process<i64,i64> shapes. New tests can mirror those — both happy-path and panic-path.

- **Workspace test count.** Baseline TBD at start; ideally additive (4 new pass, 0 break). If existing tests break, surface immediately.

- **Shared internal helper extraction.** If `do_join_with_optional_drain(thr, drain: bool)` reads cleaner than two near-duplicate fns, that's acceptable refactor IN scope. Otherwise keep them as two separate fns.

## Workspace baseline (commit `5efbc79`)

- `cargo build --release --workspace --tests`: presumed clean (last known-good at b2d6897; b678a92 + 5efbc79 are docs-only additions)
- `cargo test --release --workspace --no-fail-fast`: baseline failure count TBD at sonnet start

Post-Stone-A target:
- ≥ baseline + 4 passed (4 new tests add to the count)
- ≤ baseline failed (no existing tests broken)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90-120 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 4+ | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Additive | TBD |
