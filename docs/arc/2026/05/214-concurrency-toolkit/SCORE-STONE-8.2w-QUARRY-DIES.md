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

---

## Phase B — VERDICT: the trio-completion vigilia CONVERGED; vigilatum stamped

**The cast** (14 wards, embed-never-fetch, one spell per worker, circumspicere
last with the inward map):

| Ward | Report | Disposition |
|---|---|---|
| intueri | 2 L2 | FIXED (doc rewrite; double-name collapsed) |
| solvere | 2 L2 | FIXED (require_one_arg de-duplicated; build_write_req) |
| conformare | 3 L2 | FIXED (the #189 span-debt closed in-home: with_thread_io + register signatures; 14 sites thread list_span; freeze boot spanless-by-domain) |
| purgare | CONVERGED | — |
| struere | 1 L2 | FIXED (with intueri's) |
| sequi | 1 L2 | RUNE earned (performance-counter; ZERO-MUTEX § honest-caveats cited) |
| temperare | 2 + 1 L3 | FIXED (clones → moves; SeqCst → Relaxed) |
| secare | CONVERGED | + structurally PROVED the routing safety (replies can't cross; Register FIFO-precedes first Req) |
| exigere | 2 L1 + 7 L2 | FIXED (1f-era future-tense docs rewritten to present truth; attested-arc rune on the 255 citation) |
| perspicere | 11 → 5 nouns | FIXED (ServiceReplySender/ReplyRegistry/ServiceInputSender/WriteAckRx/ReadReplyRx minted; 2 intentional-structure runes) |
| cernere | CONVERGED | full traceability table (every wat form → declaration) |
| probare | ship-as-is ×3 | every form Expressed |
| conferre | 1 L2 | FIXED (DESIGN Stone-8.1 bullet → the lifted truth) |
| **circumspicere** | **2 L1 + 2 L2** | F1: six diagnostics now speak tagged EDN (#wat.substrate/Diag; the mechanical GATE stays tracked at the 109 NOTE / #201). F2: **the field-order invariant GATE lives** (a wat defstruct reorder is now a red build, never a silent mis-route). F3: guard-arm tests prove the loop survives malformed Reqs. F4: "canonical rig" softened to reference-pattern + review-time enforcement. |

**Orchestrator scoring catches at the sweep's return:**
- The live diagnostics showed 15 files importing the deleted name — sonnet's
  zero-grep + 0-errors claims looked contradicted; MY OWN grep + cargo check
  confirmed sonnet TRUE (LSP lag, second occurrence this stone — the t.v.
  cuts both ways: weigh even the accusations).
- **mora catch in fresh code**: the H2 guard-arm tests used
  `thread::sleep(50ms)` — sleep is a guess; both Reqs ride the same FIFO
  channel, so the valid Req's reply arriving IS the ordering proof. Both
  sleeps removed (orchestrator edit); 6/6 green in 0.01s.

**Convergence gates (all orchestrator-run):** lib 943/0/1 · nursery 861/4/4
(the 4 = parked arc-255; +6 new gates GREEN) · alpha 12/0/0 ·
check --all-targets 0 errors · clippy-in-home 0 · zero-greps
(Span::unknown in verbs.rs / uninstall_ambient_stdio / BRIEF Q5) all ZERO ·
FULL CORPUS 649/0, histogram all-zero.

**Tally: 27 fixes + 4 earned runes; zero deferrals; L1+L2 = 0.**

**vigilatum: 2026-06-08T00:07:11Z** — inscribed in src/services/mod.rs; the
ward note's promise fulfilled and rewritten as record. The home stands
watched: stdout + stderr + stdin, one general form, one guard.
