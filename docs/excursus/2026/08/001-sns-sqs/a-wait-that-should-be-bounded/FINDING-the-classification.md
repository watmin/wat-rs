# FINDING — the classification (a wait that SHOULD be bounded)

**Struck 2026-09-15 by a spawned Opus subagent. REPORT-ONLY — no `.wat` or `.rs` was touched.**

Every row below cites a `file:line` that was opened and read in context. Enclosing-form line numbers
are given so a re-read lands in the same place.

---

## 0. ⚠ The finding that comes before the classification: the brief's own grep sees half the sites

The pattern in `BRIEF.md` / the prompt —

```bash
grep -rn '(:wat::kernel::recv ' --include=*.wat wat/ wat-scripts/queue/sqs.wat \
    wat-scripts/topic/sns-fanout.wat wat-scripts/fanout/circuit.wat
```

— returns **12** sites. The trailing space that (correctly) excludes `recv-by-deadline` also excludes
**every site whose peer expression starts on the next line**, which is the dominant shape for the timer
recvs:

```
wat-scripts/fanout/circuit.wat:1210     (:wat::kernel::recv
wat-scripts/fanout/circuit.wat:1211       (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
```

⛔ Run that way, **control 1 would have been reported ABSENT** — `:fanout::await-timer-ms`,
`:user::await-timer-ms` and `:demo::await-timer-ms` are all newline-wrapped, and so are both `nap`
closures in `sqs.wat` and all five `await-ms`/`_nap` closures in `circuit.wat`. The pattern actually
used here, which keeps `recv-by-deadline` and `recv-all*` out by excluding a following letter or dash:

```bash
grep -rnE ':wat::kernel::recv([^a-zA-Z-]|$)' --include=*.wat wat/ wat-scripts/queue/sqs.wat \
    wat-scripts/topic/sns-fanout.wat wat-scripts/fanout/circuit.wat
```

→ **23 live sites.** Verified against `grep -rnoE '\(:wat::kernel::recv([^a-zA-Z-]|$)'` (call sites
only, `(` immediately before the head): 23 live, 313 corpus-wide.

---

## 1. The three controls

| # | control | verdict | PASS/FAIL |
|---|---|---|---|
| 1 | `:fanout::await-timer-ms` + twins | **TIMER** | ✅ **PASS** |
| 2 | `:user::park-receive!` | **PARK** | ✅ **PASS** — *with an amendment, §7* |
| 3 | `wat/service.wat` generated `child-main` | **ABSENT** from the bare-recv list | ✅ **PASS** |

**Control 1 — PASS.** All three twins found and read; each recvs on an `after` timer channel.

- `wat-scripts/fanout/circuit.wat:1208` `(defn :fanout::await-timer-ms [ms])` → recv at **1210**, peer
  `(:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done)` (1211).
  Header comment at 1207: *"Timer-channel recv, not a sleep — legal where mora forbids sleeping."*
- `wat-scripts/queue/sqs.wat:2088` `(defn :user::await-timer-ms …)` → recv at **2090**, same peer shape.
- `wat-scripts/topic/sns-fanout.wat:674` `(defn :demo::await-timer-ms …)` → recv at **676**, same shape.

**Control 2 — PASS** (`:user::park-receive!`, `wat-scripts/queue/sqs.wat:2053`, its bare recv at
**2065**) → **PARK**. It is the long-poll driver: it ships
`(:queue::Queue::Op::Receive … :wait wait)` with a `:queue::Queue::Wait` (2059–2063) and the site is
not placed on the BOUND list. ⚠ **But the code does not say what the control's rationale says it
says** — read §7 before any later stone acts on this. The classification stands; the *reason* needs the
builder's ruling.

**Control 3 — PASS.** `wat/service.wat` has **zero** bare `:wat::kernel::recv` call sites. The generated
`child-main` (`child-main-form`, `wat/service.wat:3489`) awaits the owner's startup ship at **3510** via
`(:wat::kernel::recv-by-deadline ~cm-self-sym :wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS)`, and the
only other `recv-by-deadline` in the file is at 4100. The trailer comment at 4254 states it outright:
*"Generated child-main awaits the owner's startup ship with recv-by-deadline."* Current code was read;
nothing stale.

---

## 2. Every live site

`enc` = enclosing form, with its own line number.

