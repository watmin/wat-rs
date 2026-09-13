# FINDING — the crash surface is enumerated

Drawn `9d71dca47`. Executor: grok, 2026-09-12, branch `sns-sqs`. **No substrate change.**
`git diff --stat -- src/ wat/` is empty.

Reachable = a command that can be run TODAY which makes the variant fire and be observed.

---

## 0. Instrument — re-derived, fifth defect

Slice `src/types.rs` on `env.register_builtin(TypeDef::Enum(EnumDef {` (29 registrations).
Match **both** `EnumVariant::Unit("X"` and `EnumVariant::Tagged { name: "X"`.

DESIGN recorded four instrument defects. This pass needed a **fifth**, or the controls fail:

| # | defect | what it did |
|---|---|---|
| 1–4 | DESIGN's table | window bleed, `name:` blindness, leaf-name phantom duplicates |
| **5** | matching `Unit("X")` | the source spells `EnumVariant::Unit("X".into())`. A pattern that requires the closing `)` after the quote sees **zero** Unit variants. RecvOutcome collapses to the three Tagged names (Message/Lost/Malformed); Closed is invisible. |

Fix: `EnumVariant::Unit\("([^"]+)"` — stop at the quote, ignore `.into()`.

**Controls (this pass, all pass):**

- `RecvOutcome` = **6** — Message, Closed, Stopped, TimedOut, Lost, Malformed
- `Closed` present on Recv, Send, TrySend, Close, Accept
- `Item` attributes only to `:wat::stream::NextOutcome` (with Exhausted)
- two distinct `ReadFrameOutcome` enums: `:wat::kernel::` and `:wat::io::IOReader::`

Wat-side (`wat/service.wat:3774`, `:3795`):

- `CallOutcome` = Answered, Lost, Closed, DeadlineFired, Malformed (5; 4 failure)
- `StopOutcome` = Stopped, Gone, GaveUp (3; 2 failure)

**27 failure variants / 10 enums**, matching DESIGN. CloseOutcome::Closed is success. RecvOutcome::Stopped is in the 27 and is **not a death** (trap-door 2).

---

## 1. The matrix

Arms = `grep -o VARIANT | wc -l` over `wat/ wat-scripts/ wat-tests/ tests/ src/` (occurrences, never `grep -c`). 542 `UNMIGRATED PLACEHOLDER` strings exist; they are not evidence a variant fires (trap-door 4).

Commands are from `wat-rs/` with `./target/release/wat`. Observation is the printed line.

### RecvOutcome (5)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Closed | 585 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-closed-lost.wat` | `recv-empty-child=Closed` |
| Lost | 645 | **FIRES** (scratch) | same probe | `recv-raise-child=Lost` |
| Malformed | 567 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-frame-cap.wat` | `a-big=Malformed` (service still serves `b-other=Message/Ok`) |
| Stopped | 562 | **FIRES** (scratch + SIGTERM) | park, then SIGTERM the reader — wrapper below | `parked-recv=Stopped` |
| TimedOut | 573 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-timedout.wat` | `recv-by-deadline=TimedOut` |

Recv Stopped wrapper (verbatim):

```
./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-stopped.wat > /tmp/cs-stopped.out 2>/tmp/cs-stopped.err &
PID=$!
for i in 1 2 3 4 5 6 7 8 9 10; do grep -q READY /tmp/cs-stopped.out 2>/dev/null && break; sleep 0.2; done
kill -TERM "$PID"; wait "$PID"; cat /tmp/cs-stopped.out
```

`recv-by-deadline` type-checks only on an owner handle (`Thread`/`Process`), not a client `Peer`. Client TimedOut is the generated-method map of `CallOutcome::DeadlineFired`. Both fire.

Trap-door 2: Recv Stopped means the **reader's** substrate was asked to stop. Nothing died. It is in the 27 because DESIGN counted it; it is not a crash.

