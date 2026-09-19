# SCORE — one selectable-set primitive

**SCORED as a STRIKE on the classification door, not a merge of the two Select constructions.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `338800365`. Did not commit.

Sentence: **`select(peers)` is `poll(∅, ∅, peers)`. There is one waiting discipline, not two.**

The decode-failure collapse is gone: both verbs classify a trusted-wire Recv Ok through `classify_trusted_wire_recv`. A garbled frame is `Malformed` from both. Select construction is still two functions — the remaining cells are set-membership and peer-kind, not a second waiting discipline. Two impls are **not** warranted for the overlapping Recv-Ok-wire cell. They **are** still two functions for building the `Select`.

---

## Row 1 — the drift table

`SelectOutcome` (`src/comms/mod.rs:507`) is `Recv { index, result }` · `Shutdown` · `Listener`. `result` is `Ok(T)` or `Err(RecvError)`. Select has **three** peer representations; poll has **one**.

Peer kinds:

| kind | type_path | crash channel | who uses it |
|---|---|---|---|
| spawned Thread | `THREAD_PEER_TYPE_PATH` | yes | **select only** |
| spawned Process | `PROCESS_PEER_TYPE_PATH` | yes (err rx) | **select only** |
| unified Peer | `PEER_TYPE_PATH` | no | select **and** poll |

### Thread tier

| SelectOutcome | select (spawn Thread) | select (bare Peer) | poll (unified Peer) |
|---|---|---|---|
| Recv Ok | death-notice → **Lost**; Reply::Failed → **Malformed**; else **Message** | same death-notice / Failed / Message | Reply::Failed → **Malformed**; else **Message**. **No death-notice check** |
| Recv Err | `classify_peer_death(crash_rx)` → **Lost** / **Closed** / **Shutdown** | **Closed** (no crash channel) | index 0 (self) Err → **Shutdown** + `broadcast_peer_severed`; index 1 (listener) Err → **RAISE**; client Err → **Closed** |
| Shutdown | **ServiceEvent::Shutdown** | **ServiceEvent::Shutdown** | **RAISE** `MalformedForm` `"select interrupted by shutdown"` |
| Listener | `unreachable!("thread-tier Peer Select has no listener arm")` | same | `unreachable!("thread-tier poll Select has no listener arm")` — listener is recv index 1, not this variant |

### Process tier

| SelectOutcome | select (spawn Process) | select (bare Peer) | poll (unified Peer) |
|---|---|---|---|
| Recv Ok (wire) | **now** `classify_trusted_wire_recv`: death-notice → Lost; Failed → Malformed; decode fail → **Malformed**; else Message. **Was Lost on decode.** | **now** same helper. UTF-8 fail → **Malformed** (was Lost). Decode fail → **Malformed** (was Lost) | **now** same helper. UTF-8 fail → **Malformed** (was RAISE). Decode fail → **Malformed** (unchanged variant; interpolates `e`) |
| Recv Err FrameTooLarge | `classify_peer_error` → **Lost** (cap reason) | folded into Err(_) → **Closed** | **Rejected** |
| Recv Err other | `classify_peer_error` → Lost / Closed / Shutdown; timer (no err rx) → Closed | **Closed** | index 0 Err → **Shutdown** + broadcast; client Err → **Closed** |
| Shutdown | **ServiceEvent::Shutdown** | **ServiceEvent::Shutdown** | **RAISE** `"poll interrupted by substrate shutdown"` |
| Listener | `unreachable!("process-tier 1-arg select has no listener arm")` | same | **Connection** (accept-arm; process poll registers `sel.listener(fd)`, not a recv index) |
| io_uring `Err` | **Lost** idx=0 `"select io_uring error"` | **Lost** idx=0 | **RAISE** `MalformedForm` interpolating `io_err` |

Index layout (set membership, not classification):

| | indices |
|---|---|
| select (any) | 0..=N-1 = peers |
| poll thread | 0 = self, 1 = listener recv, 2..= = clients |
| poll process | 0 = self, 1..= = clients; listener is the accept-arm, **not a recv index** |

