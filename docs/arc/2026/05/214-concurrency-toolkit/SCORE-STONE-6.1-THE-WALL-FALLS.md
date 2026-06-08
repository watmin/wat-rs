# SCORE — Stone 6.1: the wall falls

**Mode A.** Sonnet flight ~15 min (predicted 20–30 — under band).

## Scorecard (every row = orchestrator's own re-run/read)

| # | Row | Result |
|---|-----|--------|
| 1 | Gate-probe 61 2/2 GREEN; `src/typed_channel.rs` GONE (`ls`: no such file) | ✓ own runs |
| 2 | Home: `src/channel/` = mod.rs 81 + inner.rs 100 + transfer.rs 459 — **640 total (fresh wc)** vs the quarry's 694; the module doc's transport story carried + updated | ✓ read |
| 3 | `bounded<T>` DEAD; both tenants → `comms::thread::pair::<SpawnOutcome>()`; `ProgramHandleInner::InThread` holds `comms::thread::Receiver` directly | ✓ read diffs (value.rs + spawn sites) — same depth-1, same `Result<_, RecvError>` recv surface; STOP-1 did not fire |
| 4 | Sweep complete: 13 files + 3 new + 1 rm + 1 mv; pipes binary renamed (`wat_arc170_channel_pipes`) — names don't lie | ✓ gate-probe's tree scan GREEN = zero `typed_channel::` paths |
| 5 | χ-1 probe honestly reborn: its SUBJECT (the wrapper) died; now exercises `comms::thread::pair` with the arc-253 two-state `try_recv → Option` semantics correctly carried | ✓ read diff — a justified probe rewrite, documented in its header, not a gate-probe edit |
| 6 | lib 943/0/1 · nursery 863/4/4 (4 = parked-255; the 61-gate +2) · alpha 12/0/0 · pipes 23/0 (enveloped, --test-threads=1) · check --all-targets 0 errors · clippy src/channel/ 0 | ✓ own runs |
| 7 | FULL CORPUS 649/0/54, histogram all-zero | ✓ own run |

## The wall's standing

`typed_channel.rs` — the hand-wired stack's last load-bearing module, the
file Slice 6 was NAMED for — is dead. The seam it guarded (the two-tier
transport polymorphism that made the 5.1 floor-move possible) lives on in
`src/channel/`, warded-home shaped, awaiting its ward at 6.w. The wat
surface (make-channel / send / recv / select / close / the peer verbs) is
behaviorally byte-identical; the corpus never noticed.

**Remaining in Slice 6:** Stone 6.2 — the dead fork/spawn paths the compiler
already names (`eval_kernel_process_send`/`recv`,
`process_died_error_entry_form_failure`(+`_value`), + the fork.rs census) —
then 6.w (ward `src/channel/`, intueri-at-mint already implicit in the
layout; full vigilia → vigilatum).