| # | file:line | enclosing defn | waiting for | class | reason |
|---|---|---|---|---|---|
| 1 | `wat/spawn.wat:531` | `extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus` (487), `launch` impl (488) | the thread child's **readiness announcement** (the `lu-mk-kw` ship carrying the bound address) | **BOUND** | One-time startup handshake. Comment at 527 calls it a *"Crash-aware readiness barrier"*; it faces Lost/Stopped/Closed but a child that is **alive and silent** never returns — the `TimedOut` arm is an `assertion-failed!` placeholder that a bare recv can never reach. |
| 2 | `wat/spawn.wat:588` | `extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus` (559), `launch` impl (567) | the process child's **launch status** carrying the child-minted `Address` (capability handoff) | **BOUND** | Same one-time handshake one tier down, after `send svc ship`. A slow/silent child hangs `launch` forever; this is the exact twin of the `child-main` defect already fixed with `recv-by-deadline`. |
| 3 | `wat/spawn.wat:619` | `defn :wat::kernel::recv-all-loop` (615) | the **next value, or a terminal outcome**, from a peer being drained | **UNKNOWN** | See §6. |
| 4 | `wat/test.wat:329` | `defn :wat::test::spawn-thread-program` (325) | the test child's **completion signal** | **BOUND** | Spawn → await one specific reply. A wedged test child (alive, silent, never signals) hangs the harness with no arm to report it; `Lost`/`Closed`/`Stopped` are all handled, only the silent case is not. |
| 5 | `wat/test.wat:438` | `defn :wat::test::spawn-hermetic-program` (434) | the process test child's **pass-marker** (`println 0` on fd 1) | **BOUND** | Process-tier twin of row 4, same shape, same silent-peer hole. |
| 6 | `wat/bracket.wat:39` | `defn :wat::bracket::runner-loop` (32) | the **next work item** from the parent pool | **PARK** | Worker serve loop. File header (14–16): *"recv' raises … when the parent's Thread is dropped … No explicit termination condition is needed; the channel drain IS the signal."* Blocking between items is the loop. |
| 7 | `wat/bracket.wat:78` | `defn :wat::bracket::process-runner` (71) | the **next `PoolMsg`** (`Work` or `Setup`) | **PARK** | Baked process-pool worker loop, *"tail-recursing forever"* (63). Idle = waiting for work. |
| 8 | `wat/bracket.wat:121` | `defn :wat::bracket::process-dial-runner` (114) | the **next `PoolMsg`** (dialing cousin, holds a peer across items) | **PARK** | Same worker loop; `Setup` dials, `Work` runs, both recurse back into this recv. |
| 9 | `wat/bracket.wat:201` | `defn :wat::bracket::thread-kwargs-runner` (190) | the **next `PoolMsg`** | **PARK** | Same worker loop, kwargs tier. |
| 10 | `wat/bracket.wat:484` | `defclause :wat::bracket::process-work-forms` (363) — inside the quasiquoted `runner-def`, generating `:user::bracket::dial-runner` | the **next `PoolMsg`** in the *generated* n-dial runner | **PARK** | One source site that expands per pool; body is the same worker loop as rows 7–9 (`Setup` → assemble+hold, `Work` → `$impl` → send → recurse). |
| 11 | `wat-scripts/topic/sns-fanout.wat:113` | `defsurface :demo::Topic` (45), `publish` impl (96) | an **`after` timer** (the inlined chaos `delay-ms` park) | **TIMER** | Peer is `(after thread (Milliseconds delay-ms) :done)` (114–115). Comment at 109: *"INLINED timer park — mora-legal; a forked child cannot see parent helpers."* |
| 12 | `wat-scripts/topic/sns-fanout.wat:676` | `defn :demo::await-timer-ms` (674) | an **`after` timer** | **TIMER** | Control-1 twin. |
| 13 | `wat-scripts/queue/sqs.wat:1623` | `defn :queue::queue::retry-put` (1616), `nap` closure (1621) | an **`after` timer**, 1 ms | **TIMER** | Peer is `(after thread (Milliseconds 1) :done)` (1624–1625) — the retry backoff. |
| 14 | `wat-scripts/queue/sqs.wat:1679` | `defn :queue::queue::retry-delete` (1672), `nap` closure (1677) | an **`after` timer**, 1 ms | **TIMER** | Same backoff nap, delete path. |
| 15 | `wat-scripts/queue/sqs.wat:2065` | `defn :user::park-receive!` (2053) | the **`Reply::Stats` barrier** that confirms the parked `Receive` is lodged | **PARK** | Control 2. The enclosing fn is the long-poll driver (`:wait wait`, 2062); blocking here is inside the long-poll protocol. ⚠ **amendment in §7** — the code shows this particular recv awaiting an *immediate* Stats reply, not the poll. |
| 16 | `wat-scripts/queue/sqs.wat:2075` | `defn :user::recv-envelopes!` (2073) | the **parked long-poll `Reply::Receive`** | **PARK** | This is where the `Wait::UpTo` actually elapses: the service parks a `Waiter` and answers later (`sqs.wat:1110`–`1160`), or the sweeper answers `empty-ok` at the deadline (`1496`–`1500`). Bounding this **is** the thing that would undo `queue-long-poll`. |
| 17 | `wat-scripts/queue/sqs.wat:2090` | `defn :user::await-timer-ms` (2088) | an **`after` timer** | **TIMER** | Control-1 twin. |
| 18 | `wat-scripts/fanout/circuit.wat:656` | `defsurface :fanout::Worker` (302), `-tick` self-impl (599), `await-ms` closure (653) | an **`after` timer** | **TIMER** | Retry-spacing nap before `call-by-deadline` on the seen service. |
| 19 | `wat-scripts/fanout/circuit.wat:785` | `defsurface :fanout::Worker` (302), `-tick` (599), `_work-nap` binding (783) | an **`after` timer**, `work-delay` ms | **TIMER** | Injected per-item work delay. |
| 20 | `wat-scripts/fanout/circuit.wat:812` | `defsurface :fanout::Worker` (302), `-tick` (599), `_nap` binding (810) | an **`after` timer**, `ack-delay` ms | **TIMER** | Injected pre-ack delay. |
| 21 | `wat-scripts/fanout/circuit.wat:826` | `defsurface :fanout::Worker` (302), `-tick` (599), second `await-ms` closure (823) | an **`after` timer** | **TIMER** | Ack-path retry spacing; a distinct closure from row 18. |
| 22 | `wat-scripts/fanout/circuit.wat:1210` | `defn :fanout::await-timer-ms` (1208) | an **`after` timer** | **TIMER** | Control 1 proper. |
| 23 | `wat-scripts/fanout/circuit.wat:2168` | `defsurface :fanout::Publisher` (2039), `-run` self-impl (2128), `await-ms` closure (2165) | an **`after` timer** | **TIMER** | Publisher batch pacing. |

