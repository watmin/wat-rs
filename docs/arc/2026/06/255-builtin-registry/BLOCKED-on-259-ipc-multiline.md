# ⛔ ARC 255 — MOMENTARILY BLOCKED on a 259 enhancement (2026-06-21)

> **UPDATE 2026-06-21 (`ecda39e2`): the FRAMING half is RESOLVED.** 259.S3.6 (one
> frame-finder) landed: comms `recv`/`take_frame` now value-frames multi-line EDN (was
> first-`\n` split). The remaining live question is the SECOND bullet below — whether a
> process peer's `recv'` reads the child's **stdout (fd 1)** (→ 255 unblocked, write the
> tests) or a separate channel (→ the raw-bound-stdio design fork at the bottom is still
> live; settle it with the builder). GROUND this in `src/process/` (the spawn-program' fd
> wiring, rehomed per task #206 — NOT in process.rs). See `CURRENT-STATE.md`.

255's stdio value-framing work all SHIPPED (committed, pushed):
- metadata-of plain values + enum flip (iv-c, `695eca16`)
- pprintln (`e92f5333`); stdio value-framing + symmetry incl. epprintln (`1632d02c`)
- the 512 KiB accumulator cap (`49cbe8ee`)
- the `:max-buffer-bytes` escape hatch — readln macro + readln' prime + MAX-READLN-BYTES (`0854b081`)
- `:wat::core::Value` as an EDN coerce target — read/write symmetry (`4fb86f8b`)

Live-proven cross-process (the proxy): `wat wat-scripts/intrinsic-metadata.wat |
wat wat-scripts/read-flat.wat` round-trips the examples metadata pretty→one-value→flat.
KEEP those scripts as the proxy proof.

## Why blocked
The GOLD-STANDARD deftest-hermetic' IPC tests (round-trip + over-cap + truncated +
anti-smuggling) want to pass a MULTI-LINE value genuinely BETWEEN processes. They can't,
because:
- the multi-line value-framing is an AMBIENT-stdio capability (readln/read-frame/
  Receiver-from-pipe). The primed typed channel (`send'`/`recv'`, comms) is
  **newline-framed COMPACT** (process.rs:51 — assumes single-line wat-edn); multi-line
  breaks it.
- the primed `Peer'` exposes only `send'`/`recv'`/`poll'`/`close'` — **no raw bound
  stdio handle.** A parent cannot write raw (multi-line) to a process child's stdin nor
  read its stdout. Raw handles exist only on the DOOMED arc-170-1f `Process` struct.

So there is no PRIMED transport for a multi-line stream between processes. This is the
259 divergence: DESIGN.md §"ProcessProg — a stdio :user::main" says *"the process clause
binds the child's 0/1/2 to the comms pipe; the parent reads/writes the bound handles …
the child does ambient readln/println"* — but the impl built a typed-compact channel and
never exposed the raw bound handles.

## Unblock = 259 feature enhancement
Expose the process peer's raw bound stdio on the primed `Peer'` (realize the DESIGN):
a parent gets an `IOWriter` to the child's stdin + an `IOReader` from its stdout, so it
can write-pretty (multi-line) and read-frame. Then the gold-standard test IS the literal
`wat A | wat B`: parent orchestrates, child B is the ambient read-flat program.

Design fork to settle with the builder (process-peer stdio contract — the real 259 blocker):
how raw bound handles are exposed (a `Peer'/stdin`/`Peer'/stdout` accessor vs a distinct
stdio-program spawn shape), and whether `send'`/`recv'` coexist with raw handles on one
peer or the process tier is raw-stdio per the DESIGN (thread = typed channel).

## Resume 255 when: 259 exposes a primed raw multi-line process-stdio path → then write the
## four gold-standard deftest-hermetic' IPC tests (#267 truncated rides along) + #268 the
## unbounded-line bound.