### SendOutcome (3)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Closed | 217 | ⛔ **UNREACHABLE BY DESIGN** (arc 259 S2d, *"the user never holds the rope"* — a wall holding, NOT a gap; do not rank) | constructed only when the local send cell is already `None` (`runtime.rs` `None => send_outcome_closed()`). That is use-after-`close'`. `close'` is `:wat::kernel::`-restricted; tests cannot mint a kernel-namespace helper (`ReservedPrefix`). Peer **death** produces Lost, not Closed. | tried: process-exit send → `send-after-exit=Lost`; thread-exit send → `thread-send-after-exit=Lost`; clean `/stop` then send → `send-after-stop=Lost`; torn conn send → `send-torn=Lost` |
| Lost | 221 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-send-after-exit.wat` | `send-after-exit=Lost` |
| Stopped | 198 | **FIRES** (scratch + SIGTERM) | flood `send` against a silent peer, SIGTERM the sender | `Stopped-after=9260` |

Send Stopped wrapper: same shape as Recv Stopped, file `probe-crash-surface-send-stopped.wat`. Sleep ~1 s after READY so the flood is in `send`, then `kill -TERM`.

### TrySendOutcome (3)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Closed | 5 | ⛔ **UNREACHABLE BY DESIGN** (arc 259 S2d — same wall; do not rank) | same local-cell-`None` constructor as Send Closed (`try_send_outcome_closed`). Death → Lost. | `try-send-after-stop=Lost` |
| Lost | 5 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-die-then.wat` | `try-send-after=Lost` |
| WouldBlock | 6 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-try-send-wouldblock.wat` | `WouldBlock-after=377` |

### ConnectOutcome (3)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Failed | 296 | **UNREACHABLE** | constructor is `peer_cred` read / socket-wrap io error (`address.rs connect_as_value`). No wat knob faults those syscalls. | — |
| Refused | 299 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-connect-refused.wat` | `connect-after-stop=Refused` |
| Rejected | 295 | **UNREACHABLE** | `OnlyThisPeer`: answering pid ≠ address minter pid. Autobind stamps `getpid()`; a normal client dial never mismatches. No wat command to dial an address as a different identity. | — |

### AcceptOutcome (2)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Closed | 12 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-accept-closed.wat` | `accept-no-dial=Closed` (drop `Bound` — and its address — **before** `accept`) |
| Failed | 12 | **UNREACHABLE** | dropping the address produces Closed, not Failed. No injector for an accept(2) io error. | keeping Bound alive and not dialing **hangs**; `timeout` without `-k` never returns (SIGTERM does not unblock accept) |

### CloseOutcome (2) — success Closed excluded

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Failed | 3 | **FIRES** (scratch) | observability via `:wat::kernel::lineage-status` (the-rope-can-be-looked-at). SIGSTOP → `Some(Failed …)` stopped-not-terminated; not Signaled. `close'` stays restricted. | `probe-lineage-status.wat` `stopped=Some(Failed …)` |
| Signaled | 5 | **FIRES** (scratch) | same peek. Kill then `lineage-status` → `Some(Signaled 9)`. Does not consume; `close'` stays restricted. Send Closed / TrySend Closed remain **UNREACHABLE BY DESIGN** (arc 259 S2d). | `probe-lineage-status.wat` `signaled=Some(Signaled 9)` |

### SignalOutcome (1)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Failed | 13 | **UNREACHABLE** | `pidfd_send_signal` errno other than EINVAL/EBADF (those raise). Already-closed raises before the syscall. A zombie returns `Delivered` (measured, types.rs SignalOutcome comment STOP-2). No wat knob produces a handleable errno. | — |

### GateOutcome (2)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| GaveUp | 9 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-gate-gaveup.wat` (~10 s) | `grant-stuck-handler=GaveUp waited=10000 last=TimedOut` |
| Gone | 8 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-grant-after-die.wat` | `grant-after-die=Gone` |

Gate GaveUp needs the serve loop **already parked in a handler** before grant. Admin is a separate selectable: send-then-grant races and returns Applied. The probe `call-by-deadline`s ping (300 ms, handler on a 1-hour timer) first.

### CallOutcome (4)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Lost | 10 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-call-lost-other-conn.wat` | `call-other-conn-after-die=Lost` |
| Closed | 8 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-die-then.wat` | `call-after=Closed` (second touch after death) |
| DeadlineFired | 13 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-call-deadline.wat` | `call-by-deadline=DeadlineFired` |
| Malformed | 9 | **FIRES** (scratch) | frame-cap probe | `call-big=Malformed` |

Empty CallOutcome variants are Tagged `[]`, not Unit: match `(CallOutcome::DeadlineFired)`, not a bare keyword. Same as GateOutcome::Applied.

### StopOutcome (2)

| variant | arms | status | command / missing mechanism | observed |
|---|---|---|---|---|
| Gone | 12 | **FIRES** (scratch) | die-then probe | `stop-after=Gone` |
| GaveUp | 11 | **FIRES** (scratch) | `./target/release/wat wat-scripts/scratch-pad/probe-silent-stop-gaveup.wat` | `#wat.service.StopOutcome/GaveUp [500 "TimedOut"]` |

