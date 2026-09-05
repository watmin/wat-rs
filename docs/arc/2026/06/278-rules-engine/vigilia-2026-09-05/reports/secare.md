# ward `secare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

# `secare` — vigilia report, wat-rs @ `21530efab` (`grok-rete`)

Everything below was read this session; every `file:line` pairing was grepped.

---

## L1 — one defect, two arms, one root

### The root: single-shot `PollAdd`s are never cancelled on a **persistent** io_uring, and one consumer reads the completion queue **positionally**

`Receiver::recv` (`src/comms/process.rs:717`) alternates two functions that share **one persistent ring** (`self.ring`, `IoUring::new(4)` at `src/comms/process.rs:981`, `1996`, `2050`, `2074`):

- `wait_for_data_or_cascade` (`src/comms/process.rs:1035`), called at `:734`
- `read_into_acc` → `uring_read_into_acc` (`src/comms/process.rs:1238`), called at `:781`

`wait_for_data_or_cascade` pushes **two** single-shot `PollAdd` SQEs — `DATA_TOKEN = 1` (`:1040`, `:1050`) and `BROADCAST_TOKEN = 2` (`:1041`, `:1060`) — then `submit_and_wait(1)` (`:1081`) and drains whatever is *currently* ready (`:1091`). **It never cancels the arm that did not fire.** There is no `AsyncCancel` or `PollRemove` anywhere in the file (grepped: only the word "uncancellable" in a comment at `:355`). A single-shot `PollAdd` stays armed in the kernel until its fd becomes ready or the ring is destroyed — and the ring is not destroyed, it is the Receiver's persistent one.

The common path is `got_data = true, got_broadcast = false`, so **the broadcast poll (`user_data = 2`) survives the return** and stays armed across the loop at `:728–760`. Each further loop iteration (a multi-line EDN frame needs several reads) pushes another pair and leaks another armed broadcast poll.

**Writers into that one completion queue:** this thread's `PollAdd(1)` at `:1050`, this thread's `PollAdd(2)` at `:1060`, this thread's `Read(user_data = 1)` at `:1251`, and the timer `Read(user_data = 1)` at `:1306`. The *completion timing* of the broadcast poll is driven by a **different thread** — the shutdown worker's wake-byte `libc::write(broadcast_w_fd, …)` at `src/runtime.rs:472-473`, or the `POLLHUP` when that `OwnedFd` drops.

---

**Arm A — the Read step eats a poll completion (silent wire corruption).**

`uring_read_into_acc` (`src/comms/process.rs:1238`) does:

```rust
let cqe = ring.completion().next().ok_or(())?;     // :1271  — no user_data check
let result = cqe.result();
if result < 0 { return Err(()); }
let n = result as usize;                            // :1276
acc.borrow_mut().extend_from_slice(&buf[..n]);      // :1277
```

Interleaving:
1. `recv` → `wait_for_data_or_cascade` returns `DataReady`; the broadcast poll (token 2) is still armed.
2. `recv` → `uring_read_into_acc` pushes `Read(user_data = 1)` (`:1251`) and calls `submit_and_wait(1)` (`:1265`).
3. The shutdown worker writes the wake byte (`src/runtime.rs:472`) — or its `broadcast_w_fd` drops. The stale broadcast poll completes.
4. `submit_and_wait(1)` is satisfied by **that** completion (min_complete=1 returns as soon as the CQ is non-empty).
5. `completion().next()` (`:1271`) returns the **poll's** CQE. `cqe.result()` is the `revents` mask (`POLLIN` = 1, or `POLLIN|POLLHUP` = 17), not a byte count.
6. `n = 1` (or 17). `acc.extend_from_slice(&buf[..n])` splices **n bytes of untouched scratch** (`buf` is `[0u8; 4096]`, so NULs) into the frame accumulator, and the real Read's CQE is left pending for the *next* call to mis-consume in the same way.

**Consequence:** NUL bytes injected into the newline-framed wire; `decode_frame` (`:1127`) fails UTF-8/EDN, or the framer desynchronises; and from that point every CQE is off-by-one against its operation, permanently. It fires precisely when a shutdown cascade races an in-flight `recv` — the one scenario the whole cascade exists for. At shutdown, *all* N leaked broadcast polls complete at once, so the CQ floods and the misalignment is N deep.

**Arm A′ — the mirror, and the comment that denies it.** `wait_for_data_or_cascade`'s drain (`:1095`) matches on `cqe.user_data()` with:

