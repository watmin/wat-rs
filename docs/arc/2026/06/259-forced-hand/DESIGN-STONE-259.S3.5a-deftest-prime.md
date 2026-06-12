# DESIGN — Stone 259.S3.5a — `deftest'` / `deftest-hermetic'` (the test layer on the new substrate)

## Goal

Stage primed `deftest'` / `deftest-hermetic'` beside the legacy `deftest` / `deftest-hermetic`,
on the post-arc-214 substrate, so the test harness rides the same rails as the code it tests.
Front-half of S3.5 (the back-half retires the legacy `run-threads` reflection-macro + the
unprimed `Thread`/`Process` structs). Unblocks native wat coverage of `bracket.wat`.

## The grounded landscape — three spawn shapes

1. **`:wat::kernel::spawn-thread`** (`runtime.rs:18850`, arc 114) — body
   `Fn(Receiver<I>, Sender<O>) -> nil` (the channel-transport vestige); allocates input/output
   + a one-shot **`SpawnOutcome`** channel on **`comms::thread::pair`** (clean, arc-214 Stone 6.1);
   runs the body in `catch_unwind`; packages the outcome; `Thread/join-result` reads
   `SpawnOutcome → Result<(), Vec<ThreadDiedError>>` (message and all). **The outcome-capture is
   already clean; the legacy is the channel-transport body shape + the unprimed `Thread` struct.**
   `run-thread` (the test macro) uses this with an unused-channel wrapper `(fn [_in _out] ~body)`.
2. **`spawn-process`** — body `[] -> nil` (clean); outcome-capture via `Process/join-result`.
   `run-hermetic` uses this.
3. **`spawn-program'`** (`spawn.rs`, arc 259) — the **self-peer**: hands the prog its `Peer'<S,R>`;
   the prog streams via `send'`/`recv'`; **the result/panic is caught and DISCARDED**
   (`spawn.rs:455-458`, `let _ =`); death is signaled only by the channel dropping. A server has
   no outcome.

## The reframe — a test is NOT a self-peer

A `spawn-program'` self-peer is a **server** (streams, discards outcome). A test is a **one-shot
computation with an outcome** (pass / structured-failure). They are different shapes, and the
substrate already has both. **`deftest'` rides the one-shot outcome-capture (shape 1/2), NOT
`spawn-program'` (shape 3).** Forcing deftest onto the self-peer would make it expose an outcome it
deliberately discards — conflating *server* and *one-shot* in one primitive.

## The fork — the four questions

**How does `deftest'` obtain the body's outcome as a value?**

**(b) un-discard `spawn-program'`'s self-peer catch** (make `Thread'` expose its outcome):
- **Obvious? NO** — muddies the self-peer: is it a server or a one-shot?
- **Simple? NO** — braids two roles into one primitive; adds an outcome channel + read verb to a
  primitive that is clean *precisely because* it discards.
- **Disqualified** (Obvious + Simple).

**(a) ride the existing one-shot outcome-capture** (the `SpawnOutcome` mechanism, already on
`comms::thread::pair`), modernized:
- **Obvious? YES** — it is exactly `run-thread`/`run-hermetic`'s existing model.
- **Simple? YES** — reuses the `SpawnOutcome` capture; the only new bit is a clean `[] -> nil`
  *thread* one-shot (the process side already is clean).
- **Honest? YES** — keeps *server* (self-peer) and *one-shot* (outcome-capture) as distinct
  primitives. The "no user join" the system annihilated was the **streaming-peer lifecycle** join
  (replaced by RAII Drop) — not a one-shot computation's outcome read.
- **Good UX? YES** — `deftest'` looks like `deftest`.
- **CHOSEN.**

## The build (Path a)

- **`deftest-hermetic'`** — `run-hermetic` already spawns a clean `[] -> nil` body via
  `spawn-process` + `Process/join-result` → `RunResult`, on the clean substrate. `deftest-hermetic'`
  = prime it (primed driver, drop the unprimed `Process` struct). Likely thin.
- **`deftest'`** (thread, the cheap path) — needs a clean `[] -> nil` one-shot **thread** spawn that
  captures the `SpawnOutcome` (the thread analog of `spawn-process`'s clean shape). Today only
  `spawn-thread` captures a thread outcome, and it carries the channel-transport vestige. The stone:
  a primed `spawn-thread'` (`[] -> nil`, reusing the existing `SpawnOutcome` capture +
  `comms::thread::pair`) read by a one-shot `join-result'` → `RunResult`. This `join-result'` is the
  one-shot-outcome read — distinct from the annihilated streaming-peer join.
- **`deftest'` macro** = `(defn ~name [] -> :wat::test::TestResult (:wat::test::run-thread' ~body))` —
  mirrors `deftest` exactly, on the primed driver.

### Build step 1 (ground before the strike)

Settle the exact reuse: is `spawn-thread'` a thin new verb (`[] -> nil`, reusing the
`SpawnOutcome`/`comms::thread::pair` machinery `eval_kernel_spawn_thread` already builds), or a
`[] -> nil` mode of the existing capture? Read `runtime.rs:18850-18960` + the `Thread/join-result`
reader; the SpawnOutcome→Result conversion (`runtime.rs:18551-18556`) is reused verbatim.

## The probe (RED at HEAD) — the `run-thread.wat` two-path, on the new substrate

`tests/nursery/probe_arc259_deftest_prime.rs`:
- A `deftest'` with a **PASSING** assertion → its `RunResult.failure` is `None`.
- A `deftest'` with a **FAILING** assertion → its `RunResult.failure` is `Some(_)` (carrying the
  assertion message — the structured failure, not a bare panic).

`compute` runs both and returns an i64 code (`+1` if passing has no failure, `+2` if failing has a
failure) → expect `3`. RED at HEAD: `deftest'` does not exist → startup/expansion fails.

## Out of scope (affirmative cuts)

- **Migrating existing deftests** to `deftest'` → after `deftest'` proves out (the sweep).
- **Retiring legacy `run-threads` / unprimed `Thread`/`Process`** → S3.5 back-half.
- Native wat coverage of `bracket.wat` → rides `deftest'` once it ships.