### ⭐ The four rows that shared one name

Rows 18–21 are the census's *four rows with `enc=:fanout::worker`*. They are **four genuinely distinct
recv sites**, not one counted four times — two separate `await-ms` closures (653, 823) plus two inlined
chaos naps (783, 810), all inside the one `-tick` impl at 599. **All four are TIMER.** The census row was
a pointer and the pointer was honest; what it could not say is that none of the four is a defect.

---

## 3. Totals

| class | count |
|---|---|
| **TIMER** | **11** |
| **PARK** | **7** |
| **BOUND** | **4** |
| **UNKNOWN** | **1** |
| **live total** | **23** |

11 + 7 + 4 + 1 = 23. ✅

### ⚠ The live count is 23, not 24 (STOP-4)

Re-derived from the code. The orchestrator's 24 came from census rows; the code says **23**. I cannot
reconstruct which row was the twenty-fourth — the census ran on an older revision (its
`park-receive!` rows sit at `sqs.wat:2034–2040`, now `2053–2065`, so `sqs.wat` has since moved ~25
lines), and at least one census row in that file is `primitive=send`, not `recv`. The difference is one
site and it is **not** material to the BOUND list: every site in `wat/service.wat` is already bounded,
and the 11 TIMER / 7 PARK sites are non-defects either way.

Corpus-wide, for the ratio: **313** bare-`recv` call sites total, **23 live**, **290 non-live**. (The
earlier census figure of 259 is a different instrument — form-tree walk with a `bounded=` verdict, not a
grep — and is not contradicted here; it is simply not the same count.)

---

## 4. ⛔ The BOUND list — the defect list this stone exists to produce

**Four sites. All four are the same defect: a one-time startup/completion handshake with a freshly
spawned child, where every terminal outcome is faced except "the peer is alive and has not answered".**

| # | site | enclosing | the hang |
|---|---|---|---|
| B1 | `wat/spawn.wat:531` | `ThreadOpts` `launch` (487/488) | a thread child that reaches neither crash nor readiness hangs the launcher forever |
| B2 | `wat/spawn.wat:588` | `ProcessOpts` `launch` (559/567) | a process child that never ships its minted `Address` hangs the launcher forever |
| B3 | `wat/test.wat:329` | `:wat::test::spawn-thread-program` (325) | a wedged thread test hangs the harness with no timeout and no reportable arm |
| B4 | `wat/test.wat:438` | `:wat::test::spawn-hermetic-program` (434) | a wedged hermetic (process) test hangs the harness identically |

