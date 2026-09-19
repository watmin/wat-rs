# SCORE — a peer that is merely slow

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `79d757e5a` (DRAWN). Did not commit.

Sentence: **Induce a peer that is SLOW. Today every injector makes one that is DEAD or SILENT.**

---

## Row 1 — the knob

`delay-bp` + `delay-ms` + `delays-fired` on `:query::faulting-store::Record`, siblings of `drop-reply-bp` / `die-bp`. Defaults **0: OFF**. Existing Record sites freeze them at 0.

Priority: die > drop > delay. Delay only on the reply path (forward first, wait second). `delays-fired` increments **after** `nap` returns, never at the dice roll.

---

## Row 2 — the wait is a timer channel

`:query::faulting-store::nap` is `recv` of `(:wat::kernel::after :wat::program::PeerKind::thread (Milliseconds delay-ms) :done)`. Exemplar `wat/queue.wat:1643`. No sleep intrinsic was added. `grep wat_intrinsic sleep` is still only DESIGN prose.

---

## Row 3 — the surplus-ack desync, attempted

**Grant / AllowPeer is not reachable by this injector.** `grant` sends `Admin::AllowPeer` on the lineage handle (`wat/service.wat:3443`–`:3476`) and re-recvs via `owner-recv-loop`. The delay sits on `Store::put` / `Store::delete` Reply after the write lands. There is no call from grant to the store. That is the FINDING's hazard, and this knob cannot construct it.

**The store-peer analogue is constructible, and the re-recv ruling HOLDS there.**

Isolated run of `:slow::run-desync`:

```
reask sent1=Sent;first=TimedOut;sent2=Sent;after-reask=Message;leftover=Message
rerecv sent1=Sent;first=TimedOut;drained=Message;sent2=Sent;own=Message;leftover=TimedOut
```

Re-ask after a 50 ms `recv-by-deadline` reads the abandoned first Success as if it belonged to the second send, and the second reply stays on the peer. Re-recv drains the late first reply; the next send gets its own; leftover is TimedOut. PutResponse::Success has no tag, so identity is the leftover cell, not a tag mismatch.

The gate-methods ruling (re-recv, not re-ask) is the right act on a peer that can carry a surplus frame. It remains untested on the lineage Admin ack, because this injector cannot make that ack slow.

---

## Row 4 — one deadline fired by LATENESS

**`recv-by-deadline` 50 ms.** Not `call-by-deadline`: that verb re-establishes and abandons the late reply (stone `a-fired-deadline-hands-back-a-live-peer`), so a second recv on the same binding cannot observe lateness.

Isolated `:slow::run-late-vs-absent`:

```
late   sent=Sent;first=TimedOut;second=Message;delays-fired=1
absent sent=Sent;first=TimedOut;second=TimedOut;delays-fired=0
```

Same first window, both TimedOut. Second window: the delayed put arrives (`Message`); the drop-reply put does not (`TimedOut`). Shown, not asserted as prose.

---

## Row 5 — non-vacuity

Isolated `:slow::run-rates` (`delay-ms=200`):

```
disarmed put=Success;elapsed-ms=0;delays-fired=0
armed    put=Success;elapsed-ms=201;delays-fired=1
```

`delay-bp=0` does not fire. `delay-bp=10000` fires at the wait site and takes **at least** 200 ms (201). A test that passed at both rates would have measured nothing; these differ.

---

## Row 6 — scope wall

Not done, each named:

- Bracket PEER-path injectors. `drop-recv-bp` / `drop-ack-bp` stay in `wat/queue.wat`. No delay on a runner peer.
- The thread-tier chaos harness. Circuit spawn counts untouched (`spawn::process = 8`, `spawn::thread = 0`). These probes start faulting-store on a thread because that is how the existing store injector is driven; the delay is a handler park, not a transport fault.
- Backpressure, a partition that heals, half-close, byte corruption distinct from oversize.

Reordering is still empty by `SOCK_STREAM` physics.

---

## Row 7 — floor

`git diff HEAD` was **not empty** (did not commit, as briefed). Floor ran on HEAD `79d757e5a` plus this uncommitted tree:

```
 wat-scripts/query/faulting-store.wat               | 63 ++++++++++++++++++++--
 .../scratch-pad/probe-accepted-is-a-count.wat      |  5 +-
 .../scratch-pad/probe-store-can-fail-smoke.wat     |  2 +-
 wat-scripts/scratch-pad/probe-store-can-fail.wat   |  5 +-
 4 files changed, 67 insertions(+), 8 deletions(-)
```

Untracked: `wat-scripts/scratch-pad/probe-a-peer-that-is-merely-slow.wat`, `tests/services/probe_a_peer_that_is_merely_slow.rs`.

Quoted immediately before `scripts/floor.sh`. No concurrent circuit. No `ps` hit for floor/nextest/circuit.

```
     Summary [ 129.365s] 5329 tests run: 5329 passed, 22 skipped
```