⭐ Cells that are **not** the known decode row, and that nobody had compared:

1. **Recv Err + crash channel** (select spawn) vs **always Closed** (bare Peer / poll clients). Peer-kind, not verb. Poll never sees a spawned Thread/Process.
2. **SelectOutcome::Shutdown**: select returns a value; poll **raises**. Serve loop already matches `ServiceEvent::Shutdown` (owner-drop Recv, not this variant). Left as-is — unifying poll to the value would change a raise into a match, which is a serve-loop behavior change this stone did not take.
3. **FrameTooLarge**: poll **Rejected** (designed 400-class, keep serving); select spawn **Lost**; select bare **Closed**. Left as-is. Adopting poll's Rejected for select would make bracket's Rejected arm live; the parked stone owns placing it.
4. **io_uring error**: poll raises; select lies with Lost idx=0. Left as-is.
5. **Thread poll Recv Ok** still has no death-notice check. Select does. Left as-is (service clients do not send those sentinels as Ops).

None of (1)–(5) is "the service needs behaviour a pool must not have" on the **overlapping Recv-Ok-wire cell**. That cell was drift. It is unified.

---

## Row 2 — one engine (classification), two Select builders

Both wat verbs survive; signatures byte-identical (`src/intrinsic/kernel/message.rs` untouched).

What unified: `classify_trusted_wire_recv` is the one door for process-tier Recv Ok on a trusted wire. `eval_peer_select_values` (spawn Process + bare Fd) and `eval_poll_prime` (client arm + re-poll) both call it.

What did **not** merge: the two functions that **build** the `Select`. Process poll's listener is an accept-arm (`sel.listener(fd)`), not a recv index — `select(peers)` cannot be a literal `poll(None, None, peers)` call without inventing a third construction that omits that arm. Spawn peers carry a crash channel poll's unified Peer does not. Those follow from **what is in the set**, not from a second waiting discipline.

`unreachable!()` on `SelectOutcome::Listener` remains in select: the **enum** still has the variant (next stone: a narrower type). A peers-only construction never registers a listener, so nothing in the set can produce it. Expressing that in the TYPE needs a `SelectOutcome` without `Listener` — said so, left.

---

## Row 3 — the Lost collapse is gone

Decode failure is `Malformed` from both verbs. Mutation: `select_and_poll_share_decode_classification` — both verbs call `classify_trusted_wire_recv`; that helper's decode Err builds `service_event_malformed` and does **not** call `select_event_lost`. Delete the helper from either verb, or put `select_event_lost` back in the decode Err, and it reddens.

Parked stone consequence: bracket's `Lost` RETRY is now death-only. Decode failure hits the existing `Malformed` panic arm. That arm was **not** rewritten (scope wall).

---

## Row 4 — variant set follows from inputs

Admin / Connection are still impossible on a peers-only call because self-peer and listener are not in the set. Pinned: `kernel_select_builds_only_four_serviceevent_variants` — select's own literals stay `{Closed, Lost, Message, Shutdown}`; Admin/Connection must not appear there. Malformed is live via the helper, not as a select literal.

Narrower `ServiceEvent` so bracket is not forced to write impossible arms: **not without a narrower event type**. Next stone. Not this one.

---

## Row 5 — serve loop

Existing service test `process_service_loop_polls_serves_and_terminates_on_owner_drop` (the poll hot loop at `service.wat:2478`), isolated, release:

| | |
|---|---|
| after | `PASS [   0.220s]` |
| contended parallel run of the same test | 0.939s (noise from sharing the floor; not used) |

No regression observed on the isolated run. Single number to quote: **0.220s after**.

---

## Row 6 — scope wall

Did **not**:
- rewrite bracket's four collect-loop arms (Admin / Connection / Malformed / Rejected still panics; Malformed is now *reachable* on process-tier select and still panics until the parked stone places it)
- build the transport resend
- change `select` / `poll` wat signatures
- widen `ServiceEvent`
- merge the two Select constructors
- unify FrameTooLarge / Shutdown-raise / io_uring (named in the table, not silently adopted)

---

## Row 7 — floor

