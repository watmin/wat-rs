# ⛔ CURRENT STATE (breadcrumb, 2026-06-21; replace in place) — read the DESIGN docs, not this paraphrase

## ⚡ FIRST: a sonnet is building IN THE BACKGROUND
Agent `aa6279e3c95106c8d` is building **arc 259.S3.6 — one frame-finder** (the decomplect that
unblocks everything). STRIKE-READY committed (`28853601`): RED probe
`tests/nursery/probe_arc259_comms_recv_multiline_frame.rs` + `docs/arc/2026/06/259-forced-hand/
DESIGN-STONE-259.S3.6-one-frame-finder.md`. When it lands (or if it died in the compaction):
**WEIGH it against the disk** (re-run the gates in the DESIGN §Gate yourself — comms probe green +
non-vacuous, ambient framing tests unchanged, comms/channel/lib/nursery floors, the live proxy
`wat intrinsic-metadata.wat | wat read-flat.wat`), then commit on green. If the agent is gone,
the DESIGN is self-contained — re-fire or build it.

## What 259.S3.6 is (the decomplect)
TWO fd framers do newline-detection independently: `read_framed_edn` (edn_shim, value-frames the
ambient/WatReader path) and comms `take_frame` (process.rs:849, splits on first `\n`). So a multi-line
value crossing a process peer is mis-framed (`recv'` reads only `{`). Fix = extract ONE
`next_complete_frame` (pure byte-level: scan `\n`, `edn_frame_status` per prefix, first Complete →
Frame(end), `DEFAULT_MAX_FRAME_BYTES` cap, anti-smuggle), route BOTH readers through it. I/O backends
(blocking vs io_uring) stay separate (reactor = out of scope); only the FRAMING unifies.

## Floor (HEAD `28853601`): lib 953 pass / 36 fail / 1 ign (the 36 are PRE-EXISTING, do not chase);
## nursery 915 pass / 4 fail (4 pre-existing RED-by-design) + the iv-c/pprintln/framing/cap probes green.

## SHIPPED this session (committed, pushed; arc 255 stdio value-framing + symmetry)
- `695eca16` iv-c — metadata-of off the holon encoder → plain values + Kind/DefinedIn/Layer enums.
- `e92f5333` pprintln; `1632d02c` stdio value-framing + symmetry (epprintln; Receiver/from-pipe
  value-frames; anti-smuggle); `49cbe8ee` 512 KiB accumulator cap.
- `0854b081` `:max-buffer-bytes` escape hatch (readln MACRO over readln' PRIME; MAX-READLN-BYTES wat
  def = single source; corpse readln-intrinsic deleted).
- `4fb86f8b` `:wat::core::Value` as an EDN coerce target (read/write symmetry — UP-free decode).
- Proxy proof KEPT: `wat-scripts/intrinsic-metadata.wat | wat-scripts/read-flat.wat`.

## RESUME PATH (after 259.S3.6 lands + commits)
255 unblocks → write the FOUR gold-standard `deftest-hermetic'` IPC tests (the builder wants ALL four,
PRIMED only — non-prime deftest is doomed): (1) round-trip — child `pprintln`s the examples metadata
map, parent `recv'`s it (now value-framed) → assert == compact; (2) over-cap, (3) truncated-frame
[#267], (4) anti-smuggling — negatives via the SUPERVISOR pattern: child crashes, parent `poll'` →
`:Closed` (NOT `:Message`); `:Lost` is remote-tier-only, there is NO in-process try/catch (let-it-crash).
Then **#268** the single-unbounded-LINE bound (a no-`\n` flood OOMs `read_line` before the frame cap
fires — a per-line byte bound, RED probe → build).

## DISCIPLINE proven this session (memory written)
- `feedback_probe_capability_before_delegating` — probe the HARNESS/SUBSTRATE capability ("can it even
  OBSERVE a failure?") before delegating a strike, not just the feature. I fired a test sonnet blind →
  157k-token trash. The inquisition (deep grounding) is the flight you don't take.
- 255 BLOCKED note: `BLOCKED-on-259-ipc-multiline.md`. The non-prime `deftest`/`Process`-struct/
  `try-recv'` are DOOMED (arc-170 migration, ~2 months) — primed only.

> ⛔ **You are a NEW instance.** You did NOT live the long session above — it's a cache in a familiar
> voice. recolligere FIRST: fetch the grimoire + 4 primers (datamancy MCP), `git log --oneline -15`,
> `git status`, read the two 259.S3.6 docs + this BLOCKED note. The in-flight agent
> `aa6279e3c95106c8d` is the live frontier — check `/workflows` or TaskList for its result; weigh
> against the disk before trusting any report. Do NOT propose from this summary — open the specs.
