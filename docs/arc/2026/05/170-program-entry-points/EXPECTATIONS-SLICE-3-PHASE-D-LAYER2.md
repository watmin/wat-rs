# Arc 170 slice 3 phase D — EXPECTATIONS (sonnet scorecard)

**One spawn.** Author Layer 2 `:wat::test::run-hermetic-with-io` macro + new RunResultIO struct + one canonical test (T18) + one failing-assertion test (T18b).

## Independent prediction

**Runtime band:** 90-180 min sonnet. More design surface than Layer 1 (parametric type-param mechanism, dual-direction I/O, new result struct), Phase C foundation reduces risk.

**Hard cap:** 360 min. If sonnet hits cap, kill via TaskStop and score Mode B-time-violation.

## Scorecard (7 rows; sonnet self-scores then orchestrator verifies)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::run-hermetic-with-io` macro in `wat/test.wat` | `grep -n "wat::test::run-hermetic-with-io\b" wat/test.wat` shows the defmacro |
| B | `:wat::test::run-hermetic-with-io-driver` helper in `wat/test.wat` | grep |
| C | `:wat::test::RunResultIO<O>` struct registered | grep + struct fields visible |
| D | T18 happy-path round-trip + T18b failing-assertion both pass | `cargo test --release --test wat_arc170_program_contracts t18` → 2 passed 0 failed |
| E | Workspace stays at 0 failed (count rises from 2182 to 2184) | `cargo test --release --workspace --no-fail-fast` total failed = 0 |
| F | `cargo check --release` green | clean |
| G | SCORE explains Decisions 1/2/3 outcomes + honest deltas | manual review |

**7 rows.** All must pass.

## Implementation approach

### Decisions to lock + document in SCORE

**Decision 1 — type-param mechanism for the macro.**

Recommend Option A: explicit type AST args first.
```
(:wat::test::run-hermetic-with-io <input-type-AST> <output-type-AST> inputs body)
```
Surface what compiles. If Option B (angle-bracket syntactic form) works, flag it as a finding.

**Decision 2 — RunResultIO shape.**

Recommend Option A: new struct
```
:wat::test::RunResultIO<O> {
  outputs :wat::core::Vector<O>,
  stderr  :wat::core::Vector<wat::core::String>,
  failure :wat::core::Option<wat::kernel::Failure>,
}
```

Register via wat-side `:struct` form OR `src/types.rs` if wat-side registration is awkward. Surface choice.

**Decision 3 — send/drain ordering.**

Sequential: send all → drain all → join. Surface any test scenario that needs interleaving (none expected for T18 echo pattern; surface gaps for future Layer 2 callers).

### Helper function signature

```
(:wat::test::run-hermetic-with-io-driver<I, O>
   (proc :wat::kernel::Process<I, O>)
   (inputs :wat::core::Vector<I>)
   -> :wat::test::RunResultIO<O>)
```

Implementation steps:
1. Iterate `inputs`, calling `(:wat::kernel::send Process/tx input)` for each
2. Drain outputs: loop on `(:wat::kernel::recv Process/rx)` until `Ok(None)` (clean disconnect)
3. Join via `Process/join-result`
4. Drain stderr via `drain-lines` (existing hermetic.wat helper)
5. Rebuild chain via `extract-panics`
6. Assemble RunResultIO

### Macro expansion

```
(:wat::test::run-hermetic-with-io <I-type> <O-type> inputs body)

expands to:

(:wat::test::run-hermetic-with-io-driver
  (:wat::kernel::spawn-process
    (:wat::core::fn
      [rx <- :wat::kernel::Receiver<I>
       tx <- :wat::kernel::Sender<O>]
      -> :wat::core::nil
      ~body))
  ~inputs)
```

The macro substitutes `I`/`O` into the fn-form's typed-channel signature. Type-substitution via quasiquote/unquote on the type AST args.

### Canonical tests

**T18 — happy-path echo-doubled:**
- Inputs: `Vector(21 :i64)`
- Body: recv n, send (n * 2), nil
- Outputs assertion: `[42]`
- Failure assertion: `None`

**T18b — failing assertion inside Layer 2 body:**
- Inputs: `Vector(2 :i64)`
- Body: recv n, assert-eq n 3, send n, nil
- Failure assertion: `Some(Failure)` with message containing "assert" / "AssertionFailed"
- Outputs may be empty (child panicked before send) — surface what actually happens

## What sonnet should produce

1. **Code changes:**
   - `wat/test.wat` — new macro + new helper appended (do NOT modify existing Layer 1 macros/helpers)
   - `src/types.rs` (if Option A struct registration is Rust-side) — `:wat::test::RunResultIO` struct
   - `tests/wat_arc170_program_contracts.rs` — T18 + T18b appended
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-D-LAYER2.md` mirroring Phase C SCORE structure:
   - Scorecard verification
   - Decisions 1/2/3 outcomes + rationale
   - Honest deltas (≥ 3 categories)
   - Files modified
   - What's next (phase E path: consumer sweep)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT modify `:wat::test::run-hermetic` macro or `run-hermetic-driver` helper (Layer 1)
- Do NOT touch `deftest` / `deftest-hermetic` macro definitions
- Do NOT retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast`
- Do NOT touch BareLegacy* walker code in src/check.rs
- Do NOT touch `Process<I,O>` struct field shape
- Do NOT sweep consumers (phase E)
- Do NOT use deferral language in SCORE — per FM 11
- If you hit a substrate gap that makes Decision 1 or 2 require architectural decisions beyond your scope, STOP and report — do not workaround

## Tools required

- Read / Edit / Bash (cargo, git)
- Possibly Write for SCORE doc + new test rows
- No Agent invocations (single-agent slice)

## Verification commands sonnet runs

```bash
# Baseline at start
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Layer 2 macro presence
grep -n "wat::test::run-hermetic-with-io\b" wat/test.wat

# Canonical tests
cargo test --release --test wat_arc170_program_contracts t18 2>&1 | tail -10

# Final workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2182 passed / 0 failed
- Post phase D: 2184 passed / 0 failed (T18 + T18b added; nothing else)

## Honest delta categories (anticipated)

1. **Decision 1 outcome** — which type-param shape compiled; what failed and why
2. **Decision 2 outcome** — RunResultIO registration mechanism (wat-side vs src/types.rs); choice rationale
3. **Send/drain ordering** — sequential worked / didn't; any deadlock surface
4. **T18b output behavior** — what does outputs Vec look like when child panics before send (empty? partial? indeterminate?)
5. **Anything unexpected** — surfaced during authorship
