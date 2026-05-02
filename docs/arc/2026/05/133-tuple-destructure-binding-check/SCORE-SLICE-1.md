# Arc 133 Slice 1 — Score against pre-handoff expectations

**Written:** 2026-05-02, AFTER reading sonnet's report.

**Agent ID:** `a293ba381e3b1a32f`
**Agent runtime:** 1485 seconds (~25 min)

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single Rust file diff | **PASS** | `git diff --stat` shows only `src/check.rs` modified (518+/42- = +476 LOC). No `.wat` files. No documentation. |
| 2 | Approach picked + justified | **PASS+** | Sonnet picked the in-place inference-time check inside `infer_let_star`; justified the rejection of the walker-with-CheckEnv path (would require plumbing CheckEnv + redundant partial inference). Surfaced `unify`-via-`reduce` alias-expansion subtlety; named the trade-off clearly. |
| 3 | Tuple-destructure shapes recognized | **PASS** | `check_let_star_for_scope_deadlock_inferred` reads from `extended` map directly; `extend_pair_scope_with_tuple_destructure` extends the pair-deadlock walker. Both paths cover both binding shapes. |
| 4 | Required unit tests added | **PASS** | All four arc_133_* tests present at lines 11566 / 11626 / 11691 / 11746. Names match BRIEF verbatim. |
| 5 | **Unit tests pass** | **PASS** | `cargo test --release -p wat --lib check` exit=0; **53/53 pass**. Arc 117 + arc 128 + arc 131 + arc 126 unit tests all still pass. |
| 6 | Existing checks intact | **PASS** | `validate_scope_deadlock` + `walk_for_deadlock` + `check_let_star_for_scope_deadlock` + `parse_binding_for_typed_check` are all `#[allow(dead_code)]` with retirement notes; `validate_channel_pair_deadlock` (arc 126) extended (not retired). The structural-walker call sites at `check_program` retired with explanatory comment. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications to `src/check.rs`; agent did not commit. |
| 8 | Honest report | **PASS+** | Sonnet surfaced THREE honest deltas: (1) `unify`-calls-`reduce` alias expansion required adding `rust::crossbeam_channel::Sender` to surface match; (2) two synthetic counter variables for pair-anchor IDs; (3) `find_binding_span` helper not in BRIEF. Workspace prediction was 0 newly-firing tests in `wat-tests/` — accurate for that scope, but missed `tests/wat_*.rs` (3 sites). |

**HARD VERDICT: 8 OF 8 PASS. Clean ship for the substrate work.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 9 | LOC budget | **PASS (high band)** | +518 / -42 = +476 LOC. Within "larger budget OK if walker retires" framing — sonnet retired four pre-inference functions as `#[allow(dead_code)]` (kept as reference; net deletion possible in arc 133 slice 3 closure). The new in-place check + helpers + four unit tests sum to ~470 LOC. |
| 10 | Diagnostic span quality | **PASS** | `find_binding_span` extracts the offending binding's source span; ScopeDeadlock reads "scope-deadlock at <span>: ... offending_binding: '<name>' (a HandlePool|Sender) ..." — same shape as arc 117/131. |
| 11 | Workspace runtime | **n/a (workspace did fail)** | Workspace test `cargo test --release --workspace` runs in ~14s (build) + ~30s (test) = ~45s. Under 90s budget. |
| 12 | No prediction explosion | **PARTIAL — but slight miss** | Prediction was 0 newly-firing tests; actual is 3 sites in `tests/wat_spawn_lambda.rs`. The grep coverage missed `tests/wat_*.rs` (Rust integration tests with embedded wat strings). Real surface ≤5 sites; well within the "≤10 = OK" band; just not the predicted zero. |

**SOFT VERDICT: 3 OF 4 PASS, 1 partial.** The diagnostic-span work was clean; LOC was within band; runtime was fine; prediction was off by 3 sites in a single Rust integration test file.

## Independent prediction calibration

The orchestrator predicted (in `EXPECTATIONS-SLICE-1.md`):

- 50% in-place / 25% walker-extension / 15% hybrid /
  7% scope explosion / 3% substrate surprise.