```
// Unreachable: we only push two SQEs with these two tokens.     :1098
_ => return Err(RecvError::Disconnected),
```

That comment is **false**. `uring_read_into_acc:1251` and `uring_read_n_into_scratch:1306` both push `user_data(1)` onto the *same* ring. A stale `Read` completion is therefore **indistinguishable from `DATA_TOKEN`** — it reads as `got_data = true` and `recv` proceeds to Read with nothing to read; and a stale Read CQE with `result() < 0` is reported at `:1091` as a channel `Disconnected`.

**Race class:** undefined-order / silent-reorder — two independently-in-flight operations share one completion queue and one consumer takes from it positionally.
**Disjoint direction:** every consumer dispatches on `cqe.user_data()` (each op gets its own slot); *or* `wait_for_data_or_cascade` issues `AsyncCancel` for the un-fired token and drains its CQE before returning. Distinct tokens per operation kind are a prerequisite either way — `1` is currently used by three different operations.

---

**Arm B — `Select` leaks armed arm-polls across calls (liveness).**

`Select`'s ring (`src/comms/process.rs:1441`, `RingSlot` = `Option<(IoUring, u32)>` at `:113`) is rebuilt **only on capacity change** (`:1580–1587`), so it persists across `select()` calls. Each call pushes one `PollAdd` per receiver arm plus broadcast plus listener, waits for one (`:1665`), drains what is ready (`:1672`), and returns — **un-fired arm polls stay armed**.

The drain *does* dispatch by `user_data` (`:1676–1686`), so this is not the positional bug; it is the same un-cancelled-poll root. A stale level-triggered CQE for arm K (ready at arm time, already drained since) sets `first_data_arm = Some(K)` at `:1683`, and `rx.read_into_acc()` at `:1719` then calls `submit_and_wait(1)` (`:1265`) on arm K's own ring with nothing to read — **blocking the whole select** while other arms' data goes unserved.

---

## L2

### L2-1 — an `unsafe` safety argument names one caller; there are two, and the second breaks it

`ThreadOwnedCell` is `unsafe impl Send + Sync` (`src/rust_deps/custodia.rs:42-43`). `ref_guard` (`:118`) hands out `&T` from an `UnsafeCell` while `with_mut` (`:71`) hands out `&mut T` from the same cell — both take `&self`, so Rust cannot see the conflict. The soundness argument is written as a caller contract at `src/rust_deps/custodia.rs:113`:

> "`eval_peer_select_prime` (the sole caller) upholds it by construction: guards are scoped to the eval fn, and **no user code runs while they are held**"

Grepped: `.ref_guard(` has **five** sites in `src/runtime.rs` — `32867`, `33011`, `33197` (all inside `eval_peer_select_prime`, `src/runtime.rs:32753`) and **`33756`, `33854` inside `eval_poll_prime`, `src/runtime.rs:33718`**. The "sole caller" claim is false.

And the second caller violates the contract:

| line | act |
|---|---|
| `33755-33756` | `self_guard = self_peer_cell.ref_guard(…)` — guard live |
| `33775` | `eval_inner(&args[1], env, sym)?` — **arbitrary user wat runs** |
| `33800` | `eval_inner(&args[2], env, sym)?` — **arbitrary user wat runs** |
| `33853-33856` | N more `ref_guard`s taken |
| `33914`, `34054` | `self_guard` dereferenced again |

`eval_peer_select_prime` gets it right by construction — its only `eval_inner` is at `src/runtime.rs:32776`, *before* the first guard at `32867`.

