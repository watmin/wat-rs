# Arc 170 slice 3 phase D — BRIEF (Layer 2 `run-hermetic-with-io`)

**Sonnet.** Author Layer 2 of the DESIGN-spec'd three-layer testing-lib API: `:wat::test::run-hermetic-with-io` — the 9% case macro for tests that need typed-channel I/O between parent and child. Builds on Phase C's Layer 1 foundation (`:wat::test::run-hermetic` + `:wat::test::run-hermetic-driver`).

## Context — what's already shipped

**Phase C (commit `87564c2`)** — Layer 1 (`run-hermetic`):
- Macro at `wat/test.wat:559`: takes only `body`; generates `(fn [_rx _tx] -> :nil body)`; spawns via `:wat::kernel::spawn-process`; returns `:wat::kernel::RunResult`.
- Helper at `wat/test.wat:515`: `run-hermetic-driver(proc :Process<nil,nil>) -> :RunResult` — joins, drains stdout/stderr, builds RunResult.
- T17 + T17b at `tests/wat_arc170_program_contracts.rs` verify Layer 1 end-to-end.

**Phase C′ (commit `2680403`)** — structured panic emit:
- `src/spawn_process.rs::emit_panics_to_stderr` mirrors `fork.rs` — AssertionPayload panics emit `#wat.kernel/ProcessPanics` EDN line on stderr; `extract-panics` rebuilds the cascade; Failure.message carries the structured assert-eq diagnostic.

**Phase D's job:** add Layer 2 alongside Layer 1. Same hermetic-by-default (spawn-process) but the worker fn has typed-channel parameters in scope of body.

## DESIGN-mandated shape (§ "Layer 2")

```
(:wat::test::run-hermetic-with-io<I,O> inputs body)
```

- `inputs :Vector<I>` — typed Values the parent sends to the child via rx
- `body` — has `rx :Receiver<I>` and `tx :Sender<O>` as bindings in scope
- Returns: result struct carrying outputs (`:Vector<O>`) + failure info

What disappears from Layer 2 (vs Layer 1 + raw spawn-process):
- The fn-form wrapper itself — macro generates `(:fn [rx tx] -> :nil body)` internally
- `scope :Option<String>` (leaked substrate plumbing; drops)
- `forms :Vector<WatAST>` (caller writes body; no AST construction)
- `IOReader` / `IOWriter` byte streams — typed channels only

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-C-LAYER1.md`** — Phase C context
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-C-LAYER1.md`** — Phase C decisions + honest deltas (some closed by C′)
3. **`docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-3-PHASE-D-LAYER2.md`** — your scorecard
4. **`wat/test.wat`** lines 460-570 — Phase C Layer 1 macro + driver (the pattern to extend)
5. **`docs/arc/2026/05/170-program-entry-points/DESIGN.md`** § "Layer 2" — design spec
6. **`tests/wat_arc170_program_contracts.rs`** T4-T6, T17, T17b — canonical pattern source (T4 specifically: typed-channel round-trip via spawn-process)
7. **`tests/wat_arc170_program_contracts.rs` helpers** lines 156-220 — `drive_typed_recv`, `typed_send`, Process struct field accessors — the typed-channel API
8. **`src/types.rs`** `:wat::kernel::Process` struct shape (6 fields: stdin/stdout/stderr legacy + ProgramHandle + tx + rx)

## Architectural decisions you make (lock these explicitly in SCORE)

### Decision 1 — Macro type-param shape

How do `I` and `O` types get into the macro expansion? wat's defmacro accepts AST args; type params aren't first-class macro params.

Options:
- **A.** Two explicit type AST args first: `(run-hermetic-with-io :wat::core::i64 :wat::core::i64 inputs body)`
- **B.** Angle-bracket syntactic form (if parser supports): `(run-hermetic-with-io<:i64,:i64> inputs body)` — unlikely to work; flag if it does
- **C.** Type inference from `inputs`'s element type — probably impossible at macro expansion time

**Recommendation:** Option A. Surface what compiles; document choice.

### Decision 2 — Result type shape

Layer 2 needs to surface `outputs :Vector<O>` along with failure info. RunResult (current) carries `stdout / stderr / failure` — no typed outputs.

Options:
- **A.** New struct `:wat::test::RunResultIO<O> { outputs :Vector<O>, stderr :Vector<String>, failure :Option<Failure> }` — Layer-2-specific; cleaner
- **B.** Extend RunResult with optional `outputs :Option<Vector<wat::holon::HolonAST>>` — keeps one result type but loses type info
- **C.** Tuple return `(outputs :Vector<O>, result :RunResult)` — less idiomatic; surfaces both cleanly

**Recommendation:** Option A. Locks the surface; matches DESIGN intent.

### Decision 3 — Send/drain ordering

Parent's flow: send all inputs → drain all outputs → join. For typical test scenarios (bounded inputs/outputs that fit in pipe buffer), sequential works. For interleaved flows, threaded drain. Phase D scope: **sequential**. Surface in honest deltas if a test scenario needs interleaving.