**Actual outcome:** in-place path was chosen (matches the
50% bucket). Sonnet retired the structural walker
explicitly (as `#[allow(dead_code)]` rather than deleting),
matching the "retire cleanly" framing. The substrate
surprise (`unify`-calls-`reduce` alias expansion via
`rust::crossbeam_channel::Sender`) was a 3% tail event but
sonnet diagnosed and fixed it inline; it didn't escalate
to a separate arc.

**Sweep timing: 25 min.** Longer than recent sweeps (4-19
min range). The substrate investigation + path selection
+ alias-expansion fix + four unit tests + structural-walker
retirement together filled the time. Honest scope; no
padding.

## Workspace failures — slice 2 scope

Three tests in `tests/wat_spawn_lambda.rs` newly fire the
inferred-types ScopeDeadlock check:

- `spawn_thread_named_define_body`
- `spawn_thread_inline_lambda_body`
- `spawn_thread_closure_capture`

Failure shape (representative):

```
Check(CheckErrors([ScopeDeadlock {
  thread_binding: "thr",
  offending_binding: "tx",
  offending_kind: "Sender",
  span: Span { file: "<entry>", line: 26, col: 14 } }]))
```

The pattern in source (canonical Thread<I,O> usage):

```scheme
(:wat::core::let*
  (((thr :Thread<i64,i64>) (spawn-thread :app::increment))
   ((tx :Sender<i64>) (Thread/input thr))   ; ← fires
   ((rx :Receiver<i64>) (Thread/output thr))
   ((_ack :unit) (send tx 41 ...))
   ((result :i64) (recv rx ...)))
  (Thread/join-result thr))
```

The rule fires correctly: `tx` is a Sender; sibling to
`thr`; body has `Thread/join-result thr`. Same canonical
fix as arc 117/131: nest `tx + rx + send + recv` in an
inner `let*`; outer holds only `thr`; outer body's only
operation is `Thread/join-result thr`.

**Why these were correct-by-accident pre-arc-133:** the
old structural walker classified types via parsed source
annotations. The user wrote `:rust::crossbeam_channel::Sender<...>`
literally, but the old `type_contains_sender_kind` matched
only `wat::kernel::Sender` / `wat::kernel::Channel` heads
— so the rust:: form bypassed the check at the source-text
layer. Arc 133's in-place check uses post-`reduce` inferred
types, which canonicalize to `rust::crossbeam_channel::Sender`,
and sonnet's match extension catches them.

This is the SAME class of fix as arc 131 slice 2: legitimate
deadlock-prone shapes that prior bypasses hid become visible
under the structural rule and need inner-let* refactoring.

## Next steps

1. **Commit arc 133 slice 1** — substrate work + unit tests
   ship clean.
2. **Slice 2** — refactor the 3 `tests/wat_spawn_lambda.rs`
   sites to inner-let* shape. Small enough to fold inline OR
   spawn one quick sonnet sweep. Decision pending user
   direction.
3. **Slice 3** — closure: INSCRIPTION + WAT-CHEATSHEET §10
   note that the rule fires uniformly across binding shapes
   and surface-vs-canonical-alias-expansion. Cross-references
   to arcs 117 + 131.

## Failure-engineering record

Arc 133 slice 1 closes the binding-shape bypass surfaced by
arc 131 slice 2. The chain continues:

| # | Arc | Surfaced by | Status |
|---|---|---|---|
| 1 | 117 | substrate-author | shipped |
| 2 | 126 | arc 124 sweep | shipped |
| 3 | 128 | arc 126 sweep 1 | shipped |
| 4 | 129 | arc 126 sweep 3 | shipped |
| 5 | 131 | arc 130 sweep killed | shipped |
| 6 | 132 | user direction | shipped |
| 7 | **133 slice 1** | **arc 131 slice 2 SCORE** | **shipped (this score)** |
| 8 | 133 slice 2 | sonnet's prediction miss | TBD |

Each substrate-fix arc closes a gap the previous arc surfaced.
The artifacts-as-teaching record continues to validate the
discipline.