I could **not** construct UB from this today: I enumerated all 24 `.with_mut(` sites in `src/`, and none of them reaches a `PeerCell` (`Arc<ThreadOwnedCell<Option<Peer>>>`, `src/kernel/spawn.rs:142`). `close'`'s `with_mut` (`src/runtime.rs:32271`, `32309`) operates on `THREAD_PEER_TYPE_PATH` / `PROCESS_PEER_TYPE_PATH`, not `PEER_TYPE_PATH`; `src/kernel/spawn.rs:1261` is a test on a `ThreadPeerCell`. So the aliasing is prevented today by *which cell*, not by the stated discipline — and nothing gates that. Adding one `with_mut` verb on a `Peer` makes `(poll' p … …)` with a mutating arg expression an `&`/`&mut` alias, i.e. UB under `noalias`.

**Remedy:** hoist all three `eval_inner` calls above every `ref_guard` in `eval_poll_prime` (the shape `eval_peer_select_prime` already has), and correct `custodia.rs:113` to name both callers.

### L2-2 — `init_shutdown_signal_with_inputs`: check-then-act with no exclusion, and an argument silently discarded

`src/runtime.rs:329` reads `SHUTDOWN_RX_PTR`, `:332` compares `SHUTDOWN_INIT_PID` to `getpid()` — and `SHUTDOWN_INIT_PID` is stored **last**, at `:506`, after the rx/tx stores at `:367`/`:369`.

Two-thread interleaving (first init ever, both threads in the same process):
- T1 passes the guard, stores `rx_boxed` at `:367`, `tx_boxed` at `:369`, wake fd at `:388`, broadcast fd at `:407`, spawns the worker — and has **not yet** reached `:506`.
- T2 enters. `rx_ptr` is now non-null (`:329`) but `SHUTDOWN_INIT_PID` is still `0 ≠ getpid()` (`:332`), so T2 takes the **fork-child branch** at `:343` and executes `libc::close(old_write_fd)` at `:358` — closing T1's live, in-process wake write-fd — then rebuilds everything.
- T1's tx box is now unreachable, so `trigger_shutdown` (`:518`, swap-to-null on `SHUTDOWN_TX_PTR`) drops only T2's sender. Any `shutdown_rx()` reference handed out from T1's box (`:248`) **never wakes**; two shutdown workers now exist; the loser's worker blocks on a read fd nobody writes.

Reachability today: the three production callers are `src/freeze.rs:272`, `src/distribution/mod.rs:389`, `src/distribution/spawned_runtime.rs:50`, all on the main thread at bootstrap, and `run_with_args` (`src/distribution/mod.rs:217`) returns to `serve()` at `:255-256` before reaching `:389`. So the ordering is correct **today**; the guard cannot enforce it.

**Second, separable half:** the guard also **silently discards `extra_input_fds`** on a second call. The only caller passing them is `src/distribution/spawned_runtime.rs:50` (`LIFELINE_FD`). Any future path that calls `init_shutdown_signal()` earlier in a spawned runtime makes the lifeline never enter the poll set — parent-death propagation dies with no error, no warning, and a green init.

**Remedy:** store `SHUTDOWN_INIT_PID` *before* publishing `SHUTDOWN_RX_PTR` (or make the whole init a single `compare_exchange` claim on `SHUTDOWN_INIT_PID`); and make a second call carrying non-empty `extra_input_fds` an error rather than a no-op.

### L2-3 — SIGHUP/SIGUSR flags: poll-then-reset is a lost-update against the signal handler

`(sighup?)` is `flag.load(SeqCst)` (`src/runtime.rs:26496`, via `src/intrinsic/kernel/ambient.rs:157`) and `(reset-sighup!)` is `flag.store(false, SeqCst)` (`src/runtime.rs:26520`, via `ambient.rs:219`). The signal handler writes `KERNEL_SIGHUP.store(true)` (`src/runtime.rs:138`). Load-then-store is a non-atomic read-modify-write with a second writer in the middle:

1. SIGHUP #1 → handler stores `true`.
2. wat polls `(sighup?)` → `true`, begins handling.
3. **SIGHUP #2 arrives** → handler stores `true` (already true).
4. wat finishes and calls `(reset-sighup!)` → `store(false)`. **Signal #2 is erased.**

The doc at `src/runtime.rs:120-122` says the flag is "coalesced — five SIGHUPs in a burst read as one 'yes' on the next poll." That covers a burst *before* the poll. It does **not** cover an arrival *during* the handler's own reset window, which is a distinct event the program never learns about. A boolean cannot represent it.

**Remedy:** make the read the clearing act (`swap(false)` — a test-and-clear primitive), or replace the bool with an `AtomicU64` generation counter the handler `fetch_add`s and userland compares against its last-seen value. `compare_exchange(true, false)` does **not** fix it — the second signal's store is indistinguishable from the first's.

---

## L3 — judgement

- **`src/rete/kernel/session.rs:242-262`** — the D2 cure holds under reading. `JoinRightIndex`'s `buckets`/`indexed_n` are private, `RightIndexWriter::push` (`:257`) is the only mutator, `writer()` (`:277`) is its only constructor, and `get()` (`:285`) is shared-only. There is no accessor handing out `&mut` to the buckets. The bypass is genuinely unrepresentable.
- **`src/alloc_counter.rs:132-169`** — the A4 cure holds: `or_insert` at `:305` and `:333`, and `LAST_ORIGIN`'s "a hit cannot be stale" argument (`:180`) is sound *because* origins are write-once. One live conflation remains, and it is stated rather than hidden: `SessionOriginKey = Option<u64>` (`:130`) collapses **every** hand-assembled session on a thread into one origin. That is the `_ =>`-holds-two-facts shape (absent identity vs. shared identity), and it is the only key that can carry a memory-ceiling defect forward without the map noticing. Not a finding; a watch point.
- **`src/value/pmap.rs:37-88`** — the lane partition is correct. `LANE.fetch_add(Relaxed)` (`:38`) still yields distinct values per RMW in modification order, so uniqueness holds; `next_intern`'s wrap check `next & INTERN_SEQ_MASK == 0` (`:80`) takes a fresh lane exactly at the boundary and cannot bleed into a neighbour. No finding.
- **`src/vm_registry.rs:115-139`** — textbook double-checked read-lock/write-lock with a re-check at `:126`. Correct. (`.read().unwrap()`/`.write().unwrap()` will poison-panic every later `get` if a build panics under the write lock — cosmetic.)
- **`src/rete/kernel/arm.rs:827-828`** — the cross-thread `ArmLease` bound is *stated*, and `ArmLease` is `Send + Sync` and rides wat as a `RustOpaque` (`payload: Box<dyn Any + Send + Sync>`, `src/rust_deps/marshal.rs:329`), so a lease crossing a thread would decrement the *wrong* table if that thread armed the same network id. It is not reachable: `with-network` binds `lease` in its own `let` (`wat/rete/syntax.wat:325`), outside `body-fn`'s scope, and `adopt-session-lease` is `#[restricted_to]` `:wat::rete::` (`src/runtime.rs:5687`). Keep the fence.