⭐ **B1 and B2 are the sibling of the already-fixed control-3 defect.** `child-main` (the *child* side of
the very same handshake) was bounded with `recv-by-deadline` + `CHILD-MAIN-STARTUP-DEADLINE-MS`. The
**parent** side of that handshake — `launch` — was not. The fix bounded one end of one wire.

⛔ **All four carry a dead arm that proves it.** Each match already has
`(:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is
alive and silent" …))`. On a **bare** recv that arm is unreachable by construction
(`docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md:43`). The code
already names the failure it cannot observe.

---

## 5. Scope

A classification was produced. **No wait was bounded; nothing was edited.** Non-live sites were counted,
not classified.

---

## 6. The UNKNOWN — one site

**`wat/spawn.wat:619` — `:wat::kernel::recv-all-loop` (615).** Waiting for: the next value, or a
terminal outcome, from a peer being drained.

What defeated the classifier, specifically:

1. **It is neither of the two PARK shapes and neither of the two BOUND shapes.** Not a long-poll; not a
   service main loop awaiting its next request; not a handshake; not a request/reply. It is a
   *drain-to-EOF* — a fifth shape the four classes do not name.
2. **No live caller reveals intent.** `grep -rn 'kernel::recv-all\b'` over `wat/` + `wat-scripts/`
   returns only the rename records in `wat-scripts/fixes/reclaim-ipc-prime-names.wat:51-52`. Its
   documented purpose (spawn.wat:600–614) is to replace a retired *test* drain, and the only readings
   of intent available are from its own header prose.
3. **Its contract is compatible with both readings.** *"Reads until the peer signals a terminal
   RecvOutcome"* (604–605) is a PARK reading — the producer's termination is the signal, as in
   `runner-loop`. But a peer that is **alive and silent** hangs the drain forever, which is the BOUND
   reading, and its `TimedOut` arm is the same unreachable `assertion-failed!` placeholder as the four
   BOUND sites.
4. **Bounding it would change its return contract**, not just add a deadline: it would have to
   distinguish "drained" from "timed out holding a partial `acc`" — and the file's own comment at
   628 is an argument that returning `Ok` over a truncated read *"is the same lie in a different
   coat."* Deciding that is a ruling about this API's semantics, not a reading of the code.

Per **STOP-2**, UNKNOWN. It is **not** folded into PARK and **not** placed on the BOUND list.

---

## 7. ⚠ Amendment the builder must rule on — control 2's rationale does not survive reading the body

**The classification (PARK) stands. The stated reason does not, and a later stone reading the reason
instead of the class would aim at the wrong line.**

`DESIGN.md:28` and `BRIEF.md:22` justify PARK as: *"`:user::park-receive!` takes a
`:queue::Queue::Wait` parameter — it is the long-poll, where blocking IS the feature."* Reading the
body:

- `park-receive!` sends **`Op::Receive` with `:wait wait`** (2059–2063), then immediately sends
  **`Op::Stats`** (2063–2064), then recvs **once** (2065) and asserts the reply is
  `Reply::Stats` — *"park-receive: expected Stats reply as barrier"* (2069). The recv's own comment
  and assertion text both say **barrier**, not poll.
- The service answers `Wait::UpTo` by conj'ing a `Waiter` into state and returning
  `(:wat::service::Outcome::Continue s-a :wat::core::None …)` — **no reply** (`sqs.wat:1110`–`1160`,
  the `None` at 1160). It keeps serving; the `Stats` handler answers **unconditionally and immediately**
  (`Some (Reply::Stats …)`, `sqs.wat:1435`).
- Therefore the wait at **2065 is expected to return promptly**. The wait that actually holds for the
  `Wait::UpTo` duration is **2075** in `:user::recv-envelopes!` (row 16) — reached later, after the
  caller has done the thing that wakes the park.

Consequences, stated plainly:

- **The site whose bounding would break `queue-long-poll` is `sqs.wat:2075`, not `sqs.wat:2065`.** If
  trap-door 1 is protecting a line, it is that one.
- A generous deadline on 2065 would not affect the long-poll; on the evidence above it is a
  request/reply barrier that hangs on a silent queue service — i.e. **BOUND-shaped**.
- I did **not** move it to BOUND. The control is the builder's established answer, the harm of a
  wrong BOUND is asymmetric, and one of us is reading `queue-long-poll`'s intent wrong. That is a
  ruling, not a read.

**Ask:** rule on `sqs.wat:2065` — PARK (as classified, control honoured) or BOUND (as the barrier
semantics read). Nothing else in this finding depends on the answer.

---

## 8. The non-live count

**290.**
