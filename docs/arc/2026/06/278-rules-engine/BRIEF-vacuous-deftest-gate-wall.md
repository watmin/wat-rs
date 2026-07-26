# BRIEF — the vacuous-gate wall: a failed `deftest` MUST NOT be readable as a pass

> Builder-ruled 2026-07-25, on discovery: *"we pivot and address this - now ... i do not care what amount of work
> is necessary for this - this behavior is unacceptable - what is the type check that sets all heretics ablaze in
> one shot?"*
>
> This is R55 `REVOLVTIONE, NVLLA LARVA` recurring one layer over. That realization closed the harness's *swallow*.
> This is a different hole in the same organ: **the harness reports honestly and the Rust caller does not read the
> report.** A gate that certified a shipped stone was proven to gate nothing.

## The heresy (VERIFIED by the orchestrator, not relayed)

`tests/rete/probe_arc278_sqlite_interop.wat:33` was mutated `(:wat::test::assert-eq n 1)` →
`(:wat::test::assert-eq n 4242)` — an assertion that cannot hold — and the test **PASSED**:

```
PASS [0.150s] (1/1) wat::rete probe_arc278_sqlite_interop::sqlite_interop
Summary [0.160s] 1 test run: 1 passed
```

(Mutation reverted; the file is clean. The cache Stone 1 rider found this independently while mutation-checking
its own gate, and reported the same result.)

**The mechanism, grounded:**
- `pub fn call_beside(caller_rs: &str, fn_name: &str) -> Result<Value, RuntimeError>` — `src/freeze.rs:714`.
- `:wat::kernel::RunResult` — `src/types.rs:1492` — is a **record with exactly one field**:
  `failure <- :wat::core::Option<:wat::kernel::Failure>`.
- A fired assertion lands a `Failure` in that slot. The *evaluation* succeeded, so `call_beside` returns `Ok`.
- The caller asserts `result.is_ok()` — which answers **"did it evaluate?"**, while every author who wrote it
  believed it answered **"did it pass?"**.

Every `assert-eq` inside such a fixture is decoration. `probe_arc278_sqlite_interop` currently proves only that the
file froze and ran.

## The ONE-SHOT type check (the ablaze — this is the whole strike)

**Change the return type of `call_beside` (`src/freeze.rs:714`) to a `#[must_use]` outcome enum that has no
`is_ok()`.**

```rust
#[must_use]
pub enum DeftestOutcome {
    Passed,
    Failed { failure: Value },          // the structured :wat::kernel::Failure
    DidNotRun { error: RuntimeError },  // never evaluated (freeze/runtime error)
}
```

`Result` is the heresy-enabler: it hands out `.is_ok()`, a method whose meaning ("evaluated") is one question away
from the meaning every caller assumed ("passed"). Remove that method from the type and **every offender becomes a
compile error simultaneously** — `E0599: no method named 'is_ok'`. The compiler enumerates the worklist; nobody
audits anything. `#[must_use]` additionally catches any site that discards the outcome entirely.

Because `Failed` carries the structured `Failure`, each migrated site prints the real located diagnostic instead of
a boolean.

**Do not soften this into "make `call_beside` fold the failure into `Err`."** That would silently *fix* the call
sites while revealing nothing — the vacuous gates would go green again and we would never learn which ones were
never gating. The compile error IS the deliverable.

## The two walls that make the shape unrepresentable (not merely caught)

1. **`RunResult` becomes an enum, not a record with an ignorable `Option` slot.** That record shape is what let the
   Rust side look away: a reason-free pass and a failure wear the same type. Mirror the established outcome walls
   (`RecvOutcome` / `SendOutcome`, R53/R57) — a matchable variant per outcome, so a failure cannot be read as a
   pass. Ground the existing enum walls in `src/types.rs` (`RecvOutcome` ~:1222, `SendOutcome` ~:1269) and follow
   that exact shape.
2. **Block the wrong channel.** Once the verdict-returning verb exists, plain `call_beside` must **refuse** a
   `deftest` target rather than hand back an ignorable `Value`. Running a test through the ignore-the-verdict path
   should have no form. If the symbol table cannot distinguish a `deftest`-defined symbol at call time, STOP and
   report — do not fake the check.

## Method

1. **Prove the heresy RED first** — a probe that a fixture with a deliberately-false assertion is reported as a
   FAILURE. It must fail at HEAD (today it passes) and pass after the wall. This is the acceptance gate.
2. **Land the type change**, then let the compiler produce the migration list. Work it to zero.
3. **At each migrated site, do not just make it compile — make it *bite*.** For any gate you touch, mutate one of
   its assertions, confirm the test now fails, revert the mutation. A site that still passes under mutation has not
   been fixed. Report the count of gates that were genuinely vacuous (i.e. that fail once the wall is up).
4. Some `call_beside` callers legitimately call a non-`deftest` wat fn and want the `Value`. Those get an
   explicitly-named accessor; they are NOT collateral damage to be papered over.

## Blast radius

Intentionally large and builder-authorized (*"i do not care what amount of work is necessary"*). `src/freeze.rs`
(the signature + the outcome enum), `src/types.rs` (`RunResult` → enum), every `call_beside` call site across
`tests/` and `src/`, and any wat-side consumer of `RunResult`'s `failure` field.

**STOP + report rather than improvise** if: the `RunResult` enum flip cascades into the wat stdlib in a way that
changes the `deftest` surface authors write (that is a language-surface change and needs a ruling); or the
deftest-vs-plain-fn distinction is not available at call time.

## Gate

- The RED probe from step 1, green.
- Every previously-vacuous gate either genuinely passing or genuinely failing — **and the genuinely-failing ones
  reported, not fixed by weakening the assertion.** A real regression uncovered by this wall is a WIN and must be
  surfaced, never tuned away.
- `cargo build --release` clean; `cargo nextest run --release` — report the **Summary line verbatim**.
  Current floor: **4164 passed, 314 skipped**. Expect the count to move: this wall may legitimately turn
  previously-green vacuous tests RED. That is the point. Report every one with its real diagnostic.
- Run everything FOREGROUND. Do NOT commit — the orchestrator weighs by their own re-run and commits.
