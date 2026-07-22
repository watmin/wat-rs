# NOTE — tier-head → unified `Peer'` cleanup (the `ProcessSelectable::Timer` purge + what arc-278 Stone 1 advanced)

**The end-state (arc-109 / arc-170):** everything unifies to the fd-backed **`Peer'`**; the tier heads
(`Thread'` / `Process'`) and `Timer'` vanish; `select'` and `poll'` take **only** `Peer'` (the TODO at
`src/check.rs:12157-12160` — *"the unified fd-backed `Peer'` end-state makes `select'` take ONLY `Peer'`;
the tier heads vanish. Kept now to unblock the loci-agnostic bracket (259 S3a)"*).

## What arc-278 Stone 1 (2026-07-21) advanced toward it

Relocating the timer to the correct location (`DESIGN-self-scheduling-defservices.md`, § Stone 1) moved
the substrate two steps closer to this end-state:

- **`after` now returns a unified `Peer'<nil,O>`**, not a tier-specific `Timer'<O>` (`runtime.rs`
  `eval_kernel_after`; `check.rs` `infer_kernel_after`). The **tier-open `Timer'` type + its fusion
  machinery are retired** (`check.rs` — the `select'` `Timer'` element arm, the three unify-fusion arms,
  `is_peer_tier_head`, and the 4 fusion unit tests). One tier head class gone.
- **`select'` gained process-tier `Peer'` support** — the first real consumer (a process-tier `after`
  timer) forced closing the **C0b.3a-ii deferral**: `eval_peer_select_prime` now dispatches on
  `reactor_class` (thread crossbeam / process io_uring, mirroring `poll'`'s shipped `select_raw` +
  `decode_trusted_wire` client arm). `select'` over a unified `Peer'` is now **thread ≡ process** — a
  latent gap the timer relocation surfaced (`ALIVS ARGVIT` at the substrate).

## The cleanup OWED (tracked here, not deferred loosely)

- **`ProcessSelectable::Timer` is now DEAD** — `runtime.rs` still declares the enum variant and matches
  it in ~5 places (`send'`/`recv'`/`select'` arms), but it is **never constructed** (the process `after`
  now builds a unified `PEER_TYPE_PATH` peer, not a `ProcessSelectable::Timer`). Harmless (`pub` variant,
  no warning) but dead-but-matched. **Purge the variant + its arms** — a small, isolated cleanup
  (`grep -rn "ProcessSelectable::Timer" src/`). Do it as its own strike; verify the floor `--release`.
- **The larger remainder:** `Thread'` / `Process'` tier heads still exist for **spawned** peers
  (`spawn-thread'`/`spawn-process'`; `select'`/`poll'` still accept them for non-timer use). Collapsing
  those to the unified `Peer'` is the full arc-109/170 end-state — a larger body of work, not this note.
