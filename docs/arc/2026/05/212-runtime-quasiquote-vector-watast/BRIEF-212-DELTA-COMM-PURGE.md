# Arc 212 stone δ-comm-purge — Purge protocol violations the substrate-as-teacher cascade revealed

**Your ONE concern this spawn:** wrap each `_ <comm-call>` site in `Result/expect` so the comm Result is honestly handled. Four sites in two files. Verify both currently-failing tests now pass. Nothing else.

This is the substrate-as-teacher cascade closure for δ-comm-positions. δ-comm-positions sharpened `validate_comm_positions` to walk into let-binding-Vector RHSes; that revealed a class of protocol violations — `_`-discard of a comm Result — that were hidden behind the pre-arc-212 List-only walker. The substrate has been correctly enforcing arc 110 all along; the test fixtures were the violators.

---

## The discovery

User framing 2026-05-18: *"this is in blatant violation of the mini-tcp"* + *"non-compliance is not tolerable"*.

A `:wat::kernel::send` (or recv / process-send / process-recv / Process/readln / Process/println) call returns `Result<_, *Error>`. The Err arm signals **cross-world disconnect** — the receiver has dropped, the producer has gone away. That Result is the protocol-compliance measurement.

Binding to `_` discards the measurement. The fixture pretends "this comm call succeeded" without checking. That's the divide-by-zero — it has the SHAPE of cross-world work being done but no proof any work happened.

`_` for intra-world value discard is honest. `_` for comm Result is NOT — the comm Result is the protocol-compliance signal, not an ordinary value.

---

## The four sites

```
tests/wat_arc170_stone_a_drain_and_join.rs:101    [_ (:wat::kernel::send tx 1)
tests/wat_arc170_stone_a_drain_and_join.rs:102     _ (:wat::kernel::send tx 2)
tests/wat_arc170_stone_a_drain_and_join.rs:103     _ (:wat::kernel::send tx 3)]
tests/probe_lifeline_orphan_clean_via_fork_program.rs:209          _ (:wat::kernel::recv rx)] :wat::core::nil))"
```

---

## The migration

Replace each `_ (<comm-call>)` with `_ (:wat::core::Result/expect -> :T (<comm-call>) "msg")`.

The `:T` annotation depends on the call:
- `:wat::kernel::send` returns `Result<:wat::core::nil, :wat::kernel::SendError>` → `_ (:wat::core::Result/expect -> :wat::core::nil (:wat::kernel::send tx N) "send failed")`
- `:wat::kernel::recv` returns `Result<:wat::core::Option<I>, :wat::kernel::RecvError>` — for the lifeline probe's discard case, the appropriate shape is per the recv's I type (look at the surrounding context to determine)

**Read the surrounding code in each file to determine the exact T type before substituting.** The Result/expect form REQUIRES the -> :T annotation per arc 108.

**Worked example for the arc170 stone A case:**

Before:
```scheme
(:wat::core::let
  [_ (:wat::kernel::send tx 1)
   _ (:wat::kernel::send tx 2)
   _ (:wat::kernel::send tx 3)]
  :wat::core::nil)
```

After:
```scheme
(:wat::core::let
  [_ (:wat::core::Result/expect -> :wat::core::nil
       (:wat::kernel::send tx 1)
       "send 1 failed — receiver dropped before drain")
   _ (:wat::core::Result/expect -> :wat::core::nil
       (:wat::kernel::send tx 2)
       "send 2 failed — receiver dropped before drain")
   _ (:wat::core::Result/expect -> :wat::core::nil
       (:wat::kernel::send tx 3)
       "send 3 failed — receiver dropped before drain")]
  :wat::core::nil)
```

The `_` binding STAYS (you're still discarding the Result/expect's return — which is `:wat::core::nil` on success; panic on Err). The discard is now of an intra-world `:nil`, not of a cross-world comm Result. Mini-TCP discipline honored: the protocol-compliance measurement IS made (via `Result/expect`); the Ok arm yields nil; you discard the nil.

**For the lifeline probe recv:** read the surrounding code to determine the recv's element type, then pick the appropriate Result/expect shape. The same pattern applies; the message string should reference what the recv was reaching for.

---

## The wat-test proof gate

Two tests currently fail because of the violations. Both MUST pass post-migration.

| Test | Verifies |
|---|---|
| `cargo test --release --test wat_arc170_stone_a_drain_and_join` | All 4 sub-tests pass (T1-T4) |
| `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` | The probe passes |

These two were the "pre-existing failing" tests at session start. Closing this stone makes both green. Workspace baseline drops by 2.

---

## Verification protocol

1. Read `tests/wat_arc170_stone_a_drain_and_join.rs` around lines 90-125 to understand the test fixture context
2. Apply the migration to lines 101-103 (three sends)
3. Read `tests/probe_lifeline_orphan_clean_via_fork_program.rs` around line 209 to understand the recv context
4. Apply the migration to line 209 (one recv) — picking the appropriate :T per recv's element type
5. Run `cargo build --release 2>&1 | tail -5` — must compile clean
6. Run both named tests:
   ```bash
   cargo test --release --test wat_arc170_stone_a_drain_and_join 2>&1 | tail -5
   cargo test --release --test probe_lifeline_orphan_clean_via_fork_program 2>&1 | tail -5
   ```
7. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-COMM-PURGE.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY.

1. **Either named test FAILS post-migration.** STOP. Revert your edits. Inscribe in SCORE: which test, what diagnostic the failure emitted. Do not investigate WHY. Return.

2. **cargo build FAILS** (e.g., the Result/expect form's :T annotation is wrong). STOP. Inscribe the compile error. If the type mismatch is obvious from cargo's diagnostic (Result/expect expected :T but got :U), fix the :T and retry ONCE. If still failing, revert + report.

3. **You see a failing test OUTSIDE the two named.** STOP. The cascade is bounded to these 4 sites; any other test failure is OUT OF SCOPE.

4. **You feel the urge to "fix" any other site in the workspace.** STOP. ONE stone, four sites in two files, nothing else.

5. **You feel the urge to modify the substrate code (src/check.rs etc.).** STOP. This stone is test-fixture cleanup ONLY. No substrate edits.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-COMM-PURGE.md`:

1. Header: `# Arc 212 stone δ-comm-purge — SCORE: protocol-violation purge`
2. Summary: 4 sites migrated from `_ <comm-call>` to `_ (Result/expect -> :T <comm-call> "msg")`; protocol-compliance measurement now honest at each site
3. Per-site changes: file:line + the chosen :T type + the error message string
4. Verification: two lines (one per named test) showing pass count
5. Build line: cargo build clean
6. Workspace baseline: was 2 failing (both surfaced + diagnosed in this stone); now ?
7. Mode classification

---

## Constraints

- Edit ONLY the two test files named above
- Do NOT edit any substrate code (src/*.rs)
- Do NOT edit any other test file
- Do NOT edit any wat-tests/*.wat file
- Zero git operations (orchestrator commits)
- Run ONLY the two named tests + cargo build
- No `cargo test --workspace`

---

## Time prediction

5-10 min Mode A. The migration is mechanical (wrap each comm call in Result/expect); the :T determination requires brief context-reading per site (~1 min); two cargo test invocations.

---

## Mode classification

- **Mode A:** 4 sites migrated; both named tests pass; cargo build clean; SCORE written
- **Mode B (acceptable):** migration applied; a test still fails (e.g., :T was wrong); honest report
- **Mode C:** STOP rule broken (edited substrate, "fixed" other sites, scope-crept)

The substrate teaches; you listen; you purge the violations; nothing else.
