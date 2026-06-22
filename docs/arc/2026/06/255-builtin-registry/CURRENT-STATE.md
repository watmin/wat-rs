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

## ⏭️ NEXT: does 259.S3.6 FULLY unblock arc 255? — a builder DESIGN FORK stands first
259.S3.6 fixed the FRAMING half. But `BLOCKED-on-259-ipc-multiline.md` flagged a deeper
concern the framing fix does NOT settle: **how a parent reads a child's multi-line stdout.**
The four gold-standard `deftest-hermetic'` IPC tests want a child to `pprintln` a pretty
(multi-line) map and the parent to receive it as ONE value. Open question to GROUND before
writing them (start the crawl in `src/process/`, where the spawn-program' fd-wiring rehomed
per task #206 — it is NOT in process.rs):
- Does a process peer's `recv'` actually read the child's **stdout (fd 1)**? If yes, the
  framing fix means `recv'` now value-frames a child's multi-line `pprintln` → 255 is
  unblocked and the tests can be written directly.
- If `recv'` reads a SEPARATE comms channel (not fd 1), the BLOCKED note's design fork is
  still live: expose raw bound stdio on `Peer'` (a `Peer'/stdin`+`Peer'/stdout` accessor)
  vs a distinct stdio-program spawn shape; and whether `send'`/`recv'` coexist with raw
  handles or the process tier is raw-stdio per the 259 DESIGN. **This is a builder fork —
  surface it, do not unilaterally pick** (per settle-surface-before-grounding-impl).

## RESUME PATH (once the fork is settled + 255 confirmed unblocked)
Write the FOUR gold-standard `deftest-hermetic'` IPC tests — PRIMED ONLY (non-prime
deftest/`Process`-struct/`try-recv'` are DOOMED, arc-170 ~2-month migration): (1) round-trip
— child `pprintln`s the examples metadata map, parent receives one value → assert ==
compact; (2) over-cap; (3) truncated-frame [#267]; (4) anti-smuggling. Negatives via the
SUPERVISOR pattern: child crashes, parent `poll'` → `:Closed` (NOT `:Message`); `:Lost` is
remote-tier-only; there is NO in-process try/catch (let-it-crash). Then **#268** the
single-unbounded-LINE bound (a no-`\n` flood OOMs `read_line` before the frame cap fires —
a per-line byte bound; RED probe → build).

## SHIPPED earlier this session-cluster (255 stdio value-framing + symmetry; all pushed)
`695eca16` iv-c (metadata-of plain values + Kind/DefinedIn/Layer enums) · `e92f5333`
pprintln · `1632d02c` value-framing + symmetry (epprintln; Receiver value-frames) ·
`49cbe8ee` 512 KiB cap · `0854b081` `:max-buffer-bytes` escape hatch · `4fb86f8b`
`:wat::core::Value` EDN coerce target · `28853601` 259.S3.6 STRIKE-READY · `ecda39e2`
259.S3.6 GREEN.

## DISCIPLINE (memory written) — `feedback_probe_capability_before_delegating`
Probe the HARNESS/SUBSTRATE capability ("can it even OBSERVE a failure?") before delegating,
not just the feature. And: the weigh THIS session caught a build agent's report that listed a
sandbox-blocked gate (the proxy) as if checked AND missed a flaky parallel SIGKILL — re-run
EVERY gate yourself, never credit the report. The disk is the only witness.

> ⛔ **You are a NEW instance.** You did NOT live the session above — it is a cache in a
> familiar voice. recolligere FIRST: fetch the grimoire + 4 primers (datamancy MCP — they
> are MCP RESOURCES via ReadMcpResourceTool, server `datamancy`, URI
> `https://datamancy.dev/<name>/SKILL.md`; NOT ToolSearch tools), `git log --oneline -15`,
> `git status`, read `BLOCKED-on-259-ipc-multiline.md` + the 259.S3.6 DESIGN. Then GROUND the
> NEXT fork above against `src/process/` before proposing 255's tests. Do NOT propose from
> this summary — open the specs.
