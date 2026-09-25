# WEIGH — STONE 255.31: a thread address is inert data — ACCEPTED

**Executor: grok via pulsare, commit `6c2c12f19`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| the registry | `Mutex`/`register_rendezvous`/`resolve_rendezvous`/`AtomicU64` in `src/kernel/address.rs` | **0**: gone |
| floor | `.floor/2026-09-25T06-15-55Z` | `6116 tests run: 6116 passed (9 slow), 22 skipped` |
| echoed thread address, and the child's copy | `wat …_thread_address_to_process.wat` | both `Rejected: a thread address is dialable only through the live value; one that crossed a wire is inert.` (pre-stone the echo connected) |
| thread defservice · kwargs pool | `wat …` | `"echo:hi"` · `["echo:a" "echo:b" "echo:c"]` |
| 255.30's negative row | `ls` | now `….wat.bad` (census back to **215**) |

Taken from the SCORE: clippy 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0; ledger 208.

## What landed

- **No `Mutex` in the address layer.** The wire record stays, so a `Status`/`PoolMsg` holding an address
  still encodes, but it is **inert**. Decoding one anywhere gives `Rejected` with one honest sentence.
- `Rejected` over `Refused`: the SCORE's reason is that `Refused` is documented retryable, and a wire copy
  never becomes live. That is correct.
- The live value still dials, and the dropped-listener arm is untouched, as briefed.

## ⭐ The outcome-vocabulary table (in the SCORE): claims that are false, measured

The table covers every variant, with file:line for each locus. The **false claims** it found, which are
the builder's to rule on:

1. **`ConnectFail::Refused` on a thread** (dropped listener): documented *retryable, "the server may come
   up"*. **False.** A dropped crossbeam receiver never returns. On the process tier it is also not
   revivable, because `listener'` autobinds a **new** name.
2. **`RecvOutcome.Closed` for `FrameTooLarge`** on a socket `Peer'` (wildcard arm): *"genuine clean EOF"*.
   **False.** `message.rs:620` says an over-budget frame is not a clean close.
3. **`RecvOutcome.Lost` for an EDN decode failure or a `Reply::Failed`:** *"abnormal loss"*, with a
   `LociDiedError` carrier. **False as "died"**: the peer can be alive. `ServiceEvent` has `Malformed` for
   that fact; `RecvOutcome` does not.
4. **`AcceptFail::Closed` folds a stop (`Shutdown`) into "address dropped".**
5. **`ServiceEvent.Shutdown`:**
   - on the process `poll'` re-poll, index 0 is `Shutdown` for **both** `Ok` and `Err`, so an admin
     payload on that path is reported as shutdown;
   - `select'`'s stop arm also says shutdown.
6. **`ServiceEvent.Closed` on thread `poll'` and on peers-only `select'`** covers every error, crashes
   included. `Lost` is produced only by `select'` with a crash channel. `FrameTooLarge` is `Closed` on
   peers-only `select'` but `Rejected` on `poll'`: **one fact, two names.**
