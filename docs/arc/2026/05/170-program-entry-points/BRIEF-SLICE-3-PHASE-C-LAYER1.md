# Arc 170 slice 3 phase C — BRIEF (Layer 1 `run-hermetic` macro)

**Sonnet.** Author Layer 1 of the DESIGN-spec'd three-layer testing-lib API. This BRIEF scopes Layer 1 ONLY — `:wat::test::run-hermetic`. Layer 2 (`run-hermetic-with-io`) and consumer sweep land in phase D.

## Context — the foundation crack this closes

Slice 1f-* shipped the consumer-vantage test sweep (Phase A + B1 + B2). What did NOT ship: the *testing-lib three-layer rebuild* half of slice 3 (per DESIGN.md § 861). The stdlib `wat/kernel/sandbox.wat` + `wat/kernel/hermetic.wat` still call retired verbs `:wat::kernel::spawn-program-ast` / `:wat::kernel::fork-program-ast` as their actual primitive. The check walker exempted stdlib forms, so this dependency was invisible until the slice-4 destructive-reap dry-run revealed 278 stdlib-rooted failures.

This slice authors the new entry point. After phase C + D ship, the bandaid retires cleanly.

## DESIGN-mandated shape (§ "Tooling rebuild — testing-lib three-layer API")

**Layer 1 — `(:wat::test::run-hermetic body)`** — the 90% case macro.
- User writes `body` directly; macro generates the fn-form wrapper internally.
- No channels in the signature. No inputs. No scope. No fn ceremony.
- Returns `:wat::kernel::RunResult`.
- **Hermetic by default** — uses `:wat::kernel::spawn-process(fn)` (forked OS process) for full isolation.