`.floor/2026-09-19T10-05-29Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**. Count **5326 → 5329** (+3 tests). Not a shrink.

Clippy: **CLIPPY=0**. `cargo clippy --release --workspace --all-targets -- -D warnings` exit 0 (whole output: Finished, no warnings). First clippy pass reddened `needless_lifetimes` on `half`; elided; did not re-run that red as a green.

Floor times (nice -n 19): rates 0.627 s · late-vs-absent 1.204 s · desync 2.140 s.

---

## What landed

- `wat-scripts/query/faulting-store.wat` — nap via `after`; put/delete roll delay; `delays-fired` at the wait.
- Three existing Record constructors freeze `:delay-bp 0 :delay-ms 0 :delays-fired 0`.
- `wat-scripts/scratch-pad/probe-a-peer-that-is-merely-slow.wat` + `tests/services/probe_a_peer_that_is_merely_slow.rs`.

No `src/`. No `wat/` stdlib. `wat/core.wat` line count untouched.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | the knob | `delay-bp` + `delay-ms` + `delays-fired`; count at nap; defaults 0 |
| 2 | timer channel | `after` + `recv`; no sleep |
| 3 | surplus-ack attempted | grant Admin ack **not reachable**; store-peer re-ask leaves leftover Message; re-recv leftover TimedOut. Ruling holds on the store peer |
| 4 | deadline by lateness | `recv-by-deadline` 50 ms: late second=Message, absent second=TimedOut |
| 5 | non-vacuity | delays-fired 0 vs 1; armed elapsed-ms=201 ≥ 200 |
| 6 | scope wall | named, untouched |
| 7 | floor | Summary above; dirty tree quoted; CLIPPY=0 |

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted; row 3's negative is the finding

### ⭐ The instrument works, and it settles a contract decided without evidence

All three probes reproduce on my own run, verbatim:

```
disarmed put=Success;elapsed-ms=0;delays-fired=0
armed    put=Success;elapsed-ms=200;delays-fired=1
late     sent=Sent;first=TimedOut;second=Message;delays-fired=1
absent   sent=Sent;first=TimedOut;second=TimedOut;delays-fired=0
reask    sent1=Sent;first=TimedOut;sent2=Sent;after-reask=Message;leftover=Message
rerecv   sent1=Sent;first=TimedOut;drained=Message;sent2=Sent;own=Message;leftover=TimedOut
```

⭐ **`reask … leftover=Message` is the surplus frame**, constructed for the first time. The
`the-gate-methods-face-an-outcome` ruling — re-recv over re-ask — was decided from reasoning about a
state no instrument could produce. It can now be produced, and **the ruling holds**: re-ask reads the
abandoned first reply as if it answered the second send and leaves a frame on the wire; re-recv
drains the late one and the leftover is `TimedOut`. A contract argued into existence is now a
contract with evidence under it.

⚠ I got `elapsed-ms=200` where the SCORE says 201 — consistent, because the row was written as
**"at least N"**. That is the assertion shape EXPECTATIONS demanded and the reason the difference is
not a discrepancy.

### ⛔ ROW 3's NEGATIVE IS THE MORE IMPORTANT HALF, and it is correctly refused

> *"Grant / AllowPeer is not reachable by this injector."*

Verified: `wat/service.wat:3443`–`:3476` (`grant`) contains **zero** references to a `Store`. The
delay sits on `Store::put` / `Store::delete` replies, so there is no path from the lineage `Admin`
ack to a knob that can slow it. **The FINDING's own hazard is still not constructible** — what is
now constructible is a *store-peer analogue* of it, on a peer that can carry a surplus frame.

That distinction is stated rather than blurred, which is the whole difference between this and a
stone that claims its hazard and ships. ⭐ **The ruling is tested on the shape, not on the site.**
Slowing the lineage `Admin` ack is the next knob, and it is not this one.

### Verified

| claim | check |
|---|---|
| counter at the wait, not the dice roll | ✅ `delays-fired` incremented after `nap` returns (`:160`/`:173`, `:213`/`:226`); the file's own comment repeats the exemplar's rule twice |
| the wait is a timer channel | ✅ `nap` is `recv` of `(:wat::kernel::after … (Milliseconds delay-ms) :done)`; **no sleep intrinsic added** (still zero in the tree) |
| defaults OFF | ✅ `delay-bp 0` — `disarmed` fires 0 and costs 0 ms |
| floor | ✅ artifact `.floor/2026-09-19T10-05-29Z/`, no `ARM.txt`, `5329 passed`. **My own independent floor: `Summary [ 122.032s] 5329 tests run: 5329 passed, 22 skipped`** (`.floor/2026-09-19T10-11-16Z/`), index verified empty beforehand |
| no `src/`, no stdlib | ✅ |
| clippy | ✅ 0, and the first pass's `needless_lifetimes` red is disclosed rather than re-run into a green |

### ⚠ Row 7, adapted rather than met, and correctly so

My row 7 said "verify `git diff HEAD` is empty before the floor". grok does not commit, so it
**quoted the dirty tree instead** — the four modified files and two untracked, immediately before
`scripts/floor.sh`, plus a check for concurrent circuit/nextest processes. That is the honest
discharge of the *intent* (a floor whose subject is stated) rather than the letter, and it is better
than what I asked for: my rule was written for my own failure mode, which was committing a tree I
had not floored.
