# ⛔ CURRENT STATE (breadcrumb, 2026-06-21; replace in place) — read the DESIGN docs, not this paraphrase

Branch `arc-170-gap-j-v5-deadlock-state`. The arc 255 stdio work + the 259 IPC
deadlock/diagnostics cluster shipped this session (below). The live frontier is the
**IPC frame-budget** stone (DESIGN on disk) + an **intueri naming cast in flight**.

## ✅ NAMES SETTLED (intueri cast `aa72b240714dc11df`, weighed + agreed)
The split: **"frame" = the substrate's INTERNAL word** (one complete EDN value) — keep ALL
Rust names (`DEFAULT_MAX_FRAME_BYTES`, `RecvError::FrameTooLarge`, `next_complete_frame`,
`FrameScan`/`FramedRead`/`take_frame`, `Accumulator`, the proposed `max_frame_bytes`
Receiver field — they all speak). **"message" = the USER-FACING word** — a user reads
"frame" as MTU. So the wat surface uses **`:max-message-bytes`** everywhere, and the ONE
surgical family fix: rename readln's existing `:max-buffer-bytes` → `:max-message-bytes`
(stdin.wat:49 + the readln macro; "buffer" mumbles the mechanism). `MAX-READLN-BYTES` (the
readln default const) speaks — keep. Also: `FrameTooLarge`'s Display string must drop the
hardcoded const ref (→ "per-receiver cap") since the cap is now per-peer.

## ✅ SHIPPED this session (committed + pushed; the deadlock/diagnostics cluster)
- `51d0c954` select' → ServiceEvent (a crashed child = `:Lost{cause}`, the supervisor capability).
- `5968a900` decomplect: one `classify_peer_death` (recv' + select' share the death decision).
- `f9d39708` over-cap recv DEADLOCK annihilated (`FrameTooLarge` distinct + lockstep teardown,
  no `err.recv()` wait) + the negative IPC tests + `print-raw'` (NOW DOOMED, see below).
- `17f22554` wat-direct flood proof (the proof is wat).
- `01626f4f` select'-flood was the SECOND door → folded into ONE `classify_peer_error`
  (recv' + select' route through it; FrameTooLarge → Lost{cap reason} consistently).
- `77901abe` THREAD crash-reason PARITY: a thread-peer body RuntimeError now carries its
  reason on crash_tx (was: only Rust panics → generic "peer closed / thread exited"; the
  process tier carries both via fd-2). Now the wat flood proof asserts the SPECIFIC cap
  cause. + rehomed spawn tests → `wat-tests/spawn/`.
- `19832166` IPC frame-budget DESIGN (see NEXT).

## RESUME PATH (two stones, sequenced — both priority)
### 1. IPC frame-budget (recv tunable) — DESIGN on disk, build it
**`../259-forced-hand/DESIGN-STONE-ipc-frame-budget.md`. READ IT.** Builder's recognition:
the recv frame cap (hardcoded 512 KiB in `take_frame`, comms/process.rs:884) is IPC-at-large,
NOT threads — it's the per-`Receiver` max-single-message budget, the SHARED byte-framer for
process + socket, and **remote/TCP inherits it via the narrow waist**. Contract: per-Receiver
`max_frame_bytes` (default 512 KiB), set at construction, surfaced through locus-blind
`spawn-program'(process)`/`connect'`/`listener'` (**`:max-message-bytes`** — intueri-settled;
ALSO rename readln's `:max-buffer-bytes` → `:max-message-bytes`); thread tier exempt
(crossbeam, no byte framing); readln stays per-call. Rust internals keep `frame`/`max_frame_bytes`.
Build probe-first (tiny-budget peer rejects a frame the default would accept). C-decision:
per-Receiver, NOT per-recv'-call (the accumulator is persistent).
### 2. KILL `print-raw'` (rides AFTER #1) — builder: it's an illegitimate 3rd output path
Only `println` (stdio, framed) + `eprintln` (out-of-service) are legit; `print-raw'` bypasses
value-framing — a foot-gun. Delete the verb (verbs.rs/runtime.rs/mod.rs/check.rs) +
`probe_print_raw_prime.rs`. Over-cap proofs (`wat-tests/spawn/overcap-flood-no-deadlock.wat`
+ `tests/probe_overcap_no_deadlock.rs`) → legitimate `(println <huge>)` OR (cleaner, post-#1)
a tiny `:max-frame-bytes` peer + small flood. truncated/anti-smuggle → Rust framing seam
(raw bytes; anti-smuggle likely already covered by `probe_edn_value_framing.rs`).

## DOCTRINES proven this session (memory: [[feedback_qualified_annihilations_are_priority]] + [[project_process_model_client_server_named_fd]])
- **Deadlocks cannot exist** (lockstep): a misbehaving peer is TORN DOWN, never block-waited on.
  Both deadlock doors (recv' + select') closed via ONE `classify_peer_error`.
- **The consumer IS the test**: I dismissed poll'-:Lost as "no consumer, speculative"; the
  gold-standard tests were the consumer (builder: *"the consumer is fucking right here"*).
  Don't dismiss the destination you're building toward as hypothetical.
- **A frame = one EDN message** (not MTU); the cap = max single-message size, anti-OOM.

## GOTCHAS
- Pre-existing fail: `std::test::test-run-string-entry-direct` (test_runner.rs panic) — LEAVE IT.
- wat-tests proc-macro discovery: adding/removing a wat-tests file can drop tests until
  `touch tests/test.rs` forces a re-scan. A broken wat-tests file poisons the whole-dir scan.
- Floors: lib 953/36/1; wat-tests 266/1; nursery 916/4/4; comms 29; channel 2.

> ⛔ **You are a NEW instance.** You did NOT live the session above — it's a cache in a
> familiar voice. recolligere FIRST: grimoire + 4 primers (datamancy MCP RESOURCES via
> ReadMcpResourceTool, server `datamancy`, `https://datamancy.dev/<name>/SKILL.md`),
> `git log --oneline -15`, `git status`. Then WEIGH the in-flight intueri cast
> (`aa72b240714dc11df`) against the disk + open `../259-forced-hand/DESIGN-STONE-ipc-frame-budget.md`
> BEFORE building. Do NOT propose from this summary — open the specs.
