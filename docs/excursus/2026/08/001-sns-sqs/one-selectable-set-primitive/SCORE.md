# SCORE — one selectable-set primitive

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `338800365`. Did not commit.

The previous SCORE on this path left two Select constructors and called that "set membership." That failed row 2. This strike makes `select(peers)` the peers-only call of `fan_in_unified_peer_set`. `poll` is the same function with self and listener supplied.

Sentence: **`select(peers)` is `poll(∅, ∅, peers)`. There is one waiting discipline, not two.**

---

## Row 1 — the drift table

`SelectOutcome` (`src/comms/mod.rs:507`) is `Recv { index, result }` · `Shutdown` · `Listener`. Select has three peer representations; poll has one.

| kind | type_path | crash channel | who uses it |
|---|---|---|---|
| spawned Thread | `THREAD_PEER_TYPE_PATH` | yes | **select only** |
| spawned Process | `PROCESS_PEER_TYPE_PATH` | yes (err rx) | **select only** |
| unified Peer | `PEER_TYPE_PATH` | no | select **and** poll — **the shared engine** |

### Unified Peer (the overlapping set) — NOW one function

| SelectOutcome | `fan_in_unified_peer_set` |
|---|---|
| Recv Ok, role = self | **Admin** (only if self was registered) |
| Recv Err, role = self | **Shutdown** + `broadcast_peer_severed` |
| Recv Ok, role = listener-recv (thread) | **Connection** via `wrap_connect_request` |
| Recv Ok, role = client, thread | death-notice → Lost; Failed → Malformed; else Message |
| Recv Err, role = client, thread | **Closed** |
| Recv Ok, role = client, process | `classify_trusted_wire_recv` → Lost / Malformed / Message. UTF-8 fail → **Malformed** |
| Recv Err FrameTooLarge, client, process | **Rejected** |
| Recv Err other, client, process | **Closed** |
| Shutdown | **ServiceEvent::Shutdown** (was: poll **raised**, select returned the value — unified to the value) |
| Listener, listener in set | **Connection** (process accept-arm) |
| Listener, listener **not** in set | **error** `"listener arm fired but no listener was in the set"` — not `unreachable!()`, not a silent default |
| io_uring `Err` | **Lost** idx=0 `"select io_uring error"` (was: poll raised) |

### Spawn Thread / Process (select only — poll never receives these types)

Unchanged constructors: crash-channel Recv Err still `classify_peer_death` / `classify_peer_error`. Process spawn Recv Ok still `classify_trusted_wire_recv` (decode → Malformed). These are extra **input types**, not a second waiting discipline.

⭐ Cells that were not the known decode row:

1. Recv Err + crash channel vs always Closed — **peer-kind**, not verb. Poll never sees a spawned Thread/Process.
2. SelectOutcome::Shutdown: poll raised, select returned a value. **Unified to the value.** Serve loop already matches Shutdown.
3. FrameTooLarge: poll Rejected; select bare was Closed. **Unified to Rejected** on the shared engine.
4. io_uring: poll raised; select Lost. **Unified to Lost** on the shared engine.
5. Thread poll Recv Ok had no death-notice check. Shared engine **has** it.

None of these is "the service needs behaviour a pool must not have." Two impls are **not** warranted.

---

## Row 2 — one engine

`fan_in_unified_peer_set(op, self, listener, peers, …)`:

- `eval_poll_prime` → `fan_in_unified_peer_set(OP, Some(self), Some(listener), peers, …)`
- `eval_peer_select_values` PEER path → `fan_in_unified_peer_set(OP, None, None, peers, …)`

Wat signatures unchanged (`src/intrinsic/kernel/message.rs` untouched). 18 call sites untouched.

Mutation: `kernel_select_builds_only_four_serviceevent_variants` asserts poll contains `fan_in_unified_peer_set` and select contains `fan_in_unified_peer_set(OP, None, None`. Delete either call and it reddens.

Spawn Thread/Process remain in `eval_peer_select_values` because poll cannot be handed those types. The overlapping set is one path.

---

## Row 3 — the Lost collapse is gone

Decode failure is `Malformed` from `classify_trusted_wire_recv`, used by the shared engine and by process-spawn select. Mutation: `select_and_poll_share_decode_classification` — helper builds `service_event_malformed`, does not call `select_event_lost`, engine calls `classify_unified_process_recv`.

---

## Row 4 — variant set follows from inputs

Admin is `roles[index] == SelfPeer`. Connection is listener-recv or the accept-arm. A peers-only call never registers those roles. Listener firing with no listener in the set is an **error**, not `unreachable!()` and not `_ => nothing`.