## Helper function signature (Path A pure-wat per Phase C precedent)

```
(:wat::test::run-hermetic-with-io-driver<I, O>
   (proc :wat::kernel::Process<I, O>)
   (inputs :wat::core::Vector<I>)
   -> :wat::test::RunResultIO<O>)
```

Implementation skeleton (sonnet flesh out):
1. Send each input via `:wat::kernel::send` on `Process/tx`
2. Drain outputs via `:wat::kernel::recv` on `Process/rx` until disconnect
3. Join via `:wat::kernel::Process/join-result`
4. Drain stderr via `drain-lines`; rebuild chain via `extract-panics`
5. Assemble `:wat::test::RunResultIO` with collected outputs + stderr + Option<Failure>

## Canonical test scenario

Mirror Phase C's T17 shape but for I/O. Suggested: T18 — round-trip i64.

```
(:wat::core::define (:my::test::echo-doubled -> :wat::test::RunResultIO<wat::core::i64>)
  (:wat::test::run-hermetic-with-io :wat::core::i64 :wat::core::i64
    (:wat::core::Vector 21 (:wat::core::Vector :wat::core::i64))   ;; or whatever the inputs literal looks like
    (:wat::core::let
      [n (:wat::core::Option/expect -> :wat::core::i64
           (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
             (:wat::kernel::recv rx) "recv failed")
           "stream closed")
       _ (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send tx (:wat::core::i64::*'2 n 2))
           "send failed")]
      :wat::core::nil)))
```

Test assertions:
- `outputs` is `[42]` (single i64)
- `failure` is `:None`

Plus a complementary T18b that exercises a failing assertion inside the I/O body — verifies the structured-failure path still works in Layer 2.

## Scope (what's IN)

- New macro `:wat::test::run-hermetic-with-io` in `wat/test.wat`
- New helper `:wat::test::run-hermetic-with-io-driver` in `wat/test.wat`
- New result struct `:wat::test::RunResultIO<O>` registered (likely in src/types.rs or via wat-side `:struct` form — sonnet picks)
- ONE canonical test (T18 happy path) + ONE failing-assertion test (T18b)
- `cargo check --release` green
- Workspace stays green
- SCORE doc

## Scope (what's OUT)

- Layer 1 (`run-hermetic`) — already shipped; do NOT modify
- `deftest` / `deftest-hermetic` — keep unchanged (phase E sweeps these)
- `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` — phase F retires
- BareLegacy* walker / retired-verb eval arms — slice 4
- Process<I,O> legacy 3-byte-pipe fields — slice 4

## Ship criteria (7 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::run-hermetic-with-io` macro in `wat/test.wat` | grep |
| B | `:wat::test::run-hermetic-with-io-driver` helper in `wat/test.wat` | grep |
| C | `:wat::test::RunResultIO<O>` struct registered | grep / cargo test |
| D | T18 (happy path round-trip) + T18b (failing assertion) pass | cargo test |
| E | Workspace stays at 0 failed (2182 → 2184) | full workspace cargo test |
| F | `cargo check --release` green | clean |
| G | SCORE explains Decision 1/2/3 outcomes + honest deltas | manual review |

**7 rows.** All must pass.

## Predicted runtime

**90-180 min sonnet.** Layer 2 has more design surface than Layer 1 (parametric types, dual-direction I/O, new result struct). Pattern-source from Phase C + T4 reduces risk.

**Hard cap:** 360 min.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT modify `:wat::test::run-hermetic` macro or `run-hermetic-driver` helper (Layer 1 stays as-is).
- DO NOT touch `deftest` / `deftest-hermetic` macros (phase E).
- DO NOT retire `run-sandboxed-*` (phase F).
- DO NOT touch BareLegacy* / spawn.rs / Process struct fields (slice 4).
- DO NOT use deferral language in SCORE — per FM 11.
- If a substrate gap surfaces that makes Path A impossible AND Path B requires substrate-architectural decisions beyond your scope, STOP and report.
- Workspace must stay at 0 failed at every cargo test run.

## Honest delta categories (anticipated)

1. **Decision 1 outcome** — which type-param shape compiled; what didn't work
2. **Decision 2 outcome** — RunResultIO vs RunResult-extension; registration mechanism
3. **Send/drain ordering** — sequential vs needed-to-thread; surfaced gaps
4. **Anything unexpected** — surfaced during authorship

## Cross-references

- BRIEF (Phase C): [`BRIEF-SLICE-3-PHASE-C-LAYER1.md`](./BRIEF-SLICE-3-PHASE-C-LAYER1.md)
- SCORE (Phase C): [`SCORE-SLICE-3-PHASE-C-LAYER1.md`](./SCORE-SLICE-3-PHASE-C-LAYER1.md)
- DESIGN slice 3 spec: [`DESIGN.md`](./DESIGN.md) § "Layer 2" (line 901+)
- Phase E path forward: consumer sweep (migrate deftest → run-hermetic / run-hermetic-with-io)