---

## What I could not check, and why

- **I could not build the trigger for L1.** Read-only ward: no build, no floor run. Arm A is derived from reading `wait_for_data_or_cascade` (`1035–1116`), `uring_read_into_acc` (`1238–1278`), `Receiver::recv` (`717–762`), and confirming by grep that no `AsyncCancel`/`PollRemove` exists in the file. The window (broadcast-poll completion landing between `submit_and_wait(1)` at `:1265` and `completion().next()` at `:1271`) is narrow and timing-dependent — *that is why it would show up as an intermittent, unattributable process-tier failure rather than a reproducible one*. **"I could not construct it" is not "there is no trigger."** The construction I would use: hold a process peer in a multi-read frame (`pprintln` a multi-line EDN value so `recv`'s loop iterates ≥3 times, leaking ≥3 armed broadcast polls), then deliver SIGTERM so `request_kernel_stop` (`src/runtime.rs:89`) wakes the worker and it writes the broadcast byte (`:472`) mid-read; assert the received frame is byte-identical to what was sent.
- **I could not observe `cqe.user_data()` at runtime** to prove which CQE a given `completion().next()` returns. The claim rests on io_uring's documented `min_complete=1` semantics (return as soon as the CQ is non-empty) and on single-shot `PollAdd` not self-disarming when an unrelated op completes. Both are standard, neither is measured here.
- **L2-2's two-thread interleaving is unreachable today** through the three production callers I enumerated, so I could not construct it either. What I *can* show is that the guard at `src/runtime.rs:329-332` does not exclude it — the ordering is held by call-site discipline, not by the code.
- **`wat-reader`'s `fresh_scope`** (re-exported at `src/scope/mod.rs:41`) is outside the `src/`/`tests/`/`wat/` scope I was given; I did not read its mint. `src/scope/mod.rs:4` claims a prior secare pass cleared it; I did not re-derive that.
- **`tests/`** — I read `src/rete/kernel/tests/arm_lease.rs:59` only incidentally. I did not audit the test corpus for cross-test `thread_local!` leakage (the `uninstall_thread_io` / `take_ambient_stdio` reuse discipline at `src/services/client.rs:79`, `:219`). Under `cargo nextest` each test is its own process so the leakage class is masked; under `cargo test` it is not, and I did not check which tests would break.
- **`src/rete/kernel/census.rs`'s `#[cfg(test)]` asymmetry** — I mapped the gating (`:243`, `:298`, `:324`, `:344`, `:380`, `:384`, `:433`, `:447`, and the `:493-497` note explaining why the D2 counters are statement-gated instead of no-op-twinned) and found the two-build shapes consistent, but I did not diff a `--release` against a `--cfg test` expansion to prove no `#[cfg(test)]` statement carries a load-bearing write.
