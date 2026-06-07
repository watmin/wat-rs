# SCORE — Stone 8.2w: the quarry dies

## Phase A — the lift (sonnet, Mode A, ~16 min vs 15–25 predicted)

| # | Row | Result |
|---|-----|--------|
| 1 | Gate-probe 82w 2/2 GREEN | ✓ own run (after R1 below) |
| 2 | `src/thread_io.rs` GONE | ✓ `ls`: No such file or directory; git rm staged |
| 3 | Home shape: mod.rs 85 (index + flat re-exports) / peer.rs 171 / client.rs 341 / verbs.rs 286 — 883 total (fresh `wc -l`) | ✓ read mod.rs whole; the module doc carries the whole architecture + all four contracts + the ward note |
| 4 | Sweep: 36 files + 3 new; zero `thread_io::` paths anywhere incl. the signal.rs:502 diagnostic STRING | ✓ probe_2 (the scanner) GREEN post-R1 |
| 5 | Behavior-identical | ✓ lib 943/0/1 · alpha 12/0/0 · nursery 855/4/4 (4 = the parked-255 reds) · check --all-targets 0 errors · clippy src/services/ zero |
| 6 | FULL CORPUS | ✓ 649/0/54, histogram all-zero |
| 7 | Dead-code parity (no orphans minted by the lift) | ✓ `never used` count 6 at HEAD = 6 in working tree (the condemned process verbs — Slice 6's quarry) |

**Orchestrator catch R1 — the tombstone read its own epitaph (MY probe defect).**
probe_2's scanner walked tests/ and matched its own source (5 hits: the
doc-comments and needle strings that say `thread_io::`). Sonnet correctly
refused to edit the read-only probe, traced the failure exactly, and reported
it as a probe defect — the right behavior under a defective gate (second time
this campaign the gate's defect was mine: 8.1b's Gate-2 baseline, now 8.2w's
self-scan). Fixed by the probe's author: explicit self-exclusion with the
incident inscribed at the skip.

**Sonnet's honest delta accepted**: `with_thread_io` is `pub(crate)` (not the
BRIEF's `pub(super)`) — `pub(super)` cannot be re-exported through mod.rs for
verbs.rs; `pub(crate)` delivers the intent and the compiler agrees.

**The annihilation map: TERMINAL.** 979 → 903 → 635 → **0, git rm**. The
gate-probe asserts the absence forever. The deadlock's mechanism, its file,
and its name are gone from the live tree; the survivors live in a home whose
index tells the whole story.

## Phase B — the trio-completion vigilia (appended at convergence)

Pending: the full watch on `src/services/` + `wat/kernel/services/*.wat` —
universal seven + exigere + secare + perspicere (+excusare/mora if triggered)
+ cernere/probare/conferre on the wat half + circumspicere last. Stamp lands
at L1+L2=0.
