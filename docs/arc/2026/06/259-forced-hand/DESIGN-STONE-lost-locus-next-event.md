# DESIGN — Stone: `:Lost` for local crashes via a locus-blind `next_event` protocol

**Status: DESIGN (not yet STRIKE-READY — RED probe pending).** Drawn 2026-06-21 via the
inquisitor crawl (examinare): we grounded the whole peer/poll'/spawn dungeon before striking,
which turned a guessed "flip one match arm" into the real contract below.

## The flaw (root cause — grounded)
`:wat::spawn::ServiceEvent` has `:Lost{idx, cause :Failure}` (wat/spawn.wat:115), but `poll'`
NEVER emits it for a local crash — it folds every disconnect to `:Closed` (runtime.rs:25196
thread-tier, 25352 process-tier). The cause IS available; it is **stranded**:

- The unified `Peer` (src/kernel/peer.rs:206) is `{ tx, rx }` — **NO crash channel.** When the
  tier peers collapsed into the transport-blind `Peer` (arc 209 "unify Peer collapse"), the
  death channel was dropped. The tier structs still have it: `Thread.crash` (crash_rx),
  `Process.err` (err_rx, the fd-2 channel; spawn.rs:644-658, child dup2's fd 2 → err_tx).
- `recv()` on a `ProcessPeerBundle` DOES demux it correctly (spawn.rs:240): read `output`; on
  Ok-EOF read `err` → `Crashed(reason)` vs `Disconnected`. So `recv'` raises with the cause.
- `poll'` has a SEPARATE, LESSER copy of that logic that just says `:Closed`. **Duplication,
  and the duplicate is wrong.** (Same disease 259.S3.6 cured for framing: two copies, one
  incomplete.)
- A spawned child is a `Process'<I,O>` bundle (`PROCESS_PEER_TYPE_PATH`), NOT a `Peer`
  (`PEER_TYPE_PATH`) — so a supervisor can't even put its child into `poll'`'s peers arg
  (eval_poll_prime, runtime.rs:24905+, expects `Peer` opaques).

## The contract — locus as a defprotocol (builder's framing, confirmed)
`thread != process != remote` in their GUTS, but the SURFACE is constant. `spawn-program'`'s
surface (`send'`/`recv'`/`poll'`/`select'` + `ServiceEvent`) is one protocol; each locus is a
bespoke impl that HIDES its machinery. The death-cause arrives differently per locus:

| locus   | channels        | how the cause arrives                          | built? |
|---------|-----------------|------------------------------------------------|--------|
| thread  | output + crash  | separate in-memory death channel (`crash_rx`)  | yes    |
| process | output + err    | the fd-2 third channel (`err_rx`)              | yes    |
| remote  | rx, tx ONLY     | **multiplexed over the read channel** (`Result<T,E>` on rx) | NO (contract-shaped) |

**Remote is the forcing function.** It has no third channel — so the SURFACE must not know
about one. `poll'` must never touch `ex`/`err` directly; it must call a uniform method and let
the locus demux. Keeping remote perpetually unbuilt FORCES the contract correct now.

## The move — ONE locus-blind `next_event`, route everything through it (the decomplect)
Define a single locus-blind event method on the peer's receive path:

```rust
// the ONE death-aware receive. Locus-blind surface; bespoke per-locus demux.
enum PeerEvent { Message(Value), Closed, Lost(Value /* :wat::kernel::Failure */) }
fn next_event(&self) -> PeerEvent
```

- **process impl** — read `output`; on Ok-EOF read `err` → reason ⇒ `Lost(cause)`, EOF ⇒
  `Closed` (verbatim `recv()` logic, spawn.rs:240).
- **thread impl** — read `output`; on Ok-EOF read `crash` → `Lost(cause)` | `Closed`.
- **remote impl** — read `rx`; an `Err` frame on the read channel ⇒ `Lost(cause)`, clean EOF ⇒
  `Closed`. **`unimplemented!` for now** — the slot exists; building remote later fills it.

Then route ALL THREE consumers through `next_event`:
- `recv'` — `next_event` → `Message(v)` returns v; `Lost(cause)` RAISES (preserve today's
  behavior); `Closed` → the clean-disconnect path.
- `poll'` (runtime.rs:25196/25352) — `next_event` → emit the matching `ServiceEvent`
  (`:Message`/`:Closed`/`:Lost{idx,cause}`). NO `ex`/`err` special-casing in poll' itself.
- `select'` — same.

Net: the crash-demux lives in ONE place per locus; `recv()`'s good copy and `poll'`'s bad copy
collapse into the protocol method. Annihilation, not widening.

## The C decision (pinned): where the crash source lives
The cause source (process `err`, thread `crash`) must travel WITH the peer `poll'`/`select'`
watch — so the per-locus receive impl must bundle it. Two sub-options for the next self to weigh
in the STRIKE (not yet locked):
- **C1 (lean):** make the tier receiver self-sufficient — the process tier's receive impl holds
  `output + err`; thread holds `output + crash`; the unified `Peer` carries that locus impl
  (trait object) and `next_event` is a trait method. `poll'`/`select'` call it uniformly; the
  spawn-bundle becomes representable as a `Peer` so a supervisor can poll its children.
- **C2:** keep `Peer = {tx, rx}` and add `crash: Option<…>`; thread it through construction.
  Simpler field-add, but leaves recv-vs-poll demux duplicated unless `next_event` is still the
  single router. Prefer C1 (it's the decomplect; C2 just relocates the field).

## Exact change sites
- `src/kernel/peer.rs:206` — `Peer` gains the locus receive impl (C1) carrying the crash source.
- `src/kernel/peer.rs:215/231` — `from_thread`/`from_socket` thread the crash source; a new
  `from_process_bundle`/`from_thread_peer` (or poll' accepts bundles) bridges spawned children.
- `src/kernel/spawn.rs:240` — `recv()` re-expressed as `next_event` + raise-on-Lost (or calls it).
- `src/runtime.rs:25196, 25352` — `poll'` Err arms → `next_event` → `:Lost{cause}`/`:Closed`.
- `select'` arm(s) — same routing.
- cause conversion: reuse `ProcessDiedError/to-failure` (runtime.rs:4688) + `extract_panics`
  (runtime.rs:11468) — `reason: String` → `:wat::kernel::Failure`.
- `:wat::spawn::ServiceEvent` (wat/spawn.wat:110-115) — already has `:Lost{idx,cause}`; no change.

## RED probe (pending — write + verify RED before the strike)
A supervisor spawns a process child whose `:user::main` `panic!`s (or asserts-false); the
supervisor observes via `poll'`/`select'` → expect `:Lost{cause}` carrying the child's `Failure`.
RED at HEAD: today this yields `:Closed` (or the child isn't pollable at all → the probe
compile-fails at the missing bridge — a valid gap-isolating RED). GREEN after the protocol lands.

## Gate (independent scorecard — fill at STRIKE)
- the supervisor :Lost probe goes green; cause carries the child's Failure message.
- recv' still raises Crashed (behavior preserved) — existing comms/process tests green.
- poll'/select' suites green; service-locus-parity green.
- lib floor (953/36/1), nursery floor unchanged.
- remote slot is `unimplemented!` and there is NO remote construction path (honest omission).

## Grounding update (2026-06-21, deeper inquisitor pass — supersedes guesses above)
- **`select'` already takes process children.** `eval_peer_select_prime` (runtime.rs:24472)
  dispatches on the FIRST peer's `type_path` — THREAD_PEER_TYPE_PATH (:67), PROCESS_PEER_TYPE_PATH
  (:166), PEER_TYPE_PATH (:288). The "bundles aren't pollable" fear was WRONG: a supervisor can
  already `(select' [child …])`. The THREE real gaps, all in this fn's arms:
  1. on a peer death it **RAISES** ("peer closed / child exited", :252 process / :146 thread) —
     crashes the supervisor instead of yielding an event;
  2. it returns **`Tuple<i64, O>`** (:272), not `ServiceEvent`;
  3. it extracts only the **output** receiver (`process::Receiver<String>` :212 / thread Receiver
     :114) — never the `err`/`crash` channel, so no cause.
  → The strike is concentrated here + the poll' arms + recv(): all route through `next_event`,
  return `ServiceEvent`, read the crash channel on death.
- **There is ALSO a legacy `select`** (no prime; `eval_kernel_select`, runtime.rs:20074) —
  Receiver-based, thread-tier ONLY, `Tuple` return, `Ok(None)` on death. Separate older
  multiplexer; fold it into the annihilation or confirm it's dead (bracket uses `select'`, the
  primed one). Check callers before touching.
- **ServiceEvent migration blast radius:** `select'` → `ServiceEvent` changes every `select'`
  caller — today `wat/bracket.wat` (extracts `(idx,result)` Tuple; migrates to `:Message{idx,msg}`,
  gains `:Lost` free). Grep `select'` callers before the strike.
- **The RED probe must be RUST, not a wat-tests deftest.** At HEAD `select'` returns `Tuple`, so
  any wat probe matching `ServiceEvent` is a CHECK error — and the wat-tests harness **aborts a
  file's load on a check error** (it silently drops that file's tests AND poisons the
  whole-directory proc-macro discovery until a forced re-scan; a broken file dropped the
  round-trip test this session until `touch`). So author the probe as a Rust integration test
  (model: `tests/wat_process_peer_ipc_round_trip.rs`) — spawn a process peer whose `:user::main`
  crashes (`Option/expect` on `None`), `select'` over `[child]`, assert `ServiceEvent::Lost` with
  the cause; RED at HEAD (Tuple / raise), GREEN after.

## Out of scope (affirmative cuts)
- The remote IMPL (TCP socket spawn-program') — contract-shaped + `unimplemented!` only.
- The negative gold-standard tests — they do NOT need `:Lost`; they ride `recv'` (already raises
  `Crashed`) + `:should-panic` + the `print-raw'` tooling. Separate, smaller track.

## Sibling track (not this stone): the negatives + `print-raw'`
- `:should-panic "<substr>"` EXISTS (wat/test.wat:407-429; corpus proof
  wat-tests/core/unknown-call-head-panics.wat:10) — the harness CAN assert a rejection. This is
  the capability the last sonnet burned for lack of.
- Need `:wat::kernel::print-raw'` — a namespace-restricted ambient no-newline stdout write
  (`#[restricted_to(…, ":wat::test::")]`), so a wat child can emit malformed frames (over-cap
  un-terminated, anti-smuggle two-on-a-line). `IOWriter/write` does no-newline but only on an
  IOWriter object, not ambient fd 1.
- Then 3 negatives as `:should-panic` deftests: over-cap / truncated / anti-smuggle, child built
  with `print-raw'`, parent `recv'` → raises → should-panic asserts. (truncated also expressible
  via a child that `print-raw'`s a partial value then exits.)