`Handle/handle` type-checks as `(Peer :- [Admin Status])`, not `Process`. `(:wat::kernel::signal)` therefore refuses it. Death was provoked by `raise!` inside ping, not SIGKILL on the lineage.

---

## 2. Counts

| | n |
|---|---|
| failure variants | **27** / **10** enums |
| FIRES | **16** |
| UNREACHABLE | **11** |
| NOT SWEPT | **0** (complete) |

Priority four (Recv / Send / Call / Stop): 5+3+4+2 = 14 cells. 13 FIRES, 1 UNREACHABLE (Send Closed).

---

## 3. disrupt's reach

`circuit.wat:379–392` — `disrupt-hits` is "the poisoned call came back lost/closed". The poison is an oversized `Seen/check` (`circuit.wat:533`, pad × 200) against `:max-frame-bytes 256`.

The same oversized ping, today:

```
a-small=Message/Ok;a-big=Malformed;b-other=Message/Ok;a-again=Closed;send-torn=Lost;call-torn=Closed;call-big=Malformed
```

So the poison is **RecvOutcome::Malformed** (and CallOutcome::Malformed), not Lost. The worker's Malformed arm is the unmigrated placeholder (`assertion-failed!`). Closed is the *second* touch on the torn handle; `-disrupt` redials on tore?, so it would not see Closed even if it survived.

Measured:

| command | result |
|---|---|
| `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | `disrupts=0;disrupt-fires=0;disrupt-draws=3;bp-disrupt=500;inbox-lost=0;inbox-closed=0;inbox-timedout=0` — poison never sent (3 rolls, 5 % rate) |
| `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 10000` | **filled-stalled** `arrived=0` polls=601 — 100 % disrupt, fill never moves |
| `20 1 1 … 10000` | same stall (topology also degenerate) |

**disrupt-bp does not currently provoke Lost/Closed as a counted hit.** At a rate that actually fires, the worker dies on the Malformed placeholder and the circuit cannot fill. It is not a store-fault injector either (Seen, not Store).

---

## 4. §2d store-fault cell

The unknowable write. Live arms (line numbers **now**, DESIGN's 745/779/810 / 1208/1241/1272 have drifted):

| op | Lost | Closed | TimedOut |
|---|---|---|---|
| `queue.send` store put | `sqs.wat:780` | `:814` | `:845` |
| `queue.ack` store delete | `:1215` | `:1248` | `:1279` |

`drop-recv-bp` / `drop-ack-bp` (`sqs.wat:202–203`) suppress the queue's **reply to its caller** (`:wat::core::None` instead of `Some Reply`). They do not fail a store call.

Store is `(:wat::spawn::process)` sqlite-store (`circuit.wat:2930`). Nothing in circuit argv signals, kills, or oversizes the store.

Chaos just run: `inbox-lost=0;inbox-closed=0;inbox-timedout=0` (and DESIGN's healthy+chaos zeros still hold).

**UNREACHABLE from the harness:** no fault can be injected into a store call. The queue's only knobs suppress its own reply to its caller, and the store process is never signalled. The six arms are live code that has never executed.

Those Recv Lost/Closed/TimedOut variants **do** fire on other peers (this matrix). What has never fired is **those variants on a store call from the queue**.

---

## 5. Ranked injector list

Ranked by cells-bought-per-work, with the campaign blocker called out.

1. **⭑ Store-fault injector** — signal / kill / oversize the sqlite-store process (or a store-shaped peer the queue already holds) so `queue.send`/`queue.ack` observe Lost/Closed/TimedOut. **Does not buy new outcome variants** (those already FIRES). **Buys the §2d path**: the six arms, the unknowable write, and the measurement the A/B/C ruling needs. Blocks the banked −17.1 % `rt-store` / −60.9 % `count` patch. Copy `drop-recv-bp`'s shape: a rate, a fire counter at the suppression site, not at the dice roll.
2. **Fix disrupt's Malformed arm** — one arm: stop `assertion-failed!`ing RecvOutcome::Malformed, count it, redial. Makes `disrupt-bp` a live chaos injector again. At 100 % it currently stalls fill. Does not buy a new variant (frame-cap already fires Malformed). Buys harness-level exercise of the poison that already exists.
3. ⛔ **CORRECTED 2026-09-13 — this was ONE item and it is TWO; only half is a stone.** *Original text:*
   *"Sanctioned wat-level `close'` — unblocks Send Closed, TrySend Closed, CloseOutcome::Signaled,
   CloseOutcome::Failed. Four cells. The Kill fixture already named this wall (`ReservedPrefix` + kernel
   restriction)."* There are **two** walls, and they split the cells:
   - **WALL 1 (type)** — `:wat::kernel::close` takes `(Thread | Process)`, a **lineage**, never a dialed
     `Peer`. Probed: passing a `Peer` is a `TypeMismatch`. So these cells were never behind one door.
   - **WALL 2 (ruling)** — `close` is `:restricted-to [:wat::kernel::]` by **arc 259 S2d, "the user never
     holds the rope"** (`tests/process/signal_kill_produces_close_outcome_signaled.wat:11–24`, which
     records this entire crawl already, including the `ReservedPrefix` dead end *"verified empirically"*).
   - ✅ `CloseOutcome::Signaled` + `::Failed` are an **observability** gap → `the-rope-can-be-looked-at/`
     (a non-consuming `lineage-status` peek; reverses no ruling).
   - ⛔ `SendOutcome::Closed` + `TrySendOutcome::Closed` are **RETIRED — UNREACHABLE BY DESIGN.** They need
     *use-after-close*, i.e. userland consuming a lineage and then touching it, which the arc-259 ruling
     makes **unwritable in userland**. ★★ That is a wall working, not a debt: a cell empty because the
     mistake has no form is the top of the extirpare ladder. Reaching them would mean reversing arc 259 to
     improve a coverage number.

   ⭑⭑ **AND THE TAXONOMY OF THIS WHOLE MATRIX NEEDS A SPLIT, which matters more than these two cells:**
   `UNREACHABLE` must distinguish **"no mechanism yet"** (a gap, worth ranking) from **"no mechanism by
   ruling"** (a wall holding, worth recording and closing). This FINDING recorded both as one status, which
   is how a working wall came to sit on a to-do list **ranked above real work**. Anyone re-running the sweep
   should classify all 11 `UNREACHABLE` cells on that axis before ranking anything.