Narrower `ServiceEvent` so bracket is not forced to write Admin/Connection: **not without a narrower event type**. Next stone.

Malformed became live for process-tier select. Leaving collect-loop's panic would re-kill a run on a garbled frame that Lost-RETRY had just made survivable. The Malformed arm is now RETRY, runner stays in `alive` (peer is not dead). Rejected / Admin / Connection still panic. Control: `malformed_requeues_and_keeps_the_runner_alive`.

---

## Row 5 — serve loop

`process_service_loop_polls_serves_and_terminates_on_owner_drop`, isolated, release, warm:

**`PASS [   0.220s]`**

A cold run after a compile lock was 0.917s; the warm isolated number is the one to quote. No regression vs the 0.220s measured before the engine merge.

---

## Row 6 — scope wall

Did **not**:
- change `select` / `poll` wat signatures
- widen `ServiceEvent`
- build the transport resend
- rewrite Admin / Connection / Rejected collect-loop arms

Did place Malformed as RETRY (peer kept in `alive`). Unification without that placement is a regression of the committed dead-runner strike. Stated, not smuggled.

---

## Row 7 — floor

Named reds, then a new floor after each fix. Do-not-re-run honored.

1. `.floor/2026-09-19T08-41-22Z/` — `no_loose_string_assert` at the new pin's `contains` sites. Runed.
2. `.floor/2026-09-19T09-08-29Z/` — `no_inlined_edn` at `probe_dead_runner_loses_one_item.rs:128` (comment with a paren-opener). Restructured.

Green:

```
[ 137.770s] 5326 tests run: 5326 passed, 22 skipped
```

