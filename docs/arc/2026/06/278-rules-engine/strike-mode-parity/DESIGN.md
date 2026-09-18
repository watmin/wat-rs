# DESIGN — mode parity: `--check`'s verdict must equal the run path's freeze verdict

> Drawn 2026-09-05 at HEAD `91b8966e8` (docs) / `21530efab` (code). Source: vigilia 2026-09-05
> CLASS Ω (`../vigilia-2026-09-05/WORK-LIST.md`), found by `circumspicere`, **re-driven
> independently by the orchestrator on a fixture of its own** before this was written.

## Why

`wat --check` is sold, in its own comment at `src/distribution/mod.rs:346-350`, as
*"side-effect-free verification suitable for editor save hooks and agent sweep loops."*
**It is a broken oracle in both directions, and both arms are driven.**

| fixture | `--check` | run | same binary? |
|---|---|---|---|
| empty file | **rc=0** | rc=4 `MainSignatureError` | yes |
| freeze-time non-tail recursion, depth 1000 | **134 ×6** (`SIGABRT`, stack overflow) | **0 ×6** | yes |

A sweep loop believes it. It green-lights programs that cannot start, and it aborts on programs
that run.

## The mechanism, read on the disk (not inherited from the ward)

`run_with_args` is a **linear prefix of process setup with four mode returns cut into it**:

```
src/distribution/mod.rs
  :311   set_argv(ambient_argv)
  :316   if Mcp        -> return mcp::serve()          ← returns HERE
  :351   if check_only -> return (0 | 1)               ← returns HERE
  :394   init_shutdown_signal() + install_substrate_signal_handlers()
  :397   RLIMIT_STACK raise to min(1 GiB, rlim_max)    ← arc-261 stopgap
  :410+  freeze under a panic boundary, then :user::main
```

Two of the four modes return **above** the stack raise. Every mode that evaluates wat needs it —
`--check` runs `startup_from_source`, which evaluates top-level forms at freeze time — so the
crash arm is a **missing precondition**, not a wrong line. That is why eighteen inward wards did
not see it: **ordering is not a line**, and a prefix of setup has no line to be wrong.

⛔ **The signal wiring is NOT in scope and must not be hoisted.** `mod.rs:313-315` states its
reason: MCP *"never wants the entry-file read, the `:user::main` invocation, or the signal wiring
below."* That reasoning is sound and is a deliberate design statement. **This strike does not
touch it.** Only the stack raise is a universal precondition.

## The one contract decision, pinned

**"Agree" is NOT "same exit code."** `--check` is documented to exit 1 on freeze failure where the
run path exits 3; that difference is by design and must stay. The invariant is:

> **`--check` reports success if and only if the run path freezes the same source, and `--check`
> terminates normally whenever the run path does.**

Two arms, both falsifiable:
- **SOUNDNESS** — `--check` rc=0 ⟹ the run path does not fail before `:user::main`.
- **LIVENESS** — the run path terminates normally ⟹ `--check` does not die by signal (134/139).

Gate the **invariant**, never the observed numbers: no depth constant, no exit-code table beyond
{0 = accepted, non-zero = rejected, signal = neither}. The ceiling moved between the ward's fixture
and the orchestrator's (400–600 vs 600–1000) because frame size differs. **A depth constant in
this gate would encode a measurement, not a law.**

## Scope

**IN:** one gate driving the **real binary as a subprocess** over a fixture set, asserting the two
arms. Fixtures in `tests/cli/` beside their driver, per the adjacent-fixture convention.

**OUT, affirmatively cut — not deferred:**
- The **cure**. This strike ships the instrument only. A cure landed without a RED gate is a claim.
- The **signal wiring** (see the ⛔ above).
- **Ω3, the silent segfault** (`139` with 0 bytes of stderr when the raise takes effect; `134` with
  a message when it does not). It is a *separate* defect and interacts with the cure: hoisting the
  raise would spread the silence to `--check` and `--mcp`. It must be decided on its own evidence.
- **Ω2, the MCP sandbox and the stale installed binary.** Builder's call; not a rider's.

## Why the gate before the cure

`src/rete/kernel/tests/right_index_counter_invariant.rs` — landed in this arc as "the definition of
done" for D2 — has **one possible outcome**; the invariant it asserts holds by construction. And
`experiri` mutation-proved this cast that four `record_token` bypass sites are **invisible** to the
census gate built to catch exactly them (100 tests, 100 passed). **A gate that cannot fail is not a
gate**, and this arc has now shipped two. This one must be RED at HEAD, on both arms, before any
cure is written.
