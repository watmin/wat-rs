# BRIEF — structured-peer-death Sub-stone A: the thread crash channel carries the EDN envelope

## The work (one paragraph)

When a thread peer's body panics, the death site renders the full structured reason as a
`#wat.kernel/AssertionFailure {…}` EDN envelope — but only the bare message String is sent over
the peer's crash channel; the structure is discarded. Fix the **crash-send site** to send the
**envelope** (which carries `:actual`/`:expected`/`:frames`/…), exactly as the process tier
already sends its `#wat.kernel/ProcessPanics` envelope. `recv'` are UNCHANGED — they
already surface whatever the crash channel carries; making the channel carry the envelope makes
them honest. The committed RED probe `tests/nursery/probe_arc209_structured_peer_death.rs` is the
gate: a thread peer dies via `assertion-failed!` with `actual=ACTUAL-42173`/`expected=EXPECTED-99731`;
`recv'` must surface a reason containing **both** structured fields. Make it GREEN.

## Read in order (the rooms)

1. **`tests/nursery/probe_arc209_structured_peer_death.rs`** — the gate. The exact death shape and
   the assertion (the raised reason must contain `ACTUAL-42173` AND `EXPECTED-99731`, not just the
   message).
2. **`src/panic_hook.rs:126`** `write_assertion_failure<W: Write>` + **`:137`** `payload_to_edn`
   (`pub(crate)`) — the envelope renderer. `write_assertion_failure` does
   `format!("#wat.kernel/AssertionFailure {}", wat_edn::write(&payload_to_edn(payload)))`. This is
   the string you must produce.
3. **`src/kernel/spawn.rs:~470-474`** — the crash-send site:
   ```
   if let Err(payload) = outcome {
       let (message, _assertion) = crate::runtime::extract_panic_payload(payload);
       let _ = crash_tx.send(message);            // ← discards the structure
   }
   ```
4. **`src/runtime.rs:18840`** `extract_panic_payload` — returns `(String, Option<AssertionPayload>)`.
   The `Option<AssertionPayload>` is the structure to render when present.
5. **`tests/nursery/probe_arc259_thread_crash_reason.rs`** — the precedent (proved the *message*
   travels). Your change must keep that test green (the message is still inside the envelope).

## Implementation sketch (fill it, don't invent the shape)

**Step 1 — `src/panic_hook.rs`: factor a `String`-returning envelope helper** (so the crash-send
site can reuse it without a `Write` sink):
```rust
/// The `#wat.kernel/AssertionFailure {…}` envelope as a String (no trailing newline).
pub(crate) fn assertion_failure_envelope(payload: &AssertionPayload) -> String {
    format!("#wat.kernel/AssertionFailure {}", wat_edn::write(&payload_to_edn(payload)))
}
```
Have `write_assertion_failure` call it (`out.write_all(format!("{}\n", assertion_failure_envelope(payload)).as_bytes())`) so there is ONE renderer — no duplication.

**Step 2 — `src/kernel/spawn.rs` crash-send site:** stop discarding; send the envelope when an
assertion is present, else the plain message:
```rust
if let Err(payload) = outcome {
    let (message, assertion) = crate::runtime::extract_panic_payload(payload);
    let reason = match assertion {
        Some(a) => crate::panic_hook::assertion_failure_envelope(&a),
        None => message,
    };
    let _ = crash_tx.send(reason);
}
```
(Confirm `panic_hook`'s visibility path from `spawn.rs`; if `assertion_failure_envelope` needs to
be reachable, `pub(crate)` is sufficient — both are in the `wat` crate.)

That is the whole change. The crash channel stays `Receiver<String>`; `recv'` are not
touched — they surface the (now-rich) String.

## Blast radius

`src/panic_hook.rs` (one new `pub(crate)` helper + route `write_assertion_failure` through it) and
`src/kernel/spawn.rs` (the one crash-send site). **Do NOT** touch `recv'`/`select'`, the
`PeerRecvError` enum, the crash channel type, or the process tier — they are out of this strike.

## STOP triggers (surface the gap; do not improvise past them)

1. If making the probe green appears to require editing `recv'` (not just the crash-send
   site) — STOP and surface it. The design says the crash channel carries the envelope and recv' is
   unchanged; if that's false, it's a finding for the Inquisitor, not a fix to apply.
2. If `payload_to_edn` / `wat_edn::write` is not reachable as `pub(crate)` from a `String`-returning
   helper — STOP and report the exact visibility error; do not duplicate the EDN-rendering logic.
3. If the precedent test `probe_arc259_thread_crash_reason` goes red — STOP; the message must remain
   recoverable inside the envelope.

## Gate (run each, READ the output, report the verbatim final line — do NOT chain a commit)

1. `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1`
   → **1 passed** (the raised reason contains `ACTUAL-42173` AND `EXPECTED-99731`).
2. `cargo test --release -p wat --test nursery probe_arc259_thread_crash_reason -- --test-threads=1`
   → **1 passed** (the precedent — message still travels).
3. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds
   (arc-255 ×2, undefined-builtin ×2), zero new.
4. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken.
5. `cargo build --release` clean; `cargo clippy` clean in the two touched files.

## Report back

The helper + the crash-send change as written. Verbatim final line of each gate row. Any STOP hit +
the exact error. Any honest delta vs. this sketch. Do **NOT** commit — the Inquisitor weighs against
its own re-run and commits.