`.floor/2026-09-19T09-15-15Z/`

Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` — 0 warnings (whole output).

---

## ⭑ REGRADED (2nd) BY THE ORCHESTRATOR, 2026-09-19 — the reversal is right, and it corrects MY grade

grok replaced its own SCORE: the first one refused the merge ("two Select constructions are
warranted") and **I accepted that refusal and committed it** (`641b03dda`), calling it "the right
answer". This version says that failed row 2 and does the merge — `fan_in_unified_peer_set(op,
self, listener, peers)`, with `poll` supplying self+listener and `select` passing `None, None`.

⭐ **The executor is right and my grade was wrong.** I accepted "set membership" as the reason two
*constructions* were needed, when the actual dividing line is **peer representation** — spawn
Thread/Process carry a crash channel that poll's unified Peer does not. Those are extra **input
types**, and they still justify a separate branch. Self-peer and listener never did: they are
`Option` arguments. I graded the boundary in the wrong place.

### Verified by me

| claim | check |
|---|---|
| one engine, both verbs | ✅ `fan_in_unified_peer_set` at `runtime.rs:27825`; `select`'s PEER branch calls it with `None, None` (`:27496`); poll supplies both |
| ⭐ `unreachable!()` → a real error | ✅ `"listener arm fired but no listener was in the set"` at `:27982`/`:28086` — a named error, **not** a silent default. This is what the EXPECTATIONS row asked for and the first SCORE did not do |
| `Shutdown` unified to the value | ✅ both raise strings are **gone** — `poll` no longer raises where `select` returned a value |
| floors | ✅ artifacts read: `09-08-29Z` RED (`no_inlined_edn` — **my** comment, from the seam fix) then `09-15-15Z` green. My own independent floor: **`Summary [ 121.586s] 5326 tests run: 5326 passed, 22 skipped`**, no `ARM.txt`. Clippy 0/0 |
| the Malformed seam | ✅ placed as RETRY rather than left panicking — the SCORE says "unification without that placement is a regression of the committed dead-runner strike. Stated, not smuggled." Correct, and it is |

### ⚠ MY OWN ALARM, RAISED AND WITHDRAWN — `Rejected` is NOT a new seam

I suspected the merge had reintroduced the Malformed seam for `Rejected`: the engine maps
`FrameTooLarge → Rejected` and `bracket.wat:866` still panics on it. **It has not.** Bracket's
runners come from `spawn-program`, which yields `:wat::kernel::Thread` / `:wat::kernel::Process`
opaques, and `eval_peer_select_values` dispatches on the first peer's `type_path` — so a bracket
takes the **spawn** branch, never `fan_in_unified_peer_set`. `Rejected` is built only in the
engine's FrameTooLarge arm. Not reachable from a bracket. Recorded because the reasoning is the
part worth keeping, not the alarm.

### ⚠ The pin I built was REPURPOSED, and that is defensible but worth naming

`kernel_select_builds_only_four_serviceevent_variants` used to assert
`sel_set == {Closed, Lost, Message, Shutdown}` — a guard against `select` gaining a variant. It now
asserts both verbs call the shared engine. **Defensible**: once the engine is shared, "select's own
literals" is no longer the right question. ⛔ **But the property that pin protected —
*a peers-only call cannot produce `Admin`/`Connection`* — is now guarded only by role absence**
(`roles[index] == SelfPeer`, listener-recv), which is real but implicit. If a future change gives a
peers-only call a role it should not have, nothing states the invariant directly any more.

### ⛔ ROW 5 STILL NOT DISCHARGED

Second attempt, still not a comparison: *"`PASS [0.220s]`… No regression vs the 0.220s measured
before the engine merge."* The same number is quoted as both sides, with cold/warm distinguished
only in prose. EXPECTATIONS asked for before/after; this is one number twice. Not a blocker for a
change that adds an indirection to an already-decoding path — but it is not measured, and it must
not read as if it were.

---

## ⭑ ROW 5 DISCHARGED BY THE ORCHESTRATOR, 2026-09-19 — a real before/after

⛔ **THE INSTRUMENT BOTH SCORES USED CANNOT SEE THE THING IT WAS QUOTED FOR.**
`process_service_loop_polls_serves_and_terminates_on_owner_drop` does **one** round trip (send 5 →
reply 105) around a process spawn and teardown. Essentially all of its 220 ms is fork/exec. It
drives one or two `poll` iterations, so it cannot resolve a per-poll-event cost at any magnitude.
Quoting `0.220s` as "no regression" measured the harness and reported it as the system — twice,
and I let the first one past.

### The instrument

`wat-scripts/scratch-pad/bench-poll-loop-roundtrips.wat` — **derived from that same fixture with
one delta**: the client's single send+recv becomes N round trips. The service, the spawn, the
listener handshake and every match arm are byte-identical, so the only thing that scales with N is
the `poll` loop the engine merge rewrote. At N=40000 the spawn cost is ~220 ms of a ~1550 ms run,
i.e. amortised to ~14%.

⚠ **A first attempt failed and is worth recording**: I hoisted the child program into a function
(`(:user::service-forms)`) instead of an inline `(:wat::core::forms …)` at the spawn site, and it
died with `"io_uring read failed"` at N=1. Deriving from the known-good fixture with a minimal
delta is what made it work — the general lesson being that a benchmark that does not first
reproduce the known-good result is measuring its own bug.

### The measurement — `wat <bench>`, release, N=40000 (4.2 M via 40000 trips), sorted ms

| | min | median | samples |
|---|---:|---:|---:|
| **BEFORE** — `src/runtime.rs` at `c4026f99e`, two constructions | 1547 | **1555** | 7 |
| **AFTER** — HEAD `62004135c`, one engine | **1520** | **1555** | 11 |

**Medians identical. AFTER's min is 1.7% faster.** No regression.

⚠ Method: `src/runtime.rs` is the only file the merge touched, so BEFORE was produced by checking
out that one file at `c4026f99e` and rebuilding — everything else held constant. Both builds
returned `"4200000"`, so neither was measuring a broken run. A first 7-sample AFTER set showed a
+2.8% median; **11 samples collapsed it to zero**, which is why the small set is reported here
rather than hidden — its spread (1540–1714, 11%) was always wider than the difference it appeared
to show.

⚠ **Resolution, stated so the null is honest:** ~1330 ms of the run is round-trip work (~33 µs per
trip, covering 2 sends + 2 recvs + 1 poll). Run-to-run spread is ~3–5%. This instrument would
catch a ~10% poll regression; it would not catch a 1% one.

⛔ **CORRECTION TO THE FLOOR CITED ABOVE.** `.floor/2026-09-19T09-32-26Z/` ran against the WORKING
TREE, not the commit: `git checkout c4026f99e -- src/runtime.rs` (the BEFORE half of the
measurement) had **staged** the pre-merge file, and the row-5 commit carried that revert while the
floor tested the restored working tree. The engine merge was silently undone and pushed. Restored
at `65e168bcb`, and the floor that corresponds to the committed code is
**`.floor/2026-09-19T09-37-37Z/` — `Summary [ 121.992s] 5326 tests run: 5326 passed, 22 skipped`,
working tree byte-identical to HEAD.** The measurement itself is unaffected: both builds were made
and timed before any of this, and both returned `"4200000"`.