4. **Connect Rejected** — dial an address whose answering pid ≠ minter. One cell. Needs a forged or cross-process identity, not a new outcome type.
5. **Signal Failed** — a `pidfd_send_signal` errno that is not EINVAL/EBADF and not already-closed. One cell. Zombie is not it (`Delivered`).
6. **Accept Failed / Connect Failed** — accept(2) / `peer_cred` io errors. Two cells. No wat handle on those syscalls today.

Do **not** mint `:Unknown` for §2d until (1) exists. That would be a painted brick (arc 109 NOTE).

---

## 6. Probes (scratch-pad, type-check)

| file | cells |
|---|---|
| `probe-crash-surface-recv-closed-lost.wat` | Recv Closed, Recv Lost |
| `probe-crash-surface-recv-timedout.wat` | Recv TimedOut |
| `probe-crash-surface-recv-stopped.wat` | Recv Stopped (needs SIGTERM) |
| `probe-crash-surface-frame-cap.wat` | Recv Malformed, Recv Closed (torn), Send Lost (torn), Call Closed (torn), Call Malformed |
| `probe-crash-surface-call-deadline.wat` | Call DeadlineFired |
| `probe-crash-surface-call-lost-other-conn.wat` | Call Lost |
| `probe-crash-surface-die-then.wat` | Recv Lost, Call Closed, Send Lost, TrySend Lost, Stop Gone |
| `probe-crash-surface-send-after-exit.wat` | Send Lost, Recv Closed |
| `probe-crash-surface-send-stopped.wat` | Send Stopped (needs SIGTERM) |
| `probe-crash-surface-try-send-wouldblock.wat` | TrySend WouldBlock |
| `probe-crash-surface-try-send-after-stop.wat` | TrySend Closed did **not** fire (Lost) |
| `probe-crash-surface-send-closed-thread.wat` | Send Closed did **not** fire (Lost) |
| `probe-crash-surface-connect-refused.wat` | Connect Refused |
| `probe-crash-surface-accept-closed.wat` | Accept Closed |
| `probe-crash-surface-gate-gaveup.wat` | Gate GaveUp (~10 s) |
| `probe-crash-surface-grant-after-die.wat` | Gate Gone |
| `probe-silent-stop-gaveup.wat` | Stop GaveUp (pre-existing) |
