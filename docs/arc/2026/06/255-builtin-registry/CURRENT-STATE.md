# ⛔ CURRENT STATE (breadcrumb, 2026-06-21; replace in place) — read the DESIGN docs, not this paraphrase

## ✅ JUST LANDED: arc 259.S3.6 — one frame-finder (`ecda39e2`, pushed)
The decomplect is DONE and weighed against the disk (every gate re-run by hand, not
trusted from the build agent's report). `next_complete_frame` (src/edn_shim.rs:1060) is
the ONE pure byte-level frame-finder; both `read_framed_edn` (the blocking IOReader path)
and comms `take_frame` (process.rs, io_uring path) route through it, so framing can no
longer diverge. `take_frame` widened `Option<Frame>` → `Result<Option<Frame>, RecvError>`
to carry the cap. **Grounded deviation**: an EDN-syntax-`Malformed` prefix → `Frame(end)`
(NOT `FrameScan::Malformed`) because `String` wire content is raw passthrough (`from_wire`),
not EDN — the content error surfaces at decode (`from_wire`), which the anti-smuggle test
asserts. `FrameScan::Malformed` is now non-UTF-8 wire bytes alone.

Gates (all re-run here): comms multi-line probe RED→GREEN; comms 29/29; channel 2/2;
lib 953/36/1 (baseline); nursery 916/4/4 (+1 = the probe; the 4 fails are pre-existing
reflection/builtin-resolution, unrelated); **live proxy round-trips** (`wat
intrinsic-metadata.wat | wat read-flat.wat`, the gate the agent's sandbox couldn't run).
⚠️ The full nursery suite has a FLAKY process-deadlock SIGKILL under parallel load on this
branch (`arc-170-...-deadlock-state`) — passes isolated; a second full run completed. It is
PRE-EXISTING (tasks #163/#183/#207), not a regression. If a nursery run gets SIGKILL'd on a
`probe_arc209`/process test, re-run; don't chase it.

## ✅ 255 IS UNBLOCKED — there is NO design fork (an earlier note here was WRONG, retracted)
The PROCESS MODEL (builder, grounded in the tree): **client (parent) gets the named fd =
`Process'<I,O>` peer** (`recv'`/`send'`/`poll'`/`select'`/`close'`); **server (child) just
uses stdio** (ambient `readln`/`pprintln`). `spawn_process_peer` dup2's the child's fd 0/1
onto the channel pipe (verbs.rs:387, mod.rs:69) — so the child writes stdout, the parent's
`recv'` reads it through the named fd. 259.S3.6 made that `recv'` value-frame, so a server's
multi-line `pprintln` now returns as ONE value. **That was the only gap. 255 is unblocked
through the PRIMED peer — no raw-bound-stdio surface is needed or wanted** (the old "Peer'
has no raw stdio handle" complaint was the INTENDED design, not a gap).
`spawn-process'` (kernel/spawn.rs:344) returns `Process'<I,O>`; `spawn-thread'` returns
`Thread'`. Use these. **PRIMED ONLY** — non-prime `spawn-program`/`spawn-thread`/
`spawn-process` + the 4-field `:wat::kernel::Process` stdio record (verbs.rs:725,
IOWriter/IOReader/IOReader/ProgramHandle) are PENDING ANNIHILATION; do NOT build on them.

## ✅ ROUND-TRIP gold-standard test LANDED (`1d91fcec`, pushed)
`wat-tests/process-multiline-roundtrip.wat` — a `deftest'` whose body spawns a forms-server
child that `pprintln`s a 5-key map (7 physical lines), parent `recv'`s it as ONE value,
`assert-eq` == the map. Non-vacuous (pre-259.S3.6 it'd have gotten `{`). Green by own run
(suite 265/1; the 1 fail = `std::test::test-run-string-entry-direct`, PRE-EXISTING, leave it).

## RESUME PATH — two tracks (the rich design round of 2026-06-21 is on disk; READ them)
### Track 1 — the THREE negatives (no new substrate; the keystone capability EXISTS)
`:should-panic "<substr>"` is the assert-a-rejection capability (wat/test.wat:407; corpus
proof wat-tests/core/unknown-call-head-panics.wat:10) — this is what the last sonnet burned
for lack of. The negatives are `:should-panic` deftests: child emits a malformed frame, parent
`recv'` → RAISES (recv' already surfaces `Crashed`/decode errors) → should-panic asserts.
**Needs ONE small tool:** `:wat::kernel::print-raw'` — a namespace-restricted ambient
no-newline stdout write (`#[restricted_to(…, ":wat::test::")]`; `IOWriter/write` is no-newline
but only on an IOWriter object, not ambient fd 1). With it: over-cap (un-terminated >512KiB),
anti-smuggle (two values one line), truncated (partial then exit). Build `print-raw'` (probe →
DESIGN → strike), then the 3 deftests. Then **#268** the unbounded-LINE bound.

### Track 2 — `:Lost{cause}` for local crashes — SLICE 1 LANDED (`51d0c954`)
✅ **select' slice done & green** (weighed by hand): `select'` now returns `ServiceEvent` —
a crashed peer → `:Lost{cause}` (death demux mirrors `recv()` spawn.rs:240: output-EOF → read
the crash channel, thread `crash`/process `err`; bare connection peer → `:Closed` only). Cause
via the pre-existing `message_only_failure` (runtime.rs:21873). RED probe:
`tests/probe_supervisor_select_lost.rs`. Callers migrated: bracket.wat + 3 test files.
✅ **decomplect slice done & green** (`5968a900`): one `classify_peer_death` (spawn.rs:204) —
`select'` (both arms) + `recv()` share the Lost-vs-Closed decision; inline copies gone. Behavior
preserved. `poll'` was NOT folded in — correctly: its peers are connect'-accepted unified `Peer`s
with NO crash channel, so it never did the demux; `:Closed` is honest there.
**REMAINING:** (a) ⚠️ **poll' `:Lost` is a PHANTOM stone — do NOT add a crash field to `Peer`.**
Grounded 2026-06-21: all 9 `Peer` construction sites (from_thread/from_socket) are crashless by
nature — self-peers, connect' clients, accept'd connections, peer-pair' ends. NONE is a
supervisor-handle-to-a-crashable-child (those are `Process'`/`Thread'` BUNDLES, watched by `select'`,
already `:Lost`). So a `Peer.crash` field would be DEAD (no producer); `poll'`→`:Closed` for
connections is HONEST. `poll'` could only emit `:Lost` if it ACCEPTED bundles (heterogeneous) — that
is ADDITIVE + SPECULATIVE (no consumer multiplexes accepted-clients + spawned-children today), so NOT
a qualified annihilation. The `:Lost`-for-local goal is DONE via `select'`. (b) ✅ **THE real
remaining annihilation: the legacy non-prime `select`** (eval_kernel_select:20074, Receiver-based/
thread-only/Tuple/Ok(None)-on-death) — a genuine duplicate of the unified `select'`+classify_peer_death
path; fold or HARD-CUT (settle live-or-dead first; bracket uses the primed `select'`).
**DESIGN on disk: `../259-forced-hand/DESIGN-STONE-lost-locus-next-event.md`. READ IT.**
The flaw (inquisitor-grounded): the unified `Peer` (peer.rs:206 = `{tx,rx}`) DROPPED the crash
channel in the arc-209 unification; `Thread.crash`/`Process.err` are stranded on the tier
structs; `poll'` folds every death to `:Closed` (runtime.rs:25196/25352) while `recv()` (spawn.rs:240)
correctly demuxes the cause — a recv-vs-poll DUPLICATION. The cure = a locus-blind `next_event`
protocol (a DEFPROTOCOL: constant surface, bespoke per-locus guts): thread reads `crash`,
process reads `err`, **remote demuxes `Result<T,E>` over rx (rx/tx ONLY — the forcing function;
`unimplemented!` for now)**. Route `recv'`/`poll'`/`select'` ALL through `next_event` →
annihilate the duplicate demux; `:Lost{cause}` falls out uniformly. NOT a cheap flip — a real
multi-file stone (C decision = C1: tier-receiver self-sufficient, bundle becomes a pollable Peer).
The negatives do NOT depend on this (they ride recv'+should-panic).

## SHIPPED earlier this session-cluster (255 stdio value-framing + symmetry; all pushed)
`695eca16` iv-c (metadata-of plain values + Kind/DefinedIn/Layer enums) · `e92f5333`
pprintln · `1632d02c` value-framing + symmetry (epprintln; Receiver value-frames) ·
`49cbe8ee` 512 KiB cap · `0854b081` `:max-buffer-bytes` escape hatch · `4fb86f8b`
`:wat::core::Value` EDN coerce target · `28853601` 259.S3.6 STRIKE-READY · `ecda39e2`
259.S3.6 GREEN · `1d91fcec` round-trip gold-standard wat-test · (docs: `84a3d22d` retract the
phantom design-fork, `c4c44856`/`ab0d89a3` curare).

## DISCIPLINE (memory written) — `feedback_probe_capability_before_delegating`
Probe the HARNESS/SUBSTRATE capability ("can it even OBSERVE a failure?") before delegating,
not just the feature. And: the weigh THIS session caught a build agent's report that listed a
sandbox-blocked gate (the proxy) as if checked AND missed a flaky parallel SIGKILL — re-run
EVERY gate yourself, never credit the report. The disk is the only witness.

> ⛔ **You are a NEW instance.** You did NOT live the session above — it is a cache in a
> familiar voice. recolligere FIRST: fetch the grimoire + 4 primers (datamancy MCP — they
> are MCP RESOURCES via ReadMcpResourceTool, server `datamancy`, URI
> `https://datamancy.dev/<name>/SKILL.md`; NOT ToolSearch tools), `git log --oneline -15`,
> `git status`. Then OPEN THE TWO DESIGN DOCS before proposing anything:
> `../259-forced-hand/DESIGN-STONE-lost-locus-next-event.md` (the `:Lost`/next_event protocol)
> and this file's Track-1 (negatives + `print-raw'`). The locus-as-defprotocol contract +
> the `next_event` decomplect are the heart — see also memory
> `[[project_process_model_client_server_named_fd]]`. Do NOT propose from this summary — open
> the specs; the design was hard-won by the inquisitor crawl and the paraphrase will mislead.