What disappears from Layer 1 versus today's `run-sandboxed-ast`:
- The fn's channel parameters
- The input data parameter (Layer 2 has it; Layer 1 doesn't)
- The fn-form wrapper itself (user writes body; macro wraps)
- `scope :Option<String>` (today's leaked substrate plumbing; drops)
- `forms :Vector<WatAST>` (caller writes body; no AST construction)
- `stdin :Vector<String>` / stdout / stderr as `Vec<String>` (Layer 2 only)
- `IOReader` / `IOWriter` (byte-stream types; drop from every testing layer)

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/DESIGN.md`** § "Slice 3 — consumer sweep + tooling rebuild" (line 861+) — three-layer model
2. **`docs/arc/2026/05/170-program-entry-points/TIERS.md`** — tier semantics
3. **`wat/test.wat`** lines 275-345 — current `deftest` / `deftest-hermetic` macro definitions (the things being SUPERSEDED by Layer 1)
4. **`wat/kernel/sandbox.wat`** + **`wat/kernel/hermetic.wat`** — the run-sandboxed-* primitives Layer 1 replaces
5. **`tests/wat_arc170_program_contracts.rs`** T4-T6 — canonical post-arc-170 spawn-process(fn) usage; pattern source for the macro expansion
6. **`src/thread_io.rs`** § "Slice 1f-γ — runtime-services carrier + bridge protocol" (line 357+) — the slice 1f-* stdio services Layer 1 must compose with
7. **`src/test_runner.rs`** RunResult assembly — current shape Layer 1 must produce
8. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — discipline reminder

## Authorization shape

```
;; Macro form:
(:wat::test::run-hermetic
  (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4))

;; Expands to (semantically):
;;   1. wrap body in a static fn AST: (:wat::core::fn [_rx _tx] -> :nil ~body)
;;   2. call :wat::kernel::spawn-process with that fn
;;   3. drive stdio via the slice 1f-* services (read child's stdout/stderr lines)
;;   4. wait for child exit
;;   5. assemble RunResult { stdout :Vector<String>, stderr :Vector<String>, failure :Option<Failure> }
;;   6. return RunResult
```

Sonnet's job: design the expansion + helper composition that produces this surface.

## The architectural call to make (sonnet decides + surfaces)

Two implementation paths for the wat-side helper that bridges fn → RunResult:

**Path A — Pure-wat helper.** Layer 1 macro expands to compose `:wat::kernel::spawn-process` + drain via Process struct fields + RunResult assembly all in wat. No new substrate verb.
- Pros: no substrate change; aligns with arc 170 "fn IS the program" mantra
- Cons: more complex wat code (typed-channel drain, line-by-line stderr/stdout extraction)

**Path B — New substrate helper verb** (e.g., `:wat::kernel::run-fn-and-extract-result`). Macro expands to call this verb with the fn. Substrate does the composition.
- Pros: simpler wat-side; substrate's existing test_runner.rs RunResult assembly is reusable
- Cons: new substrate surface; arguably another bandaid in disguise

**Sonnet decides** based on what compiles + tests cleanly. STOP and surface the choice + rationale in the SCORE.

If Path A fundamentally doesn't work (e.g., wat-side can't drain Process struct stdio fields cleanly into Vec<String>), STOP and report — that's a substrate gap that requires Path B.

## Scope (what's IN)

- New macro `:wat::test::run-hermetic` in `wat/test.wat`
- Helper function/verb (whichever path is chosen) — pure-wat OR Rust substrate
- ONE canonical test that uses the new macro and passes
- `cargo check --release` green
- All existing tests stay green (NO consumer sweep yet — `deftest` / `deftest-hermetic` keep working unchanged)
- SCORE doc

## Scope (what's OUT)

- Layer 2 (`run-hermetic-with-io`) — phase D
- Layer 3 (`spawn-process` Rust impl) — already shipped; nothing to do
- Consumer sweep of existing `deftest` / `deftest-hermetic` callers — phase E
- Retiring `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` — phase F (only after all consumers migrate)
- Retiring `spawn-program-ast` / `fork-program-ast` substrate eval arms — slice 4 (after all stdlib calls retire)
- BareLegacy* walker retirement — slice 4
- Process<I,O> legacy 3-byte-pipe fields retirement — slice 4

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::run-hermetic` macro defined in `wat/test.wat` | grep |
| B | Helper function/verb defined (path A or B) | grep |
| C | One canonical test using the new macro passes | `cargo test --release --test <test_file>` |
| D | Workspace failure count UNCHANGED (still 0 from 2180 baseline) | full workspace cargo test |
| E | `cargo check --release` green | clean |
| F | SCORE doc explains the implementation path chosen + why + any honest deltas | manual review |
| G | NO consumer sweep (deftest unchanged) — only Layer 1 added, parallel to existing entry points | grep |

**7 rows.**

## Predicted runtime

**60-120 min sonnet.** Substantive design work (architectural call between path A/B + canonical test authorship). Test surface authoring is bounded — one test demonstrates the pattern works.

**Hard cap:** 240 min. If sonnet hits the cap without Layer 1 working, kill via TaskStop and re-evaluate.

## Honest delta categories (anticipated)

1. **Path A vs Path B trade-off** — surface which won, why, and what the runner-up's blocker was
2. **RunResult assembly mechanism** — how stdout/stderr lines get from the child's pipe to wat-level `:Vector<String>`
3. **slice 1f-* service integration** — confirm the services route correctly; surface any gaps
4. **Anything else surfaced during authorship**

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT touch `deftest` / `deftest-hermetic` macro definitions (those stay; Layer 1 is added alongside).
- DO NOT retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` (phase F).
- DO NOT touch BareLegacy* walker code (slice 4).
- If you hit a substrate gap that makes Path A impossible, STOP and report — do not workaround.
- Workspace must stay green at every cargo test run.

## Cross-references

- DESIGN: [`DESIGN.md`](./DESIGN.md) § "Slice 3" (line 861+)
- TIERS: [`TIERS.md`](./TIERS.md)
- Slice 1f-ι (EDN-only stdio contract): [`SCORE-SLICE-1F-IOTA.md`](./SCORE-SLICE-1F-IOTA.md)
- Slice 1f-λ (consumer sweep): [`SCORE-SLICE-1F-LAMBDA-PHASE-B1.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-B1.md), [`SCORE-SLICE-1F-LAMBDA-PHASE-B2.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-B2.md)
- Stdlib current state: `wat/kernel/sandbox.wat`, `wat/kernel/hermetic.wat`
- Slice 4 inventory: [`DESIGN.md`](./DESIGN.md) § "Slice 4 — substrate retirement" (line 968+)