First floor this SCORE, red. Do-not-re-run honored. ARM `.floor/2026-09-19T08-41-22Z/ARM.txt`:

```
wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
tests/kernel/probe_bare_recv_outcome_surface.rs:243
tests/kernel/probe_bare_recv_outcome_surface.rs:263
```

Two `contains` sites in the new pin without `rune:lint(loose-assert)`. Named, then runed (targeted presence/absence over a function-body slice). New floor after the fix:

```
[ 129.290s] 5325 tests run: 5325 passed, 22 skipped
```

`.floor/2026-09-19T08-45-38Z/`

Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` — 0 warnings (whole output).

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted, with a REGRESSION AT THE SEAM that is mine

### The refusal is the right answer, and it is the best thing here

The stone's sentence was mine and it was too strong. **"Two Select CONSTRUCTIONS are warranted; the
classification cell was drift"** is the honest split, and it is argued from the table rather than
asserted: process poll's listener is an **accept-arm** (`sel.listener(fd)`), not a recv index, and
spawn peers carry a **crash channel** poll's unified Peer does not. Those are set-membership and
peer-kind facts. The overlapping Recv-Ok-wire cell was not — it was drift, and it is gone.

⭐ **Row 1 delivered what it was for: five uncompared cells**, none of which anybody had looked at —
crash-channel Err, `Shutdown` value-vs-raise, `FrameTooLarge` (Rejected / Lost / Closed three ways),
io_uring error (raise vs a `Lost idx=0` that *lies*), and thread-poll's missing death-notice check.
Each named and **left**, not silently adopted. That is exactly the failure mode the row existed to
prevent.

| claim | my check |
|---|---|
| the collapse is gone | ✅ `classify_trusted_wire_recv` (`runtime.rs:22053`), 5 sites; decode `Err` → `service_event_malformed`; `"select EDN decode failed"` is now a *reason string* feeding Malformed, not `select_event_lost` |
| both verbs share it | ✅ |
| pins | ✅ 6/6, including the new `select_and_poll_share_decode_classification` |
| floors | ✅ read directly: `08-41-22Z` **RED** with `ARM.txt` (2 un-runed `contains` sites, named then runed), `08-45-38Z` green `5325 passed`, no `ARM.txt` |
| scope wall | ✅ bracket's arms, the resend, both wat signatures, `ServiceEvent`'s shape — untouched |

### ⛔ ROW 5 IS NOT DISCHARGED

EXPECTATIONS asked for **before/after** on the serve loop. The SCORE gives only an after
(`0.220s`), and the word "before" appears **zero** times in it. One number is not a comparison. Not
a blocker — this stone adds one function call on a path that already decoded — but it is not
measured, and it should not read as if it were.

### ⛔⛔ AND A LIVE REGRESSION AT THE SEAM — NOT grok's ERROR, MINE

grok discloses it inside row 3 and stays inside the scope wall. Stated plainly, because a reader
will not reconstruct it from "parked stone consequence":

| | a garbled frame on a process-tier bracket |
|---|---|
| **before this strike** | decode fail → `Lost` → `collect-requeue` → **re-dispatched; the run survives** |
| **after this strike** | decode fail → `Malformed` → `assertion-failed!` (`bracket.wat:833`) → **the whole run dies** |

The re-dispatch stone landed first (`338800365`, struck before the park existed), so bracket *had*
a recovery for this fault. This stone then reclassified the fault **out of that arm's reach** and
into a panic. Net: a crusade against ungraceful failure has, at this commit, made one fault
ungraceful that was graceful an hour ago. **No test covers it** — there is no bracket
decode-failure probe.

⭐ **The ordering caused it.** Had the unification landed first, bracket would never have had the
recovery to lose, and the parked stone would have placed `Malformed` correctly from the start. I
parked the bracket stone behind this one for exactly this reason and then graded them in the
opposite order.

**Fixed in the follow-up commit**, not filed: `Malformed` re-queues like `Lost`, with its own report
— and unlike `Lost` it must **keep the runner in `alive`**, because a peer that sent a bad frame is
by definition still there. The wall-clock bound remains the stop for a deterministic encode bug.
